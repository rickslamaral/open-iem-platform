import { FormEvent, useCallback, useEffect, useRef, useState } from 'react';
import './style.css';
import { useEngineerWs } from './useEngineerWs';

type Session = { user_id: string; mix_id?: string | null };
type Assignment = { mix_index: number; user_id: number; username: string };
type Telemetry = {
  availability: 'simulated' | 'available' | 'unknown';
  backend: string;
  sample_rate_hz: number | null;
  frames_processed: number | null;
  xrun_count: number | null;
};
type ChannelState = {
  index: number;
  id: number;
  name: string;
  gain_db: number;
  muted: boolean;
  locked: boolean;
  enabled: boolean;
  revision: number;
};
type Dashboard = {
  sessions: Session[];
  assignments: Assignment[];
  revision: number;
  telemetry: Telemetry;
  channels: ChannelState[];
};

/** Gain constants matching server/mix-engine (GAIN_DB_MIN / GAIN_DB_MAX). */
const GAIN_DB_MIN = -144;
const GAIN_DB_MAX = 12;

async function request<T>(path: string, token: string, init: RequestInit = {}): Promise<T> {
  const response = await fetch(path, {
    ...init,
    headers: { 'Content-Type': 'application/json', ...(token ? { Authorization: `Bearer ${token}` } : {}), ...init.headers },
    credentials: 'include',
  });
  if (!response.ok) {
    const detail = await response.text();
    throw new Error(`${response.status}: ${detail || 'request failed'}`);
  }
  if (response.status === 204) return undefined as T;
  return response.json() as Promise<T>;
}

function Login({ onSubmit, error }: { onSubmit: (username: string, password: string) => Promise<void>; error: string | null }) {
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [busy, setBusy] = useState(false);
  const submit = async (event: FormEvent) => {
    event.preventDefault();
    setBusy(true);
    try { await onSubmit(username, password); } finally { setBusy(false); }
  };
  return <main className="login-shell"><form className="card login-card" onSubmit={submit}>
    <p className="eyebrow">OPEN IEM / CONTROL PLANE</p><h1>Engineer Console</h1>
    <p className="muted">Acesso restrito a Engineer ou Admin.</p>
    <label>Usuário<input value={username} onChange={(e) => setUsername(e.target.value)} autoComplete="username" required /></label>
    <label>Senha<input type="password" value={password} onChange={(e) => setPassword(e.target.value)} autoComplete="current-password" required /></label>
    {error && <p className="error" role="alert">{error}</p>}
    <button disabled={busy}>{busy ? 'Entrando…' : 'Entrar'}</button>
  </form></main>;
}

function WsBadge({ status }: { status: string }) {
  const labels: Record<string, string> = { connected: 'WS ✓', connecting: 'WS …', disconnected: 'WS ✗', error: 'WS ✗' };
  return <span className={`pill ws-badge ws-${status}`} role="status" aria-label={`WebSocket ${status}`}>{labels[status] ?? 'WS ?'}</span>;
}

interface MixMasterControlProps {
  mixIndex: number;
  gainDb: number;
  muted: boolean;
  onGain: (gain: number) => void;
  onMute: (muted: boolean) => void;
}

function MixMasterControl({ mixIndex, gainDb, muted, onGain, onMute }: MixMasterControlProps) {
  const [localGain, setLocalGain] = useState(gainDb);

  useEffect(() => { setLocalGain(gainDb); }, [gainDb]);

  return (
    <div className="mix-master-control">
      <strong>Mix {mixIndex + 1} — Master</strong>
      <div className="row" style={{ gap: '0.75rem', alignItems: 'center', flexWrap: 'wrap' }}>
        <label style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', flex: 1 }}>
          <span className="muted" style={{ whiteSpace: 'nowrap' }}>Gain</span>
          <input
            type="range"
            aria-label={`Master gain mix ${mixIndex + 1}`}
            min={-40}
            max={10}
            step={0.5}
            value={localGain}
            onChange={(e) => setLocalGain(Number(e.target.value))}
            onMouseUp={() => { onGain(localGain); }}
            onKeyUp={() => { onGain(localGain); }}
            style={{ flex: 1 }}
          />
          <span aria-live="polite" style={{ minWidth: '3rem', textAlign: 'right' }}>{localGain.toFixed(1)} dB</span>
        </label>
        <button
          aria-label={`Master mute mix ${mixIndex + 1}`}
          aria-pressed={muted}
          className={muted ? 'danger' : 'secondary'}
          onClick={() => { onMute(!muted); }}
          style={{ whiteSpace: 'nowrap' }}
        >
          {muted ? 'MUTED' : 'Mute'}
        </button>
      </div>
    </div>
  );
}

