//! Shared WebSocket connection quotas.

use std::{
    collections::HashMap,
    net::IpAddr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

/// Maximum concurrently upgraded WebSocket connections per process.
pub const MAX_WEBSOCKET_CONNECTIONS: usize = 64;
/// Maximum concurrently upgraded WebSocket connections per authenticated user.
pub const MAX_WEBSOCKET_CONNECTIONS_PER_USER: usize = 4;
/// Maximum concurrently upgraded WebSocket connections per peer IP address.
pub const MAX_WEBSOCKET_CONNECTIONS_PER_IP: usize = 16;

/// Failed `/ws/v1` authentications allowed per peer IP in one window.
pub const MAX_WEBSOCKET_AUTH_FAILURES_PER_IP: u32 = 5;
/// Duration over which failed `/ws/v1` authentications are counted.
pub const WEBSOCKET_AUTH_FAILURE_WINDOW: Duration = Duration::from_secs(60);
/// Maximum number of peer IP entries retained by authentication limiter.
pub const MAX_WEBSOCKET_AUTH_FAILURE_IPS: usize = 4096;

#[derive(Debug, Default)]
struct FailedAuthEntry {
    failures: u32,
    reservations: u32,
    window_start: Option<Instant>,
    last_seen: Option<Instant>,
}

/// Bounded in-memory limiter for failed WebSocket authentication attempts.
#[derive(Clone, Debug, Default)]
pub struct WebSocketAuthFailureLimiter {
    state: Arc<Mutex<HashMap<IpAddr, FailedAuthEntry>>>,
}

/// Reservation for one WebSocket authentication attempt.
pub struct WebSocketAuthAttempt {
    limiter: WebSocketAuthFailureLimiter,
    ip: IpAddr,
    admitted_at: Instant,
    successful: bool,
}

impl WebSocketAuthAttempt {
    /// Mark authentication successful so reservation does not consume failure budget.
    pub fn mark_success(&mut self) {
        self.successful = true;
    }
}

impl Drop for WebSocketAuthAttempt {
    fn drop(&mut self) {
        self.limiter
            .finish_attempt(self.ip, self.successful, self.admitted_at);
    }
}

impl WebSocketAuthFailureLimiter {
    /// Atomically reserve one authentication attempt, or reject at the threshold.
    ///
    /// Reservation records failure on drop unless marked successful. This closes
    /// the check-then-record race between concurrent authentication requests.
    ///
    /// # Errors
    /// Returns `None` when this peer reached the failure threshold.
    ///
    /// # Panics
    /// Panics if limiter mutex poisoned by a prior panic.
    #[must_use]
    pub fn begin_attempt(&self, ip: IpAddr, now: Instant) -> Option<WebSocketAuthAttempt> {
        let mut state = self
            .state
            .lock()
            .expect("WebSocket auth limiter lock poisoned");
        if !state.contains_key(&ip) && state.len() >= MAX_WEBSOCKET_AUTH_FAILURE_IPS {
            let oldest = state
                .iter()
                .filter(|(_, entry)| entry.reservations == 0)
                .min_by_key(|(address, entry)| (entry.last_seen, **address))
                .map(|(address, _)| *address)?;
            state.remove(&oldest);
        }
        let entry = state.entry(ip).or_default();
        if now.duration_since(entry.window_start.unwrap_or(now)) >= WEBSOCKET_AUTH_FAILURE_WINDOW {
            entry.failures = 0;
            entry.window_start = Some(now);
        }
        if entry.failures.saturating_add(entry.reservations) >= MAX_WEBSOCKET_AUTH_FAILURES_PER_IP {
            return None;
        }
        entry.window_start.get_or_insert(now);
        entry.last_seen = Some(now);
        entry.reservations += 1;
        Some(WebSocketAuthAttempt {
            limiter: self.clone(),
            ip,
            admitted_at: now,
            successful: false,
        })
    }

    fn finish_attempt(&self, ip: IpAddr, successful: bool, now: Instant) {
        let mut state = self
            .state
            .lock()
            .expect("WebSocket auth limiter lock poisoned");
        let Some(entry) = state.get_mut(&ip) else {
            return;
        };
        entry.reservations = entry.reservations.saturating_sub(1);
        if !successful {
            if now.duration_since(entry.window_start.unwrap_or(now))
                >= WEBSOCKET_AUTH_FAILURE_WINDOW
            {
                entry.failures = 0;
                entry.window_start = Some(now);
            }
            entry.failures = entry.failures.saturating_add(1);
            entry.last_seen = Some(now);
        }
    }

    #[cfg(test)]
    fn entry_count(&self) -> usize {
        self.state
            .lock()
            .expect("WebSocket auth limiter lock poisoned")
            .len()
    }
}

#[derive(Debug, Default)]
struct QuotaState {
    global: usize,
    users: HashMap<i64, usize>,
    ips: HashMap<IpAddr, usize>,
}

/// Shared, transactional WebSocket quota state.
#[derive(Clone, Debug, Default)]
pub struct WebSocketQuota {
    state: Arc<Mutex<QuotaState>>,
}

/// Permit held for lifetime of one WebSocket connection.
pub struct WebSocketQuotaGuard {
    quota: WebSocketQuota,
    user_id: i64,
    ip: IpAddr,
}

/// Quota rejection reason.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuotaRejection {
    /// Process-wide limit reached.
    Global,
    /// User limit reached.
    User,
    /// IP limit reached.
    Ip,
}

