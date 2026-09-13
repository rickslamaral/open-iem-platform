import { beforeEach, describe, expect, it, vi } from 'vitest';
import { act, renderHook } from '@testing-library/react';
import { useEngineerWs } from './useEngineerWs';
import { PROTOCOL_VERSION } from './protocol';

// --- WebSocket mock ---

interface MockWebSocket {
  url: string;
  protocols: string | string[];
  readyState: number;
  send: ReturnType<typeof vi.fn>;
  close: ReturnType<typeof vi.fn>;
  onopen: ((event: Event) => void) | null;
  onmessage: ((event: { data?: string }) => void) | null;
  onerror: ((event: Event) => void) | null;
  onclose: ((event: CloseEvent) => void) | null;
}

let mockWsInstance: MockWebSocket | null = null;

class FakeWebSocket implements MockWebSocket {
  static OPEN = 1;
  static CONNECTING = 0;
  url: string;
  protocols: string | string[];
  readyState = FakeWebSocket.OPEN;
  send = vi.fn();
  close = vi.fn().mockImplementation(() => {
    this.readyState = 3;
    if (this.onclose) this.onclose(new CloseEvent('close'));
  });
  onopen: ((event: Event) => void) | null = null;
  onmessage: ((event: { data?: string }) => void) | null = null;
  onerror: ((event: Event) => void) | null = null;
  onclose: ((event: CloseEvent) => void) | null = null;

  constructor(url: string, protocols?: string | string[]) {
    this.url = url;
    this.protocols = protocols ?? '';
    mockWsInstance = this;
  }
}

function envelope(payload: unknown) {
  return JSON.stringify({ version: PROTOCOL_VERSION, request_id: 'srv', payload });
}

beforeEach(() => {
  mockWsInstance = null;
  vi.stubGlobal('WebSocket', FakeWebSocket);
  vi.stubGlobal('crypto', { randomUUID: () => 'test-uuid' });
});

