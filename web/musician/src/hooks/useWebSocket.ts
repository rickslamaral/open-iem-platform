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

    const ws = new WebSocket(url);
    wsRef.current = ws;

    ws.onopen = () => {
      if (!mountedRef.current) return;
      setStatus('connected');
      setError(null);
      // Request initial state
      ws.send(encodeEnvelope({ type: 'GetState' }));
    };

    ws.onmessage = (event: MessageEvent<string>) => {
      if (!mountedRef.current) return;
      try {
        const parsed = JSON.parse(event.data) as { payload: ServerMessage };
        const msg = parsed.payload;
        if (msg.type === 'State') {
          setRevision(msg.data.revision);
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
      if (!mountedRef.current) return;
      setStatus('error');
      setError('WebSocket connection error');
    };

    ws.onclose = () => {
      if (!mountedRef.current) return;
      setStatus('disconnected');
    };

    return () => {
      ws.close();
    };
  }, [token, disconnect]);

  return { status, revision, error, send, disconnect };
}
