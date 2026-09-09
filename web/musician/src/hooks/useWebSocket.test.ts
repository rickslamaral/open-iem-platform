import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, act, waitFor } from '@testing-library/react';
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

const makeSnapshot = (revision = 5) => ({
  schema_version: 1 as const,
  revision,
  channels: [{ index: 0, gain_db: 0, muted: false }],
  mixes: [{
    index: 0,
    master_gain_db: 0,
    master_muted: false,
    sends: [{ channel_index: 1, gain_db: -3, pan: 0, muted: false }],
  }],
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

  it('fetches authenticated REST snapshot after WebSocket open', async () => {
    const fetchMock = vi.fn().mockResolvedValue({ ok: true, json: async () => makeSnapshot(5) });
    vi.stubGlobal('fetch', fetchMock);
    const { result } = renderHook(() => useWebSocket('test-token'));

    act(() => {
      MockWebSocket.instances[0]?.onopen?.();
    });
    await waitFor(() => expect(result.current.snapshot?.revision).toBe(5));

    expect(fetchMock).toHaveBeenCalledWith('/api/v1/state', expect.objectContaining({
      headers: { Authorization: 'Bearer test-token' },
      signal: expect.any(AbortSignal),
    }));
    expect(MockWebSocket.instances[0]?.sent).toHaveLength(1);
  });

  it('rejects snapshot with invalid nested send', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ ...makeSnapshot(), mixes: [{ ...makeSnapshot().mixes[0], sends: [{
        channel_index: 1, gain_db: -3, pan: 2, muted: false,
      }] }] }),
    }));
    const { result } = renderHook(() => useWebSocket('test-token'));

    act(() => {
      MockWebSocket.instances[0]?.onopen?.();
    });
    await waitFor(() => expect(result.current.error).toBe('Malformed state snapshot'));
    expect(result.current.snapshot).toBeNull();
  });

  it('does not let delayed snapshot from old WebSocket overwrite newer ACK state', async () => {
    let resolveOld!: (value: unknown) => void;
    const fetchMock = vi.fn()
      .mockReturnValueOnce(new Promise((resolve) => { resolveOld = resolve; }))
      .mockResolvedValueOnce({ ok: true, json: async () => makeSnapshot(10) });
    vi.stubGlobal('fetch', fetchMock);
    const { result, rerender } = renderHook(({ token }) => useWebSocket(token), {
      initialProps: { token: 'old-token' },
    });

    act(() => MockWebSocket.instances[0]?.onopen?.());
    rerender({ token: 'new-token' });
    act(() => MockWebSocket.instances[1]?.onopen?.());
    await waitFor(() => expect(fetchMock).toHaveBeenCalledTimes(2));
    await waitFor(() => expect(result.current.snapshot?.revision).toBe(10));
    act(() => MockWebSocket.instances[1]?.onmessage?.({ data: JSON.stringify({ version: 1, request_id: '00000000-0000-4000-8000-000000000001', payload: {
      type: 'SendAck', data: { mix_index: 0, channel_index: 1, gain_db: 4, pan: 0.5, muted: true, revision: 11 },
    } }) }));
    await act(async () => resolveOld(makeSnapshot(9)));

    expect(result.current.snapshot).toMatchObject({
      revision: 11,
      mixes: [{ sends: [{ channel_index: 1, gain_db: 4, pan: 0.5, muted: true }] }],
    });
    expect(result.current.revision).toBe(11);
  });

  it('applies SendAck values to matching snapshot send', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: true, json: async () => makeSnapshot(5) }));
    const { result } = renderHook(() => useWebSocket('test-token'));
    act(() => MockWebSocket.instances[0]?.onopen?.());
    await waitFor(() => expect(result.current.snapshot).not.toBeNull());

    act(() => MockWebSocket.instances[0]?.onmessage?.({ data: JSON.stringify({ version: 1, request_id: '00000000-0000-4000-8000-000000000001', payload: {
      type: 'SendAck', data: { mix_index: 0, channel_index: 1, gain_db: 6, pan: -0.25, muted: true, revision: 6 },
    } }) }));

    expect(result.current.snapshot?.revision).toBe(6);
    expect(result.current.snapshot?.mixes[0]?.sends[0]).toMatchObject({
      gain_db: 6, pan: -0.25, muted: true,
    });
  });

  it('updates revision on State message', () => {
    const { result } = renderHook(() => useWebSocket('test-token'));
    act(() => {
      MockWebSocket.instances[0]?.onopen?.();
    });
    act(() => {
      MockWebSocket.instances[0]?.onmessage?.({
        data: JSON.stringify({ version: 1, request_id: '00000000-0000-4000-8000-000000000001', payload: { type: 'State', data: { revision: 42 } } }),
      });
    });
    expect(result.current.revision).toBe(42);
  });

  it('updates revision from SendAck and ignores stale messages', () => {
    const { result } = renderHook(() => useWebSocket('test-token'));
    act(() => {
      MockWebSocket.instances[0]?.onmessage?.({
        data: JSON.stringify({ version: 1, request_id: '00000000-0000-4000-8000-000000000001', payload: { type: 'SendAck', data: {
          mix_index: 0, channel_index: 1, gain_db: -3, pan: 0, muted: false, revision: 8,
        } } }),
      });
    });
    expect(result.current.revision).toBe(8);
    act(() => {
      MockWebSocket.instances[0]?.onmessage?.({
        data: JSON.stringify({ version: 1, request_id: '00000000-0000-4000-8000-000000000001', payload: { type: 'State', data: { revision: 7 } } }),
      });
    });
    expect(result.current.revision).toBe(8);
  });

  it('rejects malformed server revisions', () => {
    const { result } = renderHook(() => useWebSocket('test-token'));
    act(() => {
      MockWebSocket.instances[0]?.onmessage?.({
        data: JSON.stringify({ version: 1, request_id: '00000000-0000-4000-8000-000000000001', payload: { type: 'SendAck', data: { revision: '8' } } }),
      });
    });
    expect(result.current.revision).toBeNull();
    expect(result.current.error).toBe('Malformed server message');
  });

  it('sets error on TOKEN_EXPIRED', () => {
    const { result } = renderHook(() => useWebSocket('test-token'));
    act(() => {
      MockWebSocket.instances[0]?.onopen?.();
    });
    act(() => {
      MockWebSocket.instances[0]?.onmessage?.({
        data: JSON.stringify({ version: 1, request_id: '00000000-0000-4000-8000-000000000001', payload: {
          type: 'Error', data: { code: 'TOKEN_EXPIRED', message: 'expired' },
        } }),
      });
    });
    expect(result.current.status).toBe('disconnected');
    expect(result.current.error).toMatch(/expired/i);
  });

  it('sets disconnected on manual disconnect', () => {
    const { result } = renderHook(() => useWebSocket('test-token'));
    act(() => result.current.disconnect());
    expect(result.current.status).toBe('disconnected');
  });

  it('transitions to error on ws error', () => {
    const { result } = renderHook(() => useWebSocket('test-token'));
    act(() => {
      MockWebSocket.instances[0]?.onerror?.();
    });
    expect(result.current.status).toBe('error');
  });
});