describe('useEngineerWs', () => {
  it('stays disconnected when token is null', () => {
    const { result } = renderHook(() => useEngineerWs(null));
    expect(result.current.status).toBe('disconnected');
    expect(mockWsInstance).toBeNull();
  });

  it('transitions to connected and sends GetState on open', () => {
    const { result } = renderHook(() => useEngineerWs('tok'));
    act(() => { mockWsInstance!.onopen?.({} as Event); });
    expect(result.current.status).toBe('connected');
    const sent = JSON.parse(mockWsInstance!.send.mock.calls[0][0] as string) as {
      payload: { type: string };
    };
    expect(sent.payload.type).toBe('GetState');
  });

  it('applies MasterAck for mix 0', () => {
    const { result } = renderHook(() => useEngineerWs('tok'));
    act(() => { mockWsInstance!.onopen?.({} as Event); });
    act(() => {
      mockWsInstance!.onmessage?.({
        data: envelope({ type: 'MasterAck', data: { mix_index: 0, master_gain_db: -6, master_muted: false, revision: 3 } }),
      });
    });
    expect(result.current.mixes[0].master_gain_db).toBe(-6);
    expect(result.current.mixes[0].master_muted).toBe(false);
    expect(result.current.revision).toBe(3);
  });

  it('applies MasterAck for mix 1', () => {
    const { result } = renderHook(() => useEngineerWs('tok'));
    act(() => { mockWsInstance!.onopen?.({} as Event); });
    act(() => {
      mockWsInstance!.onmessage?.({
        data: envelope({ type: 'MasterAck', data: { mix_index: 1, master_gain_db: -3, master_muted: true, revision: 7 } }),
      });
    });
    expect(result.current.mixes[1].master_gain_db).toBe(-3);
    expect(result.current.mixes[1].master_muted).toBe(true);
  });

  it('ignores MasterAck for out-of-range mix_index', () => {
    const { result } = renderHook(() => useEngineerWs('tok'));
    act(() => { mockWsInstance!.onopen?.({} as Event); });
    act(() => {
      mockWsInstance!.onmessage?.({
        data: envelope({ type: 'MasterAck', data: { mix_index: 99, master_gain_db: -10, master_muted: false, revision: 1 } }),
      });
    });
    expect(result.current.revision).toBeNull();
  });

  it('updates revision on State message', () => {
    const { result } = renderHook(() => useEngineerWs('tok'));
    act(() => { mockWsInstance!.onopen?.({} as Event); });
    act(() => {
      mockWsInstance!.onmessage?.({ data: envelope({ type: 'State', data: { revision: 42 } }) });
    });
    expect(result.current.revision).toBe(42);
  });

  it('sets error on Error message', () => {
    const { result } = renderHook(() => useEngineerWs('tok'));
    act(() => { mockWsInstance!.onopen?.({} as Event); });
    act(() => {
      mockWsInstance!.onmessage?.({ data: envelope({ type: 'Error', data: { code: 'FORBIDDEN', message: 'role cannot perform this action' } }) });
    });
    expect(result.current.error).toBe('role cannot perform this action');
  });

  it('sets error on WebSocket error event', () => {
    const { result } = renderHook(() => useEngineerWs('tok'));
    act(() => { mockWsInstance!.onerror?.({} as Event); });
    expect(result.current.status).toBe('error');
    expect(result.current.error).toBe('Conexão WebSocket falhou.');
  });

  it('sends SetMasterGain with correct envelope', () => {
    const { result } = renderHook(() => useEngineerWs('tok'));
    act(() => { mockWsInstance!.onopen?.({} as Event); });
    act(() => { result.current.setMasterGain(0, -12); });
    const calls = mockWsInstance!.send.mock.calls;
    const lastSent = JSON.parse(calls[calls.length - 1][0] as string) as {
      payload: { type: string; data: { mix_index: number; gain_db: number } };
    };
    expect(lastSent.payload.type).toBe('SetMasterGain');
    expect(lastSent.payload.data.mix_index).toBe(0);
    expect(lastSent.payload.data.gain_db).toBe(-12);
  });

  it('sends SetMasterMute with correct envelope', () => {
    const { result } = renderHook(() => useEngineerWs('tok'));
    act(() => { mockWsInstance!.onopen?.({} as Event); });
    act(() => { result.current.setMasterMute(1, true); });
    const calls = mockWsInstance!.send.mock.calls;
    const lastSent = JSON.parse(calls[calls.length - 1][0] as string) as {
      payload: { type: string; data: { mix_index: number; muted: boolean } };
    };
    expect(lastSent.payload.type).toBe('SetMasterMute');
    expect(lastSent.payload.data.mix_index).toBe(1);
    expect(lastSent.payload.data.muted).toBe(true);
  });

  it('ignores send when WS is not open', () => {
    const { result } = renderHook(() => useEngineerWs('tok'));
    // Do not fire onopen — readyState stays OPEN in mock but wsRef not yet set on creation
    // Simulate closed state
    act(() => {
      if (mockWsInstance) mockWsInstance.readyState = 3;
    });
    act(() => { result.current.setMasterGain(0, 0); });
    // GetState is NOT sent because onopen was never called; no extra sends
    expect(mockWsInstance!.send).not.toHaveBeenCalled();
  });

  it('ignores malformed JSON message', () => {
    const { result } = renderHook(() => useEngineerWs('tok'));
    act(() => { mockWsInstance!.onopen?.({} as Event); });
    act(() => { mockWsInstance!.onmessage?.({ data: 'not-json{{{' }); });
    expect(result.current.revision).toBeNull();
  });

  it('closes WS when token becomes null', () => {
    const { rerender } = renderHook(({ tok }) => useEngineerWs(tok), { initialProps: { tok: 'tok' as string | null } });
    act(() => { mockWsInstance!.onopen?.({} as Event); });
    rerender({ tok: null });
    expect(mockWsInstance!.close).toHaveBeenCalled();
  });
});
