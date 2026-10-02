import { useState } from 'react';
import styles from './Login.module.css';

const instruments = [['vocals', 'Vocals'], ['guitar', 'Guitar'], ['bass', 'Bass'], ['drums', 'Drums'], ['keys', 'Keys'], ['acoustic-guitar', 'Acoustic guitar'], ['brass', 'Brass'], ['strings', 'Strings']] as const;
interface Props { onLogin: (username: string, password: string) => Promise<void>; onQrExchange: (qrSecret: string, displayName: string, instrumentId: string) => Promise<void>; error: string | null; }

export function Login({ onLogin, onQrExchange, error }: Props) {
  const [username, setUsername] = useState(''); const [password, setPassword] = useState('');
  const [qrSecret, setQrSecret] = useState(''); const [displayName, setDisplayName] = useState(''); const [instrumentId, setInstrumentId] = useState('vocals');
  const [loading, setLoading] = useState(false); const [qrMode, setQrMode] = useState(false);
  const handleQrSubmit = async (e: React.FormEvent) => { e.preventDefault(); setLoading(true); try { await onQrExchange(qrSecret.trim(), displayName, instrumentId); } finally { setLoading(false); } };
  const handlePasswordSubmit = async (e: React.FormEvent) => { e.preventDefault(); setLoading(true); try { await onLogin(username, password); } finally { setLoading(false); } };
  return <div className={styles.container}><div className={styles.card}>
    <h1 className={styles.title}>Open IEM</h1><p className={styles.subtitle}>Musician Monitor Control</p>
    <div role="tablist" aria-label="Sign-in methods"><button type="button" onClick={() => setQrMode(true)} aria-selected={qrMode}>QR onboarding</button><button type="button" onClick={() => setQrMode(false)} aria-selected={!qrMode}>Username login</button></div>
    {qrMode ? <form onSubmit={handleQrSubmit} className={styles.form}>
      <p>Local development input: paste QR secret. Camera scanning is not available yet.</p>
      <label htmlFor="qr-secret" className={styles.label}>QR secret</label><input id="qr-secret" type="text" value={qrSecret} onChange={(e) => setQrSecret(e.target.value)} className={styles.input} required disabled={loading} autoComplete="off" />
      <label htmlFor="display-name" className={styles.label}>Display name</label><input id="display-name" type="text" value={displayName} onChange={(e) => setDisplayName(e.target.value)} className={styles.input} required disabled={loading} />
      <label htmlFor="instrument-id" className={styles.label}>Instrument</label><select id="instrument-id" value={instrumentId} onChange={(e) => setInstrumentId(e.target.value)} className={styles.input} disabled={loading}>{instruments.map(([id, label]) => <option key={id} value={id}>{label}</option>)}</select>
      {error && <p role="alert" className={styles.error}>{error}</p>}<button type="submit" className={styles.button} disabled={loading}>{loading ? 'Joining…' : 'Join with QR secret'}</button>
    </form> : <form onSubmit={handlePasswordSubmit} className={styles.form}>
      <label htmlFor="username" className={styles.label}>Username</label><input id="username" type="text" autoComplete="username" value={username} onChange={(e) => setUsername(e.target.value)} className={styles.input} required disabled={loading} />
      <label htmlFor="password" className={styles.label}>Password</label><input id="password" type="password" autoComplete="current-password" value={password} onChange={(e) => setPassword(e.target.value)} className={styles.input} required disabled={loading} />
      {error && <p role="alert" className={styles.error}>{error}</p>}<button type="submit" className={styles.button} disabled={loading}>{loading ? 'Signing in…' : 'Sign in'}</button>
    </form>}
  </div></div>;
}
