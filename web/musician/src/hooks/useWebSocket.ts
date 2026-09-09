import { useEffect, useRef, useState, useCallback } from 'react';
import type { ServerMessage, StateSnapshot } from '../protocol/types';
import { encodeEnvelope } from '../protocol/envelope';
import type { ClientMessage } from '../protocol/types';

export type WsStatus = 'disconnected' | 'connecting' | 'connected' | 'error';

export interface UseWebSocketResult {
  status: WsStatus;
  revision: number | null;
  snapshot: StateSnapshot | null;
  error: string | null;
  send: (msg: ClientMessage) => void;
  disconnect: () => void;
}

function isValidRevision(value: unknown): value is number {
  return typeof value === 'number' && Number.isSafeInteger(value) && value >= 0;
}

function updateRevision(current: number | null, candidate: unknown): number | null {
  if (!isValidRevision(candidate)) return current;
  return current === null || candidate >= current ? candidate : current;
}

function isFiniteNumber(value: unknown): value is number {
  return typeof value === 'number' && Number.isFinite(value);
}

function isStateSnapshot(value: unknown): value is StateSnapshot {
  if (!value || typeof value !== 'object') return false;
  const snapshot = value as StateSnapshot;
  if (snapshot.schema_version !== 1 || !isValidRevision(snapshot.revision)
    || !Array.isArray(snapshot.channels) || !Array.isArray(snapshot.mixes)) return false;
  if (snapshot.channels.some((channel) => !channel || !Number.isInteger(channel.index)
    || !isFiniteNumber(channel.gain_db) || typeof channel.muted !== 'boolean')) return false;
  return snapshot.mixes.every((mix) => mix && Number.isInteger(mix.index)
    && isFiniteNumber(mix.master_gain_db) && typeof mix.master_muted === 'boolean'
    && Array.isArray(mix.sends) && mix.sends.every((send) => send
      && Number.isInteger(send.channel_index) && isFiniteNumber(send.gain_db)
      && isFiniteNumber(send.pan) && send.pan >= -1 && send.pan <= 1
      && typeof send.muted === 'boolean'));
}

function isServerMessage(value: unknown): value is ServerMessage {
  if (!value || typeof value !== 'object') return false;
  const message = value as { type?: unknown; data?: Record<string, unknown> };
  if (message.type === 'State') {
    return isValidRevision(message.data?.revision);
  }
  if (message.type === 'SendAck') {
    return isValidRevision(message.data?.revision)
      && Number.isInteger(message.data?.mix_index)
      && Number.isInteger(message.data?.channel_index)
      && typeof message.data?.gain_db === 'number'
      && Number.isFinite(message.data.gain_db)
      && typeof message.data?.pan === 'number'
      && Number.isFinite(message.data.pan)
      && typeof message.data?.muted === 'boolean';
  }
  if (message.type === 'Error') {
    return typeof message.data?.code === 'string' && typeof message.data.message === 'string';
  }
  return false;
}

/**
 * WebSocket hook for Open IEM control plane.
 *
 * Token is passed as a URL query parameter because browser WebSocket API
 * does not support custom headers. This is a known limitation documented
 * in docs/security/WEBSOCKET-TOKEN-TRANSPORT.md.
 * Mitigations: token is short-lived (15 min), TLS mandatory in production,
 * connection is LAN-only.
 */
