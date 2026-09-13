import { beforeEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen, waitFor, act } from '@testing-library/react';
import App from './App';
import { PROTOCOL_VERSION } from './protocol';

const json = (body: unknown, status = 200) => Promise.resolve({ ok: status < 400, status, text: () => Promise.resolve(JSON.stringify(body)), json: () => Promise.resolve(body) });

// --- Minimal WebSocket mock for App tests ---
type WsHandler = (event: { data?: string } | CloseEvent | Event) => void;

interface MockWs {
  send: ReturnType<typeof vi.fn>;
  close: ReturnType<typeof vi.fn>;
  onopen: WsHandler | null;
  onmessage: WsHandler | null;
  onerror: WsHandler | null;
  onclose: WsHandler | null;
  readyState: number;
}

let lastWs: MockWs | null = null;

class FakeWebSocket implements MockWs {
  static OPEN = 1;
  send = vi.fn();
  close = vi.fn();
  onopen: WsHandler | null = null;
  onmessage: WsHandler | null = null;
  onerror: WsHandler | null = null;
  onclose: WsHandler | null = null;
  readyState = FakeWebSocket.OPEN;
  constructor() { lastWs = this; }
}

function serverEnvelope(payload: unknown) {
  return JSON.stringify({ version: PROTOCOL_VERSION, request_id: 'srv', payload });
}

function dashboardFetch() {
  return vi.fn()
    .mockReturnValueOnce(json({ access_token: 'test-token' }))
    .mockReturnValueOnce(json({ sessions: [] }))
    .mockReturnValueOnce(json([{ mix_index: 0, user_id: 7, username: 'cantor' }]))
    .mockReturnValueOnce(json({ revision: 12 }))
    .mockReturnValueOnce(json({ availability: 'simulated', backend: 'simulated', sample_rate_hz: null, frames_processed: null, xrun_count: null }));
}

beforeEach(() => {
  lastWs = null;
  vi.restoreAllMocks();
  vi.stubGlobal('WebSocket', FakeWebSocket);
  vi.stubGlobal('crypto', { randomUUID: () => 'test-uuid' });
  vi.stubGlobal('fetch', dashboardFetch());
});

describe('Engineer Console', () => {
  it('autentica e mostra estado operacional sem prometer áudio real', async () => {
    render(<App />);
    fireEvent.change(screen.getByLabelText('Usuário'), { target: { value: 'engineer' } });
    fireEvent.change(screen.getByLabelText('Senha'), { target: { value: 'password' } });
    fireEvent.click(screen.getByRole('button', { name: 'Entrar' }));
    expect(await screen.findByRole('heading', { name: 'Engineer Console' })).toBeTruthy();
    expect(screen.getByText('Áudio SIMULATED')).toBeTruthy();
    expect(await screen.findByText('cantor (ID 7)')).toBeTruthy();
    expect(screen.getByText('XRUNs')).toBeTruthy();
    expect(screen.getByText('UNKNOWN')).toBeTruthy();
  });

  it('exibe falha de API ao carregar dashboard', async () => {
    vi.stubGlobal('fetch', vi.fn()
      .mockReturnValueOnce(json({ access_token: 'test-token' }))
      .mockReturnValueOnce(json({}, 503))
      .mockReturnValueOnce(json([]))
      .mockReturnValueOnce(json({ revision: 0 }))
      .mockReturnValueOnce(json({ availability: 'simulated', backend: 'simulated', sample_rate_hz: null, frames_processed: null, xrun_count: null })));
    render(<App />);
    fireEvent.change(screen.getByLabelText('Usuário'), { target: { value: 'engineer' } });
    fireEvent.change(screen.getByLabelText('Senha'), { target: { value: 'password' } });
    fireEvent.click(screen.getByRole('button', { name: 'Entrar' }));
    await waitFor(() => expect(screen.getByRole('alert')).toHaveTextContent('503'));
  });

  it('mostra badge WS e controles de master após autenticação', async () => {
    render(<App />);
    fireEvent.change(screen.getByLabelText('Usuário'), { target: { value: 'engineer' } });
    fireEvent.change(screen.getByLabelText('Senha'), { target: { value: 'password' } });
    fireEvent.click(screen.getByRole('button', { name: 'Entrar' }));
    await screen.findByRole('heading', { name: 'Engineer Console' });
    // WebSocket badge
    expect(screen.getByRole('status')).toBeTruthy();
    // Master control sliders for both mixes
    expect(screen.getByLabelText('Master gain mix 1')).toBeTruthy();
    expect(screen.getByLabelText('Master gain mix 2')).toBeTruthy();
    // Master mute buttons
    expect(screen.getByLabelText('Master mute mix 1')).toBeTruthy();
    expect(screen.getByLabelText('Master mute mix 2')).toBeTruthy();
  });

  it('atualiza gain do mix 0 ao receber MasterAck via WebSocket', async () => {
    render(<App />);
    fireEvent.change(screen.getByLabelText('Usuário'), { target: { value: 'engineer' } });
    fireEvent.change(screen.getByLabelText('Senha'), { target: { value: 'password' } });
    fireEvent.click(screen.getByRole('button', { name: 'Entrar' }));
    await screen.findByRole('heading', { name: 'Engineer Console' });
    act(() => { lastWs!.onopen?.({} as Event); });
    act(() => {
      lastWs!.onmessage?.({
        data: serverEnvelope({ type: 'MasterAck', data: { mix_index: 0, master_gain_db: -9, master_muted: false, revision: 5 } }),
      });
    });
    const slider = screen.getByLabelText('Master gain mix 1') as HTMLInputElement;
    expect(Number(slider.value)).toBe(-9);
  });

  it('exibe mute ativo ao receber MasterAck com master_muted true', async () => {
    render(<App />);
    fireEvent.change(screen.getByLabelText('Usuário'), { target: { value: 'engineer' } });
    fireEvent.change(screen.getByLabelText('Senha'), { target: { value: 'password' } });
    fireEvent.click(screen.getByRole('button', { name: 'Entrar' }));
    await screen.findByRole('heading', { name: 'Engineer Console' });
    act(() => { lastWs!.onopen?.({} as Event); });
    act(() => {
      lastWs!.onmessage?.({
        data: serverEnvelope({ type: 'MasterAck', data: { mix_index: 1, master_gain_db: 0, master_muted: true, revision: 8 } }),
      });
    });
    const muteBtn = screen.getByLabelText('Master mute mix 2');
    expect(muteBtn).toHaveAttribute('aria-pressed', 'true');
    expect(muteBtn).toHaveTextContent('MUTED');
  });
});
