import { useEffect, useRef, useState, useCallback } from 'react';
import type { ServerMessage } from '../protocol/types';
import { encodeEnvelope } from '../protocol/envelope';
import type { ClientMessage } from '../protocol/types';

export type WsStatus = 'disconnected' | 'connecting' | 'connected' | 'error';

export interface UseWebSocketResult {
  status: WsStatus;
  revision: number | null;
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
  const [error, setError] = useState<string | null>(null);
  const wsRef = useRef<WebSocket | null>(null);
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

    const ws = new WebSocket(url);
    wsRef.current = ws;

    ws.onopen = () => {
      if (!mountedRef.current || wsRef.current !== ws) return;
      setStatus('connected');
      setError(null);
      // Request initial state
      ws.send(encodeEnvelope({ type: 'GetState' }));
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
          setRevision((current) => updateRevision(current, msg.data.revision));
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

  return { status, revision, error, send, disconnect };
}
