//! Shared WebSocket connection quotas.

use std::{
    collections::HashMap,
    net::IpAddr,
    sync::{Arc, Mutex},
};

/// Maximum concurrently upgraded WebSocket connections per process.
pub const MAX_WEBSOCKET_CONNECTIONS: usize = 64;
/// Maximum concurrently upgraded WebSocket connections per authenticated user.
pub const MAX_WEBSOCKET_CONNECTIONS_PER_USER: usize = 4;
/// Maximum concurrently upgraded WebSocket connections per peer IP address.
pub const MAX_WEBSOCKET_CONNECTIONS_PER_IP: usize = 16;

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
