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
      ) // /telemetry
      .mockReturnValueOnce(json({ receiver: { packets_received: 0, packets_dropped: 0, late_packets: 0, reconnect_count: 0, plc_frames_total: 0, plc_consecutive_max: 0, output_failures: 0 } })) // /metrics
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

  it('exibe métricas do receiver com valores zero válidos', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn()
        .mockReturnValueOnce(json({ access_token: 'test-token' }))
        .mockReturnValueOnce(json({ sessions: [] }))
        .mockReturnValueOnce(json([]))
        .mockReturnValueOnce(json({ revision: 1, channels: [] }))
        .mockReturnValueOnce(json({ availability: 'simulated', backend: 'simulated', sample_rate_hz: null, frames_processed: null, xrun_count: null }))
        .mockReturnValueOnce(json({ receiver: { packets_received: 0, packets_dropped: 0, late_packets: 0, reconnect_count: 0, plc_frames_total: 0, plc_consecutive_max: 0, output_failures: 0 } })),
    );
    render(<App />);
    fireEvent.change(screen.getByLabelText('Usuário'), { target: { value: 'engineer' } });
    fireEvent.change(screen.getByLabelText('Senha'), { target: { value: 'password' } });
    fireEvent.click(screen.getByRole('button', { name: 'Entrar' }));
    await screen.findByRole('heading', { name: 'Receiver — métricas' });
    expect(screen.getByText('Pacotes recebidos')).toBeTruthy();
    expect(screen.getByText('Falhas de saída')).toBeTruthy();
    expect(screen.getAllByText('0').length).toBeGreaterThanOrEqual(7);
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

  // ── EQ Band Controls (Phase 93) ───────────────────────────────────────
  it('renderiza controles EQ: 4 bandas × 2 mixes visíveis após login', async () => {
    render(<App />);
    fireEvent.change(screen.getByLabelText('Usuário'), { target: { value: 'engineer' } });
    fireEvent.change(screen.getByLabelText('Senha'), { target: { value: 'password' } });
    fireEvent.click(screen.getByRole('button', { name: 'Entrar' }));
    await screen.findByRole('heading', { name: 'Engineer Console' });
    // 4 bands × 2 mixes = 8 EQ band containers
    for (let mix = 1; mix <= 2; mix++) {
      for (let band = 1; band <= 4; band++) {
        expect(screen.getByLabelText(`EQ Mix ${mix} Band ${band}`)).toBeTruthy();
      }
    }
  });

  it('envia SetEqBand ao alterar slider de gain do EQ', async () => {
    render(<App />);
    fireEvent.change(screen.getByLabelText('Usuário'), { target: { value: 'engineer' } });
    fireEvent.change(screen.getByLabelText('Senha'), { target: { value: 'password' } });
    fireEvent.click(screen.getByRole('button', { name: 'Entrar' }));
    await screen.findByRole('heading', { name: 'Engineer Console' });
    act(() => { lastWs!.onopen?.({} as Event); });
    // First enable the band
    const enableCheckbox = screen.getByLabelText('EQ band 1 mix 1 enabled');
    fireEvent.click(enableCheckbox);
    // Check that SetEqBand was sent
    await waitFor(() => {
      const calls = lastWs!.send.mock.calls;
      const eqCall = calls.find((c) => {
        const parsed = JSON.parse(c[0] as string) as { payload: { type: string } };
        return parsed.payload.type === 'SetEqBand';
      });
      expect(eqCall).toBeDefined();
    });
  });

  it('atualiza estado da banda ao receber EqBandAck via WebSocket', async () => {
    render(<App />);
    fireEvent.change(screen.getByLabelText('Usuário'), { target: { value: 'engineer' } });
    fireEvent.change(screen.getByLabelText('Senha'), { target: { value: 'password' } });
    fireEvent.click(screen.getByRole('button', { name: 'Entrar' }));
    await screen.findByRole('heading', { name: 'Engineer Console' });
    act(() => { lastWs!.onopen?.({} as Event); });
    act(() => {
      lastWs!.onmessage?.({
        data: serverEnvelope({
          type: 'EqBandAck',
          data: { mix_index: 0, band_index: 0, frequency_hz: 500, gain_db: 6, q: 2.0, enabled: true, revision: 10 },
        }),
      });
    });
    // After EqBandAck, frequency slider for mix 1 band 1 should reflect 500 Hz
    await waitFor(() => {
      const freqSlider = screen.getByLabelText('EQ band 1 mix 1 frequency') as HTMLInputElement;
      expect(freqSlider.value).toBe('500');
    });
    // enabled checkbox should be checked
    const enableCheckbox = screen.getByLabelText('EQ band 1 mix 1 enabled') as HTMLInputElement;
    expect(enableCheckbox.checked).toBe(true);
  });

  function sceneDashboardFetch(active: { id: string; name: string } | null = { id: 'scene-1', name: 'Show' }) {
    return vi.fn().mockImplementation((path: string, init?: RequestInit) => {
      if (path === '/api/v1/auth/login') return json({ access_token: 'test-token' });
      if (path === '/api/v1/audio/sessions') return json({ sessions: [] });
      if (path === '/api/v1/mixes') return json([]);
      if (path === '/api/v1/state') return json({ revision: 1, channels: [makeChannel(0)] });
      if (path === '/api/v1/telemetry') return json({ availability: 'simulated', backend: 'simulated', sample_rate_hz: null, frames_processed: null, xrun_count: null });
      if (path === '/api/v1/scenes' && init?.method === 'POST') return json({ id: 'new', name: 'Nova', revision: 1 });
      if (path === '/api/v1/scenes') return json({ scenes: [{ id: 'scene-1', name: 'Show', active_revision: 3, created_at: 1700000000, updated_at: 1700000000 }, { id: 'scene-2', name: 'Ensaio', active_revision: 2, created_at: 1700000000, updated_at: 1700000000 }] });
      if (path === '/api/v1/scenes/active') return json({ scene: active });
      if (path === '/api/v1/scenes/scene-2') return json({ id: 'scene-2', name: 'Ensaio', revision: 2, config: { channels: [], mixes: [] } });
      return json({}, 204);
    });
  }

  async function loginScenes(active: { id: string; name: string } | null = { id: 'scene-1', name: 'Show' }, fetchMock = sceneDashboardFetch(active)) {
    vi.stubGlobal('fetch', fetchMock);
    render(<App />);
    fireEvent.change(screen.getByLabelText('Usuário'), { target: { value: 'engineer' } });
    fireEvent.change(screen.getByLabelText('Senha'), { target: { value: 'password' } });
    fireEvent.click(screen.getByRole('button', { name: 'Entrar' }));
    await screen.findAllByText('Show');
  }

  it('renderiza lista de cenas com revisão e data', async () => {
    await loginScenes();
    expect(screen.getByText('Ensaio')).toBeTruthy();
    expect(screen.getAllByText(/Revisão/).length).toBe(2);
    expect(screen.getAllByText(/\d{2}\/\d{2}\/\d{4}/).length).toBe(2);
  });
  it('renderiza nome da cena ativa', async () => { await loginScenes(); expect(screen.getByText('Cena ativa:')).toBeTruthy(); expect(screen.getAllByText('Show').length).toBeGreaterThan(1); });
  it('edita cena e envia nova revisão via PUT', async () => {
    const fetchMock = sceneDashboardFetch(); vi.stubGlobal('fetch', fetchMock); await loginScenes({ id: 'scene-1', name: 'Show' }, fetchMock);
    fireEvent.click(screen.getByRole('button', { name: 'Editar cena Ensaio' }));
    const editor = await screen.findByLabelText('Configuração da cena');
    fireEvent.change(editor, { target: { value: '{\"channels\":[],\"mixes\":[]}' } });
    fireEvent.click(screen.getByRole('button', { name: 'Salvar revisão' }));
    await waitFor(() => expect(fetchMock.mock.calls.some(([path, init]) => path === '/api/v1/scenes/scene-2' && init?.method === 'PUT' && JSON.parse(String(init.body)).config.mixes.length === 0)).toBe(true));
  });
  it('rejeita configuração JSON inválida antes do PUT', async () => {
    const fetchMock = sceneDashboardFetch(); vi.stubGlobal('fetch', fetchMock); await loginScenes({ id: 'scene-1', name: 'Show' }, fetchMock);
    fireEvent.click(screen.getByRole('button', { name: 'Editar cena Ensaio' }));
    fireEvent.change(await screen.findByLabelText('Configuração da cena'), { target: { value: '{inválido' } });
    fireEvent.click(screen.getByRole('button', { name: 'Salvar revisão' }));
    expect(await screen.findByRole('alert')).toHaveTextContent('JSON inválida');
    expect(fetchMock.mock.calls.some(([path, init]) => path === '/api/v1/scenes/scene-2' && init?.method === 'PUT')).toBe(false);
  });

  it('botão recuperar envia POST de recall', async () => {
    const fetchMock = sceneDashboardFetch(); vi.stubGlobal('fetch', fetchMock); await loginScenes({ id: 'scene-1', name: 'Show' }, fetchMock);
    fireEvent.click(screen.getByRole('button', { name: 'Recuperar cena Ensaio' }));
    await waitFor(() => expect(fetchMock.mock.calls.some(([path, init]) => path === '/api/v1/scenes/scene-2/recall' && init?.method === 'POST')).toBe(true));
  });
  it('botão deletar envia DELETE', async () => {
    const fetchMock = sceneDashboardFetch(); vi.stubGlobal('fetch', fetchMock); await loginScenes({ id: 'scene-1', name: 'Show' }, fetchMock);
    fireEvent.click(screen.getByRole('button', { name: 'Deletar cena Ensaio' }));
    await waitFor(() => expect(fetchMock.mock.calls.some(([path, init]) => path === '/api/v1/scenes/scene-2' && init?.method === 'DELETE')).toBe(true));
  });
  it('desabilita deletar cena ativa', async () => { await loginScenes(); expect(screen.getByRole('button', { name: 'Deletar cena Show' })).toBeDisabled(); });
  it('formulário cria cena via POST', async () => {
    const fetchMock = sceneDashboardFetch(); vi.stubGlobal('fetch', fetchMock); await loginScenes({ id: 'scene-1', name: 'Show' }, fetchMock);
    fireEvent.change(screen.getByLabelText('Nome da cena'), { target: { value: 'Nova' } }); fireEvent.click(screen.getByRole('button', { name: 'Criar cena' }));
    await waitFor(() => expect(fetchMock.mock.calls.some(([path, init]) => path === '/api/v1/scenes' && init?.method === 'POST' && JSON.parse(String(init.body)).config.channels.length === 0)).toBe(true));
  });
  it('mostra nenhuma cena ativa quando não há ativa', async () => { await loginScenes(null); expect(screen.getByText('Nenhuma cena ativa')).toBeTruthy(); });
  it('carrega catálogo de presets somente leitura', async () => {
    const fetchMock = sceneDashboardFetch();
    fetchMock.mockImplementation((path: string, init?: RequestInit) => {
      if (path === '/api/v1/presets') return json({ presets: [{ id: 'vocal', name: 'Vocal', kind: 'channel', description: 'Neutro' }] });
      return sceneDashboardFetch()(path, init);
    });
    await loginScenes({ id: 'scene-1', name: 'Show' }, fetchMock);
    expect(await screen.findByText('Vocal')).toBeTruthy();
    expect(screen.getByText(/Presets built-in aplicam defaults/)).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Aplicar' })).toBeTruthy();
  });

  it('bloqueia aplicação de preset em canal locked', async () => {
    await loginAndLoad([makeChannel(0, { locked: true, name: 'Bateria' }), makeChannel(1, { locked: true, name: 'Voz' })]);
    const fetchCalls = vi.mocked(globalThis.fetch);
    fetchCalls.mockImplementation(async (input) => {
      const path = String(input);
      if (path === '/api/v1/presets') return json({ presets: [{ id: 'vocal', name: 'Vocal', kind: 'channel', description: 'Neutro' }] }) as unknown as Response;
      return json({ revision: 2, channels: [makeChannel(0, { locked: true }), makeChannel(1, { locked: true })] }) as unknown as Response;
    });
    expect(await screen.findByText('Canal selecionado está bloqueado para aplicação de preset.')).toBeTruthy();
    expect(screen.getByRole('option', { name: /Bateria.*bloqueado/ })).toHaveProperty('disabled', true);
    expect(screen.getByLabelText('Canal do preset')).toHaveProperty('disabled', true);
    expect(await screen.findByRole('button', { name: 'Aplicar' })).toHaveProperty('disabled', true);
    expect(fetchCalls.mock.calls.some(([path]) => String(path).includes('/presets/vocal/apply'))).toBe(false);
  });

  it('aplica preset ao canal selecionado e recarrega estado', async () => {
    const fetchMock = sceneDashboardFetch();
    fetchMock.mockImplementation((path: string, init?: RequestInit) => {
      if (path === '/api/v1/presets') return json({ presets: [{ id: 'default-vocal', name: 'Vocal', kind: 'channel', description: 'Neutro' }] });
      if (path === '/api/v1/presets/default-vocal/apply') return json({ applied: true, revision: 3 });
      return sceneDashboardFetch()(path, init);
    });
    await loginScenes({ id: 'scene-1', name: 'Show' }, fetchMock);
    fireEvent.click(await screen.findByRole('button', { name: 'Aplicar' }));
    await waitFor(() => expect(fetchMock.mock.calls.some(([path, init]) => path === '/api/v1/presets/default-vocal/apply' && init?.method === 'POST' && JSON.parse(String(init.body)).channel_index === 0)).toBe(true));
    expect(await screen.findByText('Preset default-vocal aplicado no canal 1.')).toBeTruthy();
    expect(fetchMock.mock.calls.filter(([path]) => path === '/api/v1/state').length).toBeGreaterThan(1);
  });

  it('ignora entradas de preset inválidas sem quebrar catálogo', async () => {
    const fetchMock = sceneDashboardFetch();
    fetchMock.mockImplementation((path: string, init?: RequestInit) => {
      if (path === '/api/v1/presets') return json({ presets: [
        { id: 'vocal', name: 'Vocal', kind: 'channel', description: 'Neutro' },
        { id: 'missing-description', name: 'Inválido', kind: 'channel' },
        null,
      ] });
      return sceneDashboardFetch()(path, init);
    });
    await loginScenes({ id: 'scene-1', name: 'Show' }, fetchMock);
    expect(await screen.findByText('Vocal')).toBeTruthy();
    expect(screen.queryByText('Inválido')).toBeNull();
  });

  it('mostra estado vazio para payload de presets não-array', async () => {
    const fetchMock = sceneDashboardFetch();
    fetchMock.mockImplementation((path: string, init?: RequestInit) => {
      if (path === '/api/v1/presets') return json({ presets: {} });
      return sceneDashboardFetch()(path, init);
    });
    await loginScenes({ id: 'scene-1', name: 'Show' }, fetchMock);
    expect(await screen.findByText('Nenhum preset disponível.')).toBeTruthy();
  });

  it('exibe erro ao carregar catálogo de presets', async () => {
    const fetchMock = sceneDashboardFetch();
    fetchMock.mockImplementation((path: string, init?: RequestInit) => {
      if (path === '/api/v1/presets') return json({ error: 'falhou' }, 503);
      return sceneDashboardFetch()(path, init);
    });
    await loginScenes({ id: 'scene-1', name: 'Show' }, fetchMock);
    expect(await screen.findByRole('alert')).toHaveTextContent('503');
  });
  it('mostra erro quando recall falha', async () => {
    const fetchMock = sceneDashboardFetch(); fetchMock.mockImplementation((path: string, init?: RequestInit) => path.endsWith('/recall') ? json({ error: 'falhou' }, 500) : sceneDashboardFetch() (path, init));
    vi.stubGlobal('fetch', fetchMock); await loginScenes({ id: 'scene-1', name: 'Show' }, fetchMock); fireEvent.click(screen.getByRole('button', { name: 'Recuperar cena Ensaio' })); await screen.findByRole('alert'); expect(screen.getByRole('alert')).toHaveTextContent('500');
  });


  it('duplica cena e envia POST para endpoint', async () => {
    const fetchMock = sceneDashboardFetch();
    fetchMock.mockImplementation((path: string) => {
      if (path.endsWith('/duplicate')) return json({ id: 'copy', name: 'Show (cópia)', revision: 1 });
      if (path === '/api/v1/auth/login') return json({ access_token: 'test-token' });
      if (path === '/api/v1/audio/sessions') return json({ sessions: [] });
      if (path === '/api/v1/mixes') return json([]);
      if (path === '/api/v1/state') return json({ revision: 1, channels: [makeChannel(0)] });
      if (path === '/api/v1/telemetry') return json({ availability: 'simulated', backend: 'simulated', sample_rate_hz: null, frames_processed: null, xrun_count: null });
      if (path === '/api/v1/scenes') return json({ scenes: [{ id: 'scene-1', name: 'Show', active_revision: 3, created_at: 1700000000, updated_at: 1700000000 }] });
      if (path === '/api/v1/scenes/active') return json({ scene: { id: 'scene-1', name: 'Show' } });
      return json({}, 204);
    });
    await loginScenes({ id: 'scene-1', name: 'Show' }, fetchMock);
    fireEvent.click(screen.getByRole('button', { name: 'Duplicar cena Show' }));
    await waitFor(() => expect(fetchMock.mock.calls.some(([path, init]) => path === '/api/v1/scenes/scene-1/duplicate' && (init as RequestInit)?.method === 'POST')).toBe(true));
  });

});