impl WebSocketQuota {
    /// Reserve one connection, or return first limit that prevents reservation.
    ///
    /// # Errors
    /// Returns quota rejection when global, user, or IP limit reached.
    ///
    /// # Panics
    /// Panics if quota mutex poisoned by a prior panic.
    pub fn try_reserve(
        &self,
        user_id: i64,
        ip: IpAddr,
    ) -> Result<WebSocketQuotaGuard, QuotaRejection> {
        let mut state = self.state.lock().expect("WebSocket quota lock poisoned");
        if state.global >= MAX_WEBSOCKET_CONNECTIONS {
            return Err(QuotaRejection::Global);
        }
        if state.users.get(&user_id).copied().unwrap_or(0) >= MAX_WEBSOCKET_CONNECTIONS_PER_USER {
            return Err(QuotaRejection::User);
        }
        if state.ips.get(&ip).copied().unwrap_or(0) >= MAX_WEBSOCKET_CONNECTIONS_PER_IP {
            return Err(QuotaRejection::Ip);
        }

        state.global += 1;
        *state.users.entry(user_id).or_default() += 1;
        *state.ips.entry(ip).or_default() += 1;
        Ok(WebSocketQuotaGuard {
            quota: self.clone(),
            user_id,
            ip,
        })
    }

    fn release(&self, user_id: i64, ip: IpAddr) {
        let mut state = self.state.lock().expect("WebSocket quota lock poisoned");
        state.global -= 1;
        decrement(&mut state.users, &user_id);
        decrement(&mut state.ips, &ip);
    }

    #[cfg(test)]
    fn counts(&self, user_id: i64, ip: IpAddr) -> (usize, usize, usize) {
        let state = self.state.lock().expect("WebSocket quota lock poisoned");
        (
            state.global,
            state.users.get(&user_id).copied().unwrap_or(0),
            state.ips.get(&ip).copied().unwrap_or(0),
        )
    }
}

fn decrement<K: Eq + std::hash::Hash>(map: &mut HashMap<K, usize>, key: &K) {
    if let Some(count) = map.get_mut(key) {
        *count -= 1;
        if *count == 0 {
            map.remove(key);
        }
    }
}

impl Drop for WebSocketQuotaGuard {
    fn drop(&mut self) {
        self.quota.release(self.user_id, self.ip);
    }
}

