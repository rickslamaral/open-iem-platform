import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useWebSocket } from './useWebSocket';

// Minimal WebSocket mock
class MockWebSocket {
  static OPEN = 1;
  readyState = MockWebSocket.OPEN;
  onopen: (() => void) | null = null;
  onmessage: ((e: { data: string }) => void) | null = null;
  onerror: (() => void) | null = null;
  onclose: (() => void) | null = null;
  sent: string[] = [];

  constructor(public url: string) {
    MockWebSocket.instances.push(this);
  }

  send(data: string) {
    this.sent.push(data);
  }

  close() {
    this.onclose?.();
  }

  static instances: MockWebSocket[] = [];
  static reset() {
    MockWebSocket.instances = [];
  }
}

beforeEach(() => {
  MockWebSocket.reset();
  vi.stubGlobal('WebSocket', MockWebSocket);
});

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('useWebSocket', () => {
  it('stays disconnected when token is null', () => {
    const { result } = renderHook(() => useWebSocket(null));
    expect(result.current.status).toBe('disconnected');
  });

  it('transitions to connecting when token provided', () => {
    const { result } = renderHook(() => useWebSocket('test-token'));
    expect(result.current.status).toBe('connecting');
  });

  it('transitions to connected on open event', () => {
    const { result } = renderHook(() => useWebSocket('test-token'));
    act(() => {
      MockWebSocket.instances[0]?.onopen?.();
    });
    expect(result.current.status).toBe('connected');
  });

  it('updates revision on State message', () => {
    const { result } = renderHook(() => useWebSocket('test-token'));
    act(() => {
      MockWebSocket.instances[0]?.onopen?.();
    });
    act(() => {
      MockWebSocket.instances[0]?.onmessage?.({
        data: JSON.stringify({ payload: { type: 'State', data: { revision: 42 } } }),
      });
    });
    expect(result.current.revision).toBe(42);
  });

  it('sets error on TOKEN_EXPIRED', () => {
    const { result } = renderHook(() => useWebSocket('test-token'));
    act(() => {
      MockWebSocket.instances[0]?.onopen?.();
    });
    act(() => {
      MockWebSocket.instances[0]?.onmessage?.({
        data: JSON.stringify({
          payload: { type: 'Error', data: { code: 'TOKEN_EXPIRED', message: 'expired' } },
        }),
      });
    });
    expect(result.current.status).toBe('disconnected');
    expect(result.current.error).toMatch(/expired/i);
  });

  it('transitions to error on ws error', () => {
    const { result } = renderHook(() => useWebSocket('test-token'));
    act(() => {
      MockWebSocket.instances[0]?.onerror?.();
    });
    expect(result.current.status).toBe('error');
  });
});
