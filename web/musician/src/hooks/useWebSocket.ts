import { useEffect, useRef, useState, useCallback } from 'react';
import type { ServerMessage, StateSnapshot } from '../protocol/types';
import { encodeEnvelope } from '../protocol/envelope';
import type { ClientMessage } from '../protocol/types';
import { PROTOCOL_VERSION } from '../protocol/types';

const GAIN_DB_MIN = -144;
const GAIN_DB_MAX = 12;
const MAX_REQUEST_ID_BYTES = 128;

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
  const channelIndexes = new Set<number>();
  if (snapshot.channels.some((channel) => !channel || !Number.isInteger(channel.index)
    || channel.index < 0 || channel.index >= 8 || channelIndexes.has(channel.index)
    || !channelIndexes.add(channel.index) || !isFiniteNumber(channel.gain_db)
    || channel.gain_db < GAIN_DB_MIN || channel.gain_db > GAIN_DB_MAX
    || typeof channel.muted !== 'boolean')) return false;
  const mixIndexes = new Set<number>();
  return snapshot.mixes.every((mix) => mix && Number.isInteger(mix.index)
    && mix.index >= 0 && mix.index < 2 && !mixIndexes.has(mix.index)
    && mixIndexes.add(mix.index)
    && isFiniteNumber(mix.master_gain_db)
    && mix.master_gain_db >= GAIN_DB_MIN && mix.master_gain_db <= GAIN_DB_MAX
    && typeof mix.master_muted === 'boolean'
    && Array.isArray(mix.sends) && (() => {
      const sendIndexes = new Set<number>();
      return mix.sends.every((send) => send
        && Number.isInteger(send.channel_index) && send.channel_index >= 0
        && send.channel_index < 8 && !sendIndexes.has(send.channel_index)
        && sendIndexes.add(send.channel_index) && isFiniteNumber(send.gain_db)
        && isFiniteNumber(send.pan) && send.pan >= -1 && send.pan <= 1
        && typeof send.muted === 'boolean');
    })());
}

function isServerMessage(value: unknown): value is ServerMessage {
  if (!value || typeof value !== 'object') return false;
  const message = value as { type?: unknown; data?: Record<string, unknown> };
  if (message.type === 'State') {
    return isValidRevision(message.data?.revision);
  }
  if (message.type === 'SendAck') {
    return isValidRevision(message.data?.revision)
      && typeof message.data?.mix_index === 'number'
      && Number.isInteger(message.data.mix_index)
      && typeof message.data?.channel_index === 'number'
      && Number.isInteger(message.data.channel_index)
      && message.data.mix_index >= 0 && message.data.mix_index < 2
      && message.data.channel_index >= 0 && message.data.channel_index < 8
      && typeof message.data?.gain_db === 'number'
      && Number.isFinite(message.data.gain_db)
      && message.data.gain_db >= GAIN_DB_MIN && message.data.gain_db <= GAIN_DB_MAX
      && typeof message.data?.pan === 'number'
      && Number.isFinite(message.data.pan)
      && message.data.pan >= -1 && message.data.pan <= 1
      && typeof message.data?.muted === 'boolean';
  }
  if (message.type === 'MasterAck') {
    return isValidRevision(message.data?.revision)
      && typeof message.data?.mix_index === 'number'
      && Number.isInteger(message.data.mix_index)
      && message.data.mix_index >= 0 && message.data.mix_index < 2
      && isFiniteNumber(message.data?.master_gain_db)
      && message.data.master_gain_db >= GAIN_DB_MIN
      && message.data.master_gain_db <= GAIN_DB_MAX
      && typeof message.data?.master_muted === 'boolean';
  }
  if (message.type === 'Error') {
    return typeof message.data?.code === 'string' && typeof message.data.message === 'string';
  }
  return false;
}

/**
 * WebSocket hook for Open IEM control plane.
 *
 * Token is sent in a negotiated WebSocket subprotocol because browser WebSocket
 * API does not allow custom headers. Server authenticates `openiem.bearer.<JWT>`
 * and echoes only `openiem.v1` as selected protocol during upgrade.
 */