#[cfg(test)]
#[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
mod tests {
    use super::*;
    use std::{
        net::{IpAddr, Ipv4Addr},
        sync::{Arc, Barrier},
        thread,
    };

    fn ip() -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1))
    }

    #[test]
    fn failed_auth_limiter_blocks_after_threshold_and_resets() {
        let limiter = WebSocketAuthFailureLimiter::default();
        let start = Instant::now();
        for _ in 0..MAX_WEBSOCKET_AUTH_FAILURES_PER_IP {
            drop(limiter.begin_attempt(ip(), start).expect("reserve"));
        }
        assert!(limiter.begin_attempt(ip(), start).is_none());
        assert!(limiter
            .begin_attempt(ip(), start + WEBSOCKET_AUTH_FAILURE_WINDOW)
            .is_some());
    }

    #[test]
    fn failed_auth_limiter_evicts_oldest_ip_at_bound() {
        let limiter = WebSocketAuthFailureLimiter::default();
        let start = Instant::now();
        for value in 0..MAX_WEBSOCKET_AUTH_FAILURE_IPS {
            drop(
                limiter
                    .begin_attempt(
                        IpAddr::V4(Ipv4Addr::new(198, 51, (value / 256) as u8, value as u8)),
                        start + Duration::from_secs(value as u64),
                    )
                    .expect("reserve"),
            );
        }
        let newest = IpAddr::V4(Ipv4Addr::new(203, 0, 113, 1));
        drop(limiter.begin_attempt(newest, start + Duration::from_secs(10_000)));
        assert_eq!(limiter.entry_count(), MAX_WEBSOCKET_AUTH_FAILURE_IPS);
        assert!(limiter
            .begin_attempt(newest, start + Duration::from_secs(10_000))
            .is_some());
    }

    #[test]
    fn full_table_rejects_new_ip_without_evicting_active_entry() {
        let limiter = WebSocketAuthFailureLimiter::default();
        let start = Instant::now();
        let original_ip = ip();
        let original = limiter.begin_attempt(original_ip, start).expect("reserve");
        let mut attempts = vec![original];
        for value in 1..MAX_WEBSOCKET_AUTH_FAILURE_IPS {
            attempts.push(
                limiter
                    .begin_attempt(
                        IpAddr::V4(Ipv4Addr::new(198, 51, (value / 256) as u8, value as u8)),
                        start,
                    )
                    .expect("reserve"),
            );
        }
        assert_eq!(limiter.entry_count(), MAX_WEBSOCKET_AUTH_FAILURE_IPS);
        assert!(limiter
            .begin_attempt(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 1)), start)
            .is_none());

        drop(attempts);
        for _ in 0..MAX_WEBSOCKET_AUTH_FAILURES_PER_IP - 1 {
            drop(limiter.begin_attempt(original_ip, start).expect("reserve"));
        }
        assert!(limiter.begin_attempt(original_ip, start).is_none());
    }

    #[test]
    fn successful_attempt_does_not_consume_budget() {
        let limiter = WebSocketAuthFailureLimiter::default();
        let start = Instant::now();
        for _ in 0..100 {
            let mut attempt = limiter.begin_attempt(ip(), start).expect("reserve");
            attempt.mark_success();
        }
        for _ in 0..MAX_WEBSOCKET_AUTH_FAILURES_PER_IP {
            drop(limiter.begin_attempt(ip(), start).expect("reserve"));
        }
        assert!(limiter.begin_attempt(ip(), start).is_none());
    }

    #[test]
    fn concurrent_reservations_never_exceed_failure_limit() {
        let limiter = Arc::new(WebSocketAuthFailureLimiter::default());
        let barrier = Arc::new(Barrier::new(32));
        let threads: Vec<_> = (0..32)
            .map(|_| {
                let limiter = Arc::clone(&limiter);
                let barrier = Arc::clone(&barrier);
                thread::spawn(move || {
                    barrier.wait();
                    limiter.begin_attempt(ip(), Instant::now())
                })
            })
            .collect();
        let attempts: Vec<_> = threads
            .into_iter()
            .map(|thread| thread.join().unwrap())
            .collect();
        assert_eq!(
            attempts.iter().filter(|attempt| attempt.is_some()).count(),
            MAX_WEBSOCKET_AUTH_FAILURES_PER_IP as usize
        );
    }

    #[test]
    fn accepts_until_each_limit() {
        let quota = WebSocketQuota::default();
        let permits: Vec<_> = (0..MAX_WEBSOCKET_CONNECTIONS_PER_USER)
            .map(|_| quota.try_reserve(1, ip()).expect("reserve"))
            .collect();
        assert_eq!(permits.len(), 4);
        assert!(matches!(
            quota.try_reserve(1, ip()),
            Err(QuotaRejection::User)
        ));
        assert_eq!(quota.counts(1, ip()), (4, 4, 4));
    }

    #[test]
    fn rejects_ip_and_global_limits() {
        let quota = WebSocketQuota::default();
        let ip_permits: Vec<_> = (0..MAX_WEBSOCKET_CONNECTIONS_PER_IP)
            .map(|user| quota.try_reserve(user as i64, ip()).expect("reserve"))
            .collect();
        assert!(matches!(
            quota.try_reserve(100, ip()),
            Err(QuotaRejection::Ip)
        ));
        drop(ip_permits);
        let permits: Vec<_> = (0_i64..MAX_WEBSOCKET_CONNECTIONS as i64)
            .map(|user| {
                quota
                    .try_reserve(user, IpAddr::V4(Ipv4Addr::new(198, 51, 100, user as u8)))
                    .expect("reserve")
            })
            .collect();
        assert!(matches!(
            quota.try_reserve(100, IpAddr::V4(Ipv4Addr::new(192, 0, 2, 2))),
            Err(QuotaRejection::Global)
        ));
        drop(permits);
    }

    #[test]
    fn guard_releases_connection() {
        let quota = WebSocketQuota::default();
        let permit = quota.try_reserve(1, ip()).expect("reserve");
        assert_eq!(quota.counts(1, ip()), (1, 1, 1));
        drop(permit);
        assert_eq!(quota.counts(1, ip()), (0, 0, 0));
    }

    #[test]
    fn failed_reservation_rolls_back_nothing() {
        let quota = WebSocketQuota::default();
        let permits: Vec<_> = (0..MAX_WEBSOCKET_CONNECTIONS_PER_USER)
            .map(|_| quota.try_reserve(1, ip()).expect("reserve"))
            .collect();
        assert!(matches!(
            quota.try_reserve(1, IpAddr::V4(Ipv4Addr::new(192, 0, 2, 2))),
            Err(QuotaRejection::User)
        ));
        assert_eq!(quota.counts(1, ip()), (4, 4, 4));
        drop(permits);
    }

    #[test]
    fn concurrent_reservations_never_exceed_limit() {
        let quota = Arc::new(WebSocketQuota::default());
        let barrier = Arc::new(Barrier::new(32));
        let threads: Vec<_> = (0..32)
            .map(|_| {
                let quota = Arc::clone(&quota);
                let barrier = Arc::clone(&barrier);
                thread::spawn(move || {
                    barrier.wait();
                    quota.try_reserve(7, ip())
                })
            })
            .collect();
        let permits: Vec<_> = threads
            .into_iter()
            .filter_map(|t| t.join().expect("thread").ok())
            .collect();
        assert_eq!(permits.len(), MAX_WEBSOCKET_CONNECTIONS_PER_USER);
        assert_eq!(
            quota.counts(7, ip()),
            (
                MAX_WEBSOCKET_CONNECTIONS_PER_USER,
                MAX_WEBSOCKET_CONNECTIONS_PER_USER,
                MAX_WEBSOCKET_CONNECTIONS_PER_USER,
            )
        );
        drop(permits);
    }
}