export function useWebSocket(token: string | null): UseWebSocketResult {
  const [status, setStatus] = useState<WsStatus>('disconnected');
  const [revision, setRevision] = useState<number | null>(null);
  const [snapshot, setSnapshot] = useState<StateSnapshot | null>(null);
  const [error, setError] = useState<string | null>(null);
  const wsRef = useRef<WebSocket | null>(null);
  const latestRevisionRef = useRef<number | null>(null);
  const mountedRef = useRef(true);

  const disconnect = useCallback(() => {
    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }
    setStatus('disconnected');
  }, []);

  const send = useCallback((msg: ClientMessage) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(encodeEnvelope(msg));
    }
  }, []);

  useEffect(() => {
    mountedRef.current = true;
    return () => {
      mountedRef.current = false;
    };
  }, []);

  useEffect(() => {
    if (!token) {
      disconnect();
      setStatus('disconnected');
      return;
    }

    const url = `/ws/v1?token=${encodeURIComponent(token)}`;
    setStatus('connecting');
    setRevision(null);
    latestRevisionRef.current = null;
    setSnapshot(null);

    const ws = new WebSocket(url);
    wsRef.current = ws;

    let snapshotRequestInFlight = false;
    const refreshSnapshot = async () => {
      if (snapshotRequestInFlight) return;
      snapshotRequestInFlight = true;
      try {
        const response = await fetch('/api/v1/state', {
          headers: { Authorization: `Bearer ${token}` },
        });
        if (!response.ok) throw new Error(`Snapshot failed: ${response.status}`);
        const value: unknown = await response.json();
        if (!isStateSnapshot(value)) throw new Error('Malformed state snapshot');
        if (!mountedRef.current || wsRef.current !== ws) return;
        const latestRevision = latestRevisionRef.current;
        if (latestRevision !== null && value.revision < latestRevision) return;
        setSnapshot((current) => current === null || value.revision >= current.revision ? value : current);
        latestRevisionRef.current = value.revision;
        setRevision((current) => updateRevision(current, value.revision));
      } catch (err) {
        if (mountedRef.current && wsRef.current === ws) {
          setError(err instanceof Error ? err.message : 'Snapshot failed');
        }
      } finally {
        snapshotRequestInFlight = false;
      }
    };

    ws.onopen = () => {
      if (!mountedRef.current || wsRef.current !== ws) return;
      setStatus('connected');
      setError(null);
      // Request initial state over WS, then reconcile full state over REST.
      ws.send(encodeEnvelope({ type: 'GetState' }));
      void refreshSnapshot();
    };

    ws.onmessage = (event: MessageEvent<string>) => {
      if (!mountedRef.current || wsRef.current !== ws) return;
      try {
        const parsed: unknown = JSON.parse(event.data);
        const payload = parsed && typeof parsed === 'object' && 'payload' in parsed
          ? (parsed as { payload?: unknown }).payload
          : undefined;
        if (!isServerMessage(payload)) {
          setError('Malformed server message');
          return;
        }
        const msg = payload;
        if (msg.type === 'State' || msg.type === 'SendAck') {
          latestRevisionRef.current = updateRevision(latestRevisionRef.current, msg.data.revision);
          setRevision((current) => updateRevision(current, msg.data.revision));
          if (msg.type === 'SendAck') {
            setSnapshot((current) => {
              if (!current || msg.data.revision < current.revision) return current;
              return {
                ...current,
                revision: msg.data.revision,
                mixes: current.mixes.map((mix) => mix.index !== msg.data.mix_index ? mix : {
                  ...mix,
                  sends: mix.sends.map((send) => send.channel_index !== msg.data.channel_index ? send : {
                    ...send,
                    gain_db: msg.data.gain_db,
                    pan: msg.data.pan,
                    muted: msg.data.muted,
                  }),
                }),
              };
            });
          }
        } else if (msg.type === 'Error') {
          if (msg.data.code === 'TOKEN_EXPIRED') {
            setStatus('disconnected');
            setError('Session expired. Please log in again.');
          } else {
            setError(`Server error: ${msg.data.code} — ${msg.data.message}`);
          }
        }
      } catch {
        setError('Malformed server message');
      }
    };

    ws.onerror = () => {
      if (!mountedRef.current || wsRef.current !== ws) return;
      setStatus('error');
      setError('WebSocket connection error');
    };

    ws.onclose = () => {
      if (!mountedRef.current || wsRef.current !== ws) return;
      setStatus('disconnected');
    };

    return () => {
      ws.close();
    };
  }, [token, disconnect]);

  return { status, revision, snapshot, error, send, disconnect };
}