/**
 * ChannelStrip — single-channel gain slider + mute button for the Engineer.
 *
 * Props are controlled: caller owns channel state and provides mutation
 * callbacks so optimistic updates stay outside this component.
 */
function ChannelStrip({
  channel,
  onGainChange,
  onMuteToggle,
  disabled,
}: {
  channel: ChannelState;
  onGainChange: (index: number, gainDb: number) => void;
  onMuteToggle: (index: number, muted: boolean) => void;
  disabled: boolean;
}) {
  return (
    <div className="channel-strip card" aria-label={`Canal ${channel.index + 1}: ${channel.name}`}>
      <div className="channel-header">
        <span className="channel-name">{channel.name || `Ch ${channel.index + 1}`}</span>
        {channel.muted && <span className="pill mute-pill">MUTED</span>}
      </div>
      <label className="gain-label" htmlFor={`gain-${channel.index}`}>
        Gain<span className="gain-value">{channel.gain_db.toFixed(1)} dB</span>
      </label>
      <input
        id={`gain-${channel.index}`}
        type="range"
        min={GAIN_DB_MIN}
        max={GAIN_DB_MAX}
        step={0.5}
        value={channel.gain_db}
        disabled={disabled || channel.locked}
        aria-label={`Gain canal ${channel.index + 1}`}
        className="gain-slider"
        onChange={(e) => onGainChange(channel.index, parseFloat(e.target.value))}
      />
      <button
        className={channel.muted ? 'mute-btn active' : 'mute-btn'}
        disabled={disabled || channel.locked}
        aria-pressed={channel.muted}
        aria-label={`${channel.muted ? 'Desmutar' : 'Mutar'} canal ${channel.index + 1}`}
        onClick={() => onMuteToggle(channel.index, !channel.muted)}
      >
        {channel.muted ? 'Desmutar' : 'Mutar'}
      </button>
    </div>
  );
}


const MAX_EQ_BANDS = 4;

interface EqBandControlProps {
  mixIndex: number;
  bandIndex: number;
  frequencyHz: number;
  gainDb: number;
  q: number;
  enabled: boolean;
  onChange: (mixIndex: number, bandIndex: number, params: { frequency_hz?: number; gain_db?: number; q?: number; enabled?: boolean }) => void;
}

