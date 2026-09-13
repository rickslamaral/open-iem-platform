import { beforeEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen, waitFor, act } from '@testing-library/react';
import App from './App';
import { PROTOCOL_VERSION } from './protocol';

// Helper: build a minimal ChannelState object.
const makeChannel = (index: number, overrides: Record<string, unknown> = {}) => ({
  index,
  id: index + 1,
  name: `Vocals ${index + 1}`,
  gain_db: 0.0,
  muted: false,
  locked: false,
  enabled: true,
  revision: 1,
  ...overrides,
});

const json = (body: unknown, status = 200) =>
  Promise.resolve({
    ok: status < 400,
    status,
    text: () => Promise.resolve(JSON.stringify(body)),
    json: () => Promise.resolve(body),
  });

// Default mock sequence: login → dashboard (sessions/mixes/state/telemetry).
function stubDashboard(channels = [makeChannel(0), makeChannel(1)]) {
  vi.stubGlobal(
    'fetch',
    vi
      .fn()
      .mockReturnValueOnce(json({ access_token: 'test-token' })) // login
      .mockReturnValueOnce(json({ sessions: [] })) // /audio/sessions
      .mockReturnValueOnce(json([{ mix_index: 0, user_id: 7, username: 'cantor' }])) // /mixes
      .mockReturnValueOnce(json({ revision: 12, channels })) // /state
      .mockReturnValueOnce(
        json({
          availability: 'simulated',
          backend: 'simulated',
          sample_rate_hz: null,
          frames_processed: null,
          xrun_count: null,
        }),
      ), // /telemetry
  );
}

