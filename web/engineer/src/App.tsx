import { FormEvent, useCallback, useEffect, useState } from 'react';
import './style.css';

type Session = { user_id: string; mix_id?: string | null };
type Assignment = { mix_index: number; user_id: number; username: string };
type Dashboard = { sessions: Session[]; assignments: Assignment[]; revision: number };

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

export default function App() {
  const [token, setToken] = useState<string | null>(null);
  const [data, setData] = useState<Dashboard | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [userId, setUserId] = useState('');

  const load = useCallback(async () => {
    if (!token) return;
    setLoading(true); setError(null);
    try {
      const [sessions, assignments, state] = await Promise.all([
        request<{ sessions: Session[] }>('/api/v1/audio/sessions', token),
        request<Assignment[]>('/api/v1/mixes', token),
        request<{ revision: number }>('/api/v1/state', token),
      ]);
      setData({ sessions: sessions.sessions, assignments, revision: state.revision });
    } catch (cause) {
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

  if (!token) return <Login onSubmit={login} error={error} />;
  return <main className="app-shell">
    <header><div><p className="eyebrow">OPEN IEM / CONTROL PLANE</p><h1>Engineer Console</h1></div><button className="secondary" onClick={() => void logout()}>Sair</button></header>
    <div className="notice"><strong>Áudio SIMULATED</strong><span>VPS sem PipeWire. Sessões WebRTC não representam mídia validada em hardware.</span></div>
    {error && <div className="error banner" role="alert">{error}</div>}
    <section className="metrics"><div className="card"><span className="muted">Revision</span><strong>{data?.revision ?? '—'}</strong></div><div className="card"><span className="muted">Sessões ativas</span><strong>{data?.sessions.length ?? 0}</strong></div><div className="card"><span className="muted">Atualização</span><strong>{loading ? 'carregando' : '5 s'}</strong></div></section>
    <section className="grid"><div className="card"><h2>Mix assignments</h2><p className="muted">Atribuição exige ID do usuário. Catálogo de usuários fica restrito a Admin.</p>
      {[0, 1].map((mix) => { const assignment = data?.assignments.find((item) => item.mix_index === mix); return <div className="row" key={mix}><div><strong>Mix {mix + 1}</strong><br /><span className="muted">{assignment ? `${assignment.username} (ID ${assignment.user_id})` : 'Livre'}</span></div>{assignment ? <button className="danger" onClick={() => void unassign(mix)}>Remover</button> : <div className="assign"><input aria-label={`ID usuário mix ${mix + 1}`} inputMode="numeric" placeholder="ID usuário" value={userId} onChange={(e) => setUserId(e.target.value)} /><button onClick={() => void assign(mix)}>Atribuir</button></div>}</div>; })}
    </div><div className="card"><h2>Sessões de áudio</h2>{data?.sessions.length ? data.sessions.map((session) => <div className="row" key={session.user_id}><span>{session.user_id}</span><span className="pill">ativa</span></div>) : <p className="muted">Nenhuma sessão ativa.</p>}</div></section>
  </main>;
}