function EqBandControl({ mixIndex, bandIndex, frequencyHz, gainDb, q, enabled, onChange }: EqBandControlProps) {
  const [localFreq, setLocalFreq] = useState(frequencyHz);
  const [localGain, setLocalGain] = useState(gainDb);
  const [localQ, setLocalQ] = useState(q);
  const freqDebounce = useRef<ReturnType<typeof setTimeout> | null>(null);
  const gainDebounceEq = useRef<ReturnType<typeof setTimeout> | null>(null);
  const qDebounce = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => { setLocalFreq(frequencyHz); }, [frequencyHz]);
  useEffect(() => { setLocalGain(gainDb); }, [gainDb]);
  useEffect(() => { setLocalQ(q); }, [q]);

  function debounced(
    ref: React.MutableRefObject<ReturnType<typeof setTimeout> | null>,
    key: 'frequency_hz' | 'gain_db' | 'q',
    value: number,
  ) {
    if (ref.current !== null) clearTimeout(ref.current);
    ref.current = setTimeout(() => {
      onChange(mixIndex, bandIndex, { [key]: value });
      ref.current = null;
    }, 300);
  }

  return (
    <div className="eq-band" aria-label={`EQ Mix ${mixIndex + 1} Band ${bandIndex + 1}`}>
      <span className="muted" style={{ fontSize: '0.75rem' }}>Band {bandIndex + 1}</span>
      <label style={{ display: 'flex', alignItems: 'center', gap: '0.4rem', fontSize: '0.8rem' }}>
        <input
          type="checkbox"
          aria-label={`EQ band ${bandIndex + 1} mix ${mixIndex + 1} enabled`}
          checked={enabled}
          onChange={(e) => onChange(mixIndex, bandIndex, { enabled: e.target.checked })}
        />
        On
      </label>
      <label style={{ display: 'flex', alignItems: 'center', gap: '0.4rem', fontSize: '0.8rem' }}>
        <span style={{ minWidth: '2.5rem' }}>Freq</span>
        <input
          type="range"
          aria-label={`EQ band ${bandIndex + 1} mix ${mixIndex + 1} frequency`}
          min={20}
          max={20000}
          step={1}
          value={localFreq}
          disabled={!enabled}
          onChange={(e) => {
            const v = Number(e.target.value);
            setLocalFreq(v);
            debounced(freqDebounce, 'frequency_hz', v);
          }}
          style={{ flex: 1 }}
        />
        <span style={{ minWidth: '3.5rem', textAlign: 'right' }}>{localFreq} Hz</span>
      </label>
      <label style={{ display: 'flex', alignItems: 'center', gap: '0.4rem', fontSize: '0.8rem' }}>
        <span style={{ minWidth: '2.5rem' }}>Gain</span>
        <input
          type="range"
          aria-label={`EQ band ${bandIndex + 1} mix ${mixIndex + 1} gain`}
          min={-24}
          max={24}
          step={0.5}
          value={localGain}
          disabled={!enabled}
          onChange={(e) => {
            const v = Number(e.target.value);
            setLocalGain(v);
            debounced(gainDebounceEq, 'gain_db', v);
          }}
          style={{ flex: 1 }}
        />
        <span style={{ minWidth: '3.5rem', textAlign: 'right' }}>{localGain.toFixed(1)} dB</span>
      </label>
      <label style={{ display: 'flex', alignItems: 'center', gap: '0.4rem', fontSize: '0.8rem' }}>
        <span style={{ minWidth: '2.5rem' }}>Q</span>
        <input
          type="range"
          aria-label={`EQ band ${bandIndex + 1} mix ${mixIndex + 1} Q`}
          min={0.1}
          max={10.0}
          step={0.1}
          value={localQ}
          disabled={!enabled}
          onChange={(e) => {
            const v = Number(e.target.value);
            setLocalQ(v);
            debounced(qDebounce, 'q', v);
          }}
          style={{ flex: 1 }}
        />
        <span style={{ minWidth: '3.5rem', textAlign: 'right' }}>{localQ.toFixed(1)}</span>
      </label>
    </div>
  );
}

type SceneSummary = { id: string; name: string; active_revision: number; created_at: number; updated_at: number };
type Scene = { id: string; name: string; revision: number };

function ScenePanel({ token }: { token: string }) {
  const [scenes, setScenes] = useState<SceneSummary[]>([]);
  const [activeScene, setActiveScene] = useState<Scene | null>(null);
  const [name, setName] = useState('');
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const loadScenes = useCallback(async () => {
    setLoading(true); setError(null);
    try {
      const [list, active] = await Promise.all([
        request<{ scenes: SceneSummary[] }>('/api/v1/scenes', token),
        request<{ scene: Scene | null }>('/api/v1/scenes/active', token),
      ]);
      setScenes(Array.isArray(list?.scenes) ? list.scenes : []); setActiveScene(active?.scene ?? null);
    } catch (cause) { setError(cause instanceof Error ? cause.message : 'Falha ao carregar cenas'); }
    finally { setLoading(false); }
  }, [token]);
  useEffect(() => { void loadScenes(); }, [loadScenes]);
  async function mutate(id: string, method: 'DELETE' | 'POST', action: string) {
    setBusy(id); setError(null);
    try { await request(`/api/v1/scenes/${id}${action}`, token, { method }); await loadScenes(); }
    catch (cause) { setError(cause instanceof Error ? cause.message : 'Falha ao alterar cena'); }
    finally { setBusy(null); }
  }
  async function create(event: FormEvent) {
    event.preventDefault(); if (!name.trim()) return;
    setBusy('create'); setError(null);
    try { await request('/api/v1/scenes', token, { method: 'POST', body: JSON.stringify({ name: name.trim(), config: { channels: [], mixes: [] } }) }); setName(''); await loadScenes(); }
    catch (cause) { setError(cause instanceof Error ? cause.message : 'Falha ao criar cena'); }
    finally { setBusy(null); }
  }
  return <>
    <p className="muted">Cena ativa: <strong>{activeScene?.name ?? 'Nenhuma cena ativa'}</strong></p>
    <form onSubmit={create} style={{ display: 'flex', gap: '0.5rem', marginBottom: '1rem', flexWrap: 'wrap' }}>
      <label style={{ flex: 1, minWidth: '12rem' }}><span className="muted">Nome da cena</span><input aria-label="Nome da cena" value={name} onChange={(e) => setName(e.target.value)} required /></label>
      <button aria-label="Criar cena" disabled={busy !== null}>Criar</button>
    </form>
    {loading && <p className="muted" role="status">Carregando cenas…</p>}
    {error && <p className="error" role="alert">{error}</p>}
    {!loading && scenes.length === 0 && <p className="muted">Nenhuma cena cadastrada.</p>}
    {scenes.map((scene) => { const active = activeScene?.id === scene.id; return <div className="row" key={scene.id}>
      <div><strong>{scene.name}</strong><br /><span className="muted">Revisão {scene.active_revision} · {new Date(scene.created_at * 1000).toLocaleDateString('pt-BR')}</span></div>
      <div className="row" style={{ gap: '0.5rem' }}><button aria-label={`Recuperar cena ${scene.name}`} disabled={busy !== null} onClick={() => void mutate(scene.id, 'POST', '/recall')}>Recuperar</button><button className="danger" aria-label={`Deletar cena ${scene.name}`} disabled={active || busy !== null} onClick={() => void mutate(scene.id, 'DELETE', '')}>Deletar</button></div>
    </div>; })}
  </>;
}