export function useWebSocket(token: string | null): UseWebSocketResult {
  const [status, setStatus] = useState<WsStatus>('disconnected');
  const [revision, setRevision] = useState<number | null>(null);
  const [snapshot, setSnapshot] = useState<StateSnapshot | null>(null);
  const [error, setError] = useState<string | null>(null);
  const wsRef = useRef<WebSocket | null>(null);
  const snapshotAbortRef = useRef<AbortController | null>(null);
  const latestRevisionRef = useRef<number | null>(null);
  const mountedRef = useRef(true);

  const disconnect = useCallback(() => {
    snapshotAbortRef.current?.abort();
    snapshotAbortRef.current = null;
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

    const url = '/ws/v1';
    const protocols = [`openiem.bearer.${token}`, 'openiem.v1'];
    setStatus('connecting');
    setRevision(null);
    latestRevisionRef.current = null;
    setSnapshot(null);

    const ws = new WebSocket(url, protocols);
    wsRef.current = ws;

    const abortController = new AbortController();
    snapshotAbortRef.current = abortController;
    let snapshotRequestInFlight = false;
    let snapshotRefreshQueued = false;
    const refreshSnapshot = async () => {
      if (snapshotRequestInFlight) {
        snapshotRefreshQueued = true;
        return;
      }
      snapshotRequestInFlight = true;
      try {
        const response = await fetch('/api/v1/state', {
          headers: { Authorization: `Bearer ${token}` },
          signal: abortController.signal,
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
        if (snapshotRefreshQueued && !abortController.signal.aborted) {
          snapshotRefreshQueued = false;
          void refreshSnapshot();
        }
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
        const envelope = parsed && typeof parsed === 'object' ? parsed as {
          version?: unknown;
          request_id?: unknown;
          payload?: unknown;
        } : null;
        const payload = envelope?.payload;
        if (envelope?.version !== PROTOCOL_VERSION
          || typeof envelope.request_id !== 'string'
          || envelope.request_id.length === 0
          || envelope.request_id.length > MAX_REQUEST_ID_BYTES
          || !isServerMessage(payload)) {
          setError('Malformed server message');
          return;
        }
        const msg = payload;
        if (msg.type === 'State' || msg.type === 'SendAck' || msg.type === 'MasterAck') {
          if (msg.type === 'State') {
            latestRevisionRef.current = updateRevision(latestRevisionRef.current, msg.data.revision);
            void refreshSnapshot();
          }
          setRevision((current) => updateRevision(current, msg.data.revision));
          if (msg.type === 'MasterAck') {
            setSnapshot((current) => {
              if (!current || msg.data.revision < current.revision) return current;
              const mix = current.mixes.find((item) => item.index === msg.data.mix_index);
              if (!mix) return current;
              latestRevisionRef.current = updateRevision(latestRevisionRef.current, msg.data.revision);
              return {
                ...current,
                revision: Math.max(current.revision, msg.data.revision),
                mixes: current.mixes.map((item) => item.index !== msg.data.mix_index ? item : {
                  ...item,
                  master_gain_db: msg.data.master_gain_db,
                  master_muted: msg.data.master_muted,
                }),
              };
            });
          } else if (msg.type === 'SendAck') {
            setSnapshot((current) => {
              if (!current || msg.data.revision < current.revision) return current;
              const mix = current.mixes.find((item) => item.index === msg.data.mix_index);
              const send = mix?.sends.find((item) => item.channel_index === msg.data.channel_index);
              if (!mix || !send) return current;
              latestRevisionRef.current = updateRevision(latestRevisionRef.current, msg.data.revision);
              return {
                ...current,
                revision: Math.max(current.revision, msg.data.revision),
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
      abortController.abort();
      if (snapshotAbortRef.current === abortController) snapshotAbortRef.current = null;
      ws.close();
      if (wsRef.current === ws) wsRef.current = null;
    };
  }, [token, disconnect]);

  return { status, revision, snapshot, error, send, disconnect };
}