async function loginAndLoad(channels = [makeChannel(0), makeChannel(1)]) {
  stubDashboard(channels);
  render(<App />);
  fireEvent.change(screen.getByLabelText('Usuário'), { target: { value: 'engineer' } });
  fireEvent.change(screen.getByLabelText('Senha'), { target: { value: 'password' } });
  fireEvent.click(screen.getByRole('button', { name: 'Entrar' }));
  // Wait for dashboard to appear.
  await screen.findByRole('heading', { name: 'Engineer Console' });
}

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
    await loginAndLoad();
    expect(screen.getByText('Áudio SIMULATED')).toBeTruthy();
    expect(await screen.findByText('cantor (ID 7)')).toBeTruthy();
    expect(screen.getByText('XRUNs')).toBeTruthy();
    expect(screen.getByText('UNKNOWN')).toBeTruthy();
  });

  it('exibe falha de API ao carregar dashboard', async () => {
    vi.stubGlobal(
      'fetch',
      vi
        .fn()
        .mockReturnValueOnce(json({ access_token: 'test-token' }))
        .mockReturnValueOnce(json({}, 503))
        .mockReturnValueOnce(json([]))
        .mockReturnValueOnce(json({ revision: 0 }))
        .mockReturnValueOnce(
          json({
            availability: 'simulated',
            backend: 'simulated',
            sample_rate_hz: null,
            frames_processed: null,
            xrun_count: null,
          }),
        ),
    );
    render(<App />);
    fireEvent.change(screen.getByLabelText('Usuário'), { target: { value: 'engineer' } });
    fireEvent.change(screen.getByLabelText('Senha'), { target: { value: 'password' } });
    fireEvent.click(screen.getByRole('button', { name: 'Entrar' }));
    await waitFor(() => expect(screen.getByRole('alert')).toHaveTextContent('503'));
  });

  // ── Master Controls (Phase 87) ─────────────────────────────────────────
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

  // ── Channel Strip (Phase 89) ───────────────────────────────────────────
  it('renderiza channel strips para todos os canais retornados pelo servidor', async () => {
    await loginAndLoad([makeChannel(0, { name: 'Guitarra' }), makeChannel(1, { name: 'Bateria' })]);
    await screen.findByText('Canais de entrada');
    expect(screen.getByLabelText(/Canal 1: Guitarra/)).toBeTruthy();
    expect(screen.getByLabelText(/Canal 2: Bateria/)).toBeTruthy();
  });

  it('não renderiza seção de canais quando servidor retorna lista vazia', async () => {
    await loginAndLoad([]);
    await screen.findByText('cantor (ID 7)');
    expect(screen.queryByText('Canais de entrada')).toBeNull();
  });

  it('exibe MUTED badge quando canal está mutado', async () => {
    await loginAndLoad([makeChannel(0, { muted: true }), makeChannel(1)]);
    await screen.findByText('Canais de entrada');
    expect(screen.getByText('MUTED')).toBeTruthy();
  });

  it('slider de gain reflete valor do servidor e tem o range correto', async () => {
    await loginAndLoad([makeChannel(0, { gain_db: -6.0 })]);
    await screen.findByText('Canais de entrada');
    const slider = screen.getByLabelText('Gain canal 1') as HTMLInputElement;
    expect(slider.value).toBe('-6');
    expect(slider.min).toBe('-144');
    expect(slider.max).toBe('12');
  });

  it('gain slider desabilitado quando canal bloqueado', async () => {
    await loginAndLoad([makeChannel(0, { locked: true })]);
    await screen.findByText('Canais de entrada');
    const slider = screen.getByLabelText('Gain canal 1') as HTMLInputElement;
    expect(slider.disabled).toBe(true);
  });

  it('botão mute toggle envia PUT /api/v1/channels/:index/mute', async () => {
    const fetchMock = vi
      .fn()
      .mockReturnValueOnce(json({ access_token: 'test-token' }))
      .mockReturnValueOnce(json({ sessions: [] }))
      .mockReturnValueOnce(json([]))
      .mockReturnValueOnce(json({ revision: 1, channels: [makeChannel(0, { muted: false })] }))
      .mockReturnValueOnce(
        json({ availability: 'simulated', backend: 'simulated', sample_rate_hz: null, frames_processed: null, xrun_count: null }),
      )
      .mockImplementation((path: string) =>
        path.includes('/channels/')
          ? json({ revision: 2 })
          : path.includes('/audio/sessions')
          ? json({ sessions: [] })
          : path.endsWith('/mixes')
          ? json([])
          : path.endsWith('/state')
          ? json({ revision: 2, channels: [makeChannel(0)] })
          : json({ availability: 'simulated', backend: 'simulated', sample_rate_hz: null, frames_processed: null, xrun_count: null }),
      );
    vi.stubGlobal('fetch', fetchMock);
    render(<App />);
    fireEvent.change(screen.getByLabelText('Usuário'), { target: { value: 'engineer' } });
    fireEvent.change(screen.getByLabelText('Senha'), { target: { value: 'password' } });
    fireEvent.click(screen.getByRole('button', { name: 'Entrar' }));
    await screen.findByText('Canais de entrada');
    const muteBtn = screen.getByRole('button', { name: 'Mutar canal 1' });
    fireEvent.click(muteBtn);
    await waitFor(() => {
      const calls = fetchMock.mock.calls;
      const mutateCall = calls.find(
        (c) => typeof c[0] === 'string' && (c[0] as string).includes('/channels/0/mute'),
      );
      expect(mutateCall).toBeDefined();
    });
  });

  it('gain slider envia PUT /api/v1/channels/:index/gain após debounce de 300 ms', async () => {
    const fetchMock = vi
      .fn()
      .mockReturnValueOnce(json({ access_token: 'test-token' }))
      .mockReturnValueOnce(json({ sessions: [] }))
      .mockReturnValueOnce(json([]))
      .mockReturnValueOnce(json({ revision: 1, channels: [makeChannel(0, { gain_db: 0 })] }))
      .mockReturnValueOnce(
        json({ availability: 'simulated', backend: 'simulated', sample_rate_hz: null, frames_processed: null, xrun_count: null }),
      )
      .mockImplementation((path: string) =>
        path.includes('/channels/')
          ? json({ revision: 2 })
          : path.includes('/audio/sessions')
          ? json({ sessions: [] })
          : path.endsWith('/mixes')
          ? json([])
          : path.endsWith('/state')
          ? json({ revision: 2, channels: [makeChannel(0, { gain_db: -12 })] })
          : json({ availability: 'simulated', backend: 'simulated', sample_rate_hz: null, frames_processed: null, xrun_count: null }),
      );
    vi.stubGlobal('fetch', fetchMock);
    render(<App />);
    fireEvent.change(screen.getByLabelText('Usuário'), { target: { value: 'engineer' } });
    fireEvent.change(screen.getByLabelText('Senha'), { target: { value: 'password' } });
    fireEvent.click(screen.getByRole('button', { name: 'Entrar' }));
    await screen.findByText('Canais de entrada');
    const slider = screen.getByLabelText('Gain canal 1');
    fireEvent.change(slider, { target: { value: '-12' } });
    // Wait up to 1s for the 300ms debounce to fire and the request to land.
    await waitFor(
      () => {
        const calls = fetchMock.mock.calls;
        const gainCall = calls.find(
          (c) => typeof c[0] === 'string' && (c[0] as string).includes('/channels/0/gain'),
        );
        expect(gainCall).toBeDefined();
      },
      { timeout: 1000 },
    );
  });

  it('label de gain mostra valor em dB com uma casa decimal', async () => {
    await loginAndLoad([makeChannel(0, { gain_db: 3.5 })]);
    await screen.findByText('Canais de entrada');
    expect(screen.getByText('3.5 dB')).toBeTruthy();
  });
});