export default function App() {
  const [token, setToken] = useState<string | null>(null);
  const [data, setData] = useState<Dashboard | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [userId, setUserId] = useState('');
  /** Tracks in-flight channel mutations so UI reflects optimistic state. */
  const [pendingChannels, setPendingChannels] = useState<Record<number, Partial<ChannelState>>>({});
  /** Mutation identity prevents stale responses overwriting newer optimistic state. */
  const channelMutationIds = useRef<Record<number, number>>({});
  const nextMutationId = useRef(0);
  /** Drops out-of-order dashboard responses. */
  const loadGeneration = useRef(0);
  /** Debounce timers for gain slider (avoids a request per pixel). */
  const gainDebounce = useRef<Record<number, ReturnType<typeof setTimeout>>>({});

  useEffect(() => () => {
    Object.values(gainDebounce.current).forEach(clearTimeout);
    loadGeneration.current += 1;
    channelMutationIds.current = {};
  }, []);

  const ws = useEngineerWs(token);

  const load = useCallback(async () => {
    if (!token) return;
    const generation = ++loadGeneration.current;
    setLoading(true); setError(null);
    try {
      const [sessions, assignments, state, telemetry] = await Promise.all([
        request<{ sessions: Session[] }>('/api/v1/audio/sessions', token),
        request<Assignment[]>('/api/v1/mixes', token),
        request<{ revision: number; channels?: ChannelState[] }>('/api/v1/state', token),
        request<Telemetry>('/api/v1/telemetry', token).catch(() => ({
          availability: 'unknown' as const,
          backend: 'unknown',
          sample_rate_hz: null,
          frames_processed: null,
          xrun_count: null,
        })),
      ]);
      if (generation !== loadGeneration.current) return;
      setData({
        sessions: sessions.sessions,
        assignments,
        revision: state.revision,
        telemetry,
        channels: state.channels ?? [],
      });
      // Keep overlays while mutations are in flight; each mutation clears its own overlay.
    } catch (cause) {
      if (generation !== loadGeneration.current) return;
      const message = cause instanceof Error ? cause.message : 'Falha ao carregar console';
      if (message.startsWith('401:')) {
        try {
          const refreshed = await request<{ access_token: string }>('/api/v1/auth/refresh', '');
          setToken(refreshed.access_token);
          setError('Sessão renovada; atualizando console.');
        } catch { setToken(null); setData(null); setError('Sessão expirada. Entre novamente.'); }
      } else if (message.startsWith('403:')) {
        setToken(null); setData(null); setError('Conta sem permissão de Engineer/Admin.');
      } else setError(message);
    } finally { setLoading(false); }
  }, [token]);

  useEffect(() => { void load(); }, [load]);
  useEffect(() => {
    if (!token) return;
    const timer = window.setInterval(() => { if (document.visibilityState === 'visible') void load(); }, 5000);
    return () => window.clearInterval(timer);
  }, [load, token]);

  async function login(username: string, password: string) {
    setError(null);
    try {
      const result = await request<{ access_token: string }>('/api/v1/auth/login', '', { method: 'POST', body: JSON.stringify({ username, password }) });
      setToken(result.access_token);
    } catch (cause) { setError(cause instanceof Error ? cause.message : 'Falha de autenticação'); }
  }
  async function logout() {
    if (token) { try { await request('/api/v1/auth/logout', token, { method: 'POST' }); } catch { /* local logout still required */ } }
    Object.values(gainDebounce.current).forEach(clearTimeout);
    gainDebounce.current = {};
    loadGeneration.current += 1;
    channelMutationIds.current = {};
    setPendingChannels({});
    setToken(null); setData(null); setError(null);
  }
  async function assign(mixIndex: number) {
    if (!token || !/^[1-9]\d*$/.test(userId)) { setError('Informe ID numérico positivo de usuário.'); return; }
    try { await request(`/api/v1/mixes/${mixIndex}/assign`, token, { method: 'POST', body: JSON.stringify({ user_id: Number(userId) }) }); setUserId(''); await load(); }
    catch (cause) { setError(cause instanceof Error ? cause.message : 'Falha ao atribuir mix'); }
  }
  async function unassign(mixIndex: number) {
    if (!token) return;
    try { await request(`/api/v1/mixes/${mixIndex}/assign`, token, { method: 'DELETE' }); await load(); }
    catch (cause) { setError(cause instanceof Error ? cause.message : 'Falha ao remover mix'); }
  }

  /** Return unique mutation ID and mark it current for channel. */
  function beginChannelMutation(channelIndex: number): number {
    const mutationId = ++nextMutationId.current;
    channelMutationIds.current[channelIndex] = mutationId;
    return mutationId;
  }

  /** Clear overlay only when response belongs to current mutation. */
  function finishChannelMutation(channelIndex: number, mutationId: number) {
    if (channelMutationIds.current[channelIndex] !== mutationId) return;
    // Invalidate loads started before mutation; refresh authoritative state after response.
    loadGeneration.current += 1;
    setPendingChannels((prev) => {
      const next = { ...prev };
      delete next[channelIndex];
      return next;
    });
    // Refresh authoritative state; failed writes must revert optimistic UI.
    void load();
  }

  /** Optimistic gain update with 300 ms debounce before network request. */
  function handleGainChange(channelIndex: number, gainDb: number) {
    if (!token) return;
    // Apply optimistic overlay immediately; newest slider value supersedes older timer.
    setPendingChannels((prev) => ({ ...prev, [channelIndex]: { ...prev[channelIndex], gain_db: gainDb } }));
    if (gainDebounce.current[channelIndex] !== undefined) clearTimeout(gainDebounce.current[channelIndex]);
    gainDebounce.current[channelIndex] = setTimeout(() => {
      const mutationId = beginChannelMutation(channelIndex);
      void (async () => {
        try {
          await request(`/api/v1/channels/${channelIndex}/gain`, token, {
            method: 'PUT', body: JSON.stringify({ gain_db: gainDb }),
          });
        } catch (cause) {
          setError(cause instanceof Error ? cause.message : 'Falha ao ajustar gain');
        }
        delete gainDebounce.current[channelIndex];
        finishChannelMutation(channelIndex, mutationId);
      })();
    }, 300);
  }

  /** Optimistic mute toggle — immediate UI then network request. */
  function handleMuteToggle(channelIndex: number, muted: boolean) {
    if (!token) return;
    const mutationId = beginChannelMutation(channelIndex);
    setPendingChannels((prev) => ({ ...prev, [channelIndex]: { ...prev[channelIndex], muted } }));
    void (async () => {
      try {
        await request(`/api/v1/channels/${channelIndex}/mute`, token, {
          method: 'PUT', body: JSON.stringify({ muted }),
        });
      } catch (cause) {
        setError(cause instanceof Error ? cause.message : 'Falha ao alterar mute');
      }
      finishChannelMutation(channelIndex, mutationId);
    })();
  }

  /** Merge server channel data with any pending optimistic overrides. */
  function resolvedChannels(): ChannelState[] {
    return (data?.channels ?? []).map((ch) => {
      const pending = pendingChannels[ch.index];
      return pending ? { ...ch, ...pending } : ch;
    });
  }

  if (!token) return <Login onSubmit={login} error={error} />;
  const channels = resolvedChannels();
  return <main className="app-shell">
    <header>
      <div><p className="eyebrow">OPEN IEM / CONTROL PLANE</p><h1>Engineer Console</h1></div>
      <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
        <WsBadge status={ws.status} />
        <button className="secondary" onClick={() => void logout()}>Sair</button>
      </div>
    </header>
    <div className="notice"><strong>Áudio SIMULATED</strong><span>VPS sem PipeWire. Sessões WebRTC não representam mídia validada em hardware.</span></div>
    {(error ?? ws.error) && <div className="error banner" role="alert">{error ?? ws.error}</div>}
    <section className="metrics">
      <div className="card"><span className="muted">Revision</span><strong>{ws.revision ?? data?.revision ?? '—'}</strong></div>
      <div className="card"><span className="muted">Sessões ativas</span><strong>{data?.sessions.length ?? 0}</strong></div>
      <div className="card"><span className="muted">Backend</span><strong>{loading ? 'carregando' : (data?.telemetry.backend ?? '—')}</strong></div>
      <div className="card"><span className="muted">XRUNs</span><strong>{data?.telemetry.xrun_count ?? 'UNKNOWN'}</strong></div>
    </section>
    {channels.length > 0 && (
      <section className="channel-section">
        <h2 className="section-title">Canais de entrada</h2>
        <div className="channel-grid">
          {channels.map((ch) => (
            <ChannelStrip
              key={ch.index}
              channel={ch}
              onGainChange={handleGainChange}
              onMuteToggle={handleMuteToggle}
              disabled={loading || pendingChannels[ch.index] !== undefined}
            />
          ))}
        </div>
      </section>
    )}
    <section className="grid">
      <div className="card">
        <h2>Controles de Master</h2>
        <p className="muted">Gain (−40 a +10 dB) e mute por mix. Requer papel Engineer ou Admin.</p>
        {[0, 1].map((mix) => (
          <MixMasterControl
            key={mix}
            mixIndex={mix}
            gainDb={ws.mixes[mix]?.master_gain_db ?? 0}
            muted={ws.mixes[mix]?.master_muted ?? false}
            onGain={(gain) => ws.setMasterGain(mix, gain)}
            onMute={(muted) => ws.setMasterMute(mix, muted)}
          />
        ))}
      </div>
      <div className="card">
        <h2>EQ por Mix</h2>
        <p className="muted">Equalização paramétrica por banda — SIMULATED (sem validação em hardware).</p>
        {[0, 1].map((mix) => (
          <div key={mix} style={{ marginBottom: '1rem' }}>
            <strong>Mix {mix + 1} — EQ</strong>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '0.5rem', marginTop: '0.5rem' }}>
              {Array.from({ length: MAX_EQ_BANDS }, (_, band) => (
                <EqBandControl
                  key={band}
                  mixIndex={mix}
                  bandIndex={band}
                  frequencyHz={ws.eqBands[mix]?.[band]?.frequency_hz ?? 1000}
                  gainDb={ws.eqBands[mix]?.[band]?.gain_db ?? 0}
                  q={ws.eqBands[mix]?.[band]?.q ?? 1.4}
                  enabled={ws.eqBands[mix]?.[band]?.enabled ?? false}
                  onChange={ws.setEqBand}
                />
              ))}
            </div>
          </div>
        ))}
      </div>
      <div className="card"><h2>Cenas</h2>{data && <ScenePanel token={token} />}</div>
      <div className="card"><h2>Mix assignments</h2><p className="muted">Atribuição exige ID do usuário. Catálogo de usuários fica restrito a Admin.</p>
        {[0, 1].map((mix) => { const assignment = data?.assignments.find((item) => item.mix_index === mix); return <div className="row" key={mix}><div><strong>Mix {mix + 1}</strong><br /><span className="muted">{assignment ? `${assignment.username} (ID ${assignment.user_id})` : 'Livre'}</span></div>{assignment ? <button className="danger" onClick={() => void unassign(mix)}>Remover</button> : <div className="assign"><input aria-label={`ID usuário mix ${mix + 1}`} inputMode="numeric" placeholder="ID usuário" value={userId} onChange={(e) => setUserId(e.target.value)} /><button onClick={() => void assign(mix)}>Atribuir</button></div>}</div>; })}
      </div>
      <div className="card"><h2>Sessões de áudio</h2>{data?.sessions.length ? data.sessions.map((session) => <div className="row" key={session.user_id}><span>{session.user_id}</span><span className="pill">ativa</span></div>) : <p className="muted">Nenhuma sessão ativa.</p>}</div>
    </section>
  </main>;
}
