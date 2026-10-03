import { useEffect, useRef, useState } from 'react';
import styles from './Login.module.css';
const instruments = [['vocals', 'Vocals'], ['guitar', 'Guitar'], ['bass', 'Bass'], ['drums', 'Drums'], ['keys', 'Keys'], ['acoustic-guitar', 'Acoustic guitar'], ['brass', 'Brass'], ['strings', 'Strings']] as const;
const invitationToken = (value: string): string => {
  try {
    const url = new URL(value);
    return url.searchParams.get('qr_secret')?.trim() ?? value.trim();
  } catch { return value.trim(); }
};
interface Props { onLogin: (username: string, password: string) => Promise<void>; onQrExchange: (qrSecret: string, displayName: string, instrumentId: string, username: string, password: string) => Promise<void>; error: string | null; }

interface BarcodeDetectorLike { detect(source: ImageBitmapSource): Promise<Array<{ rawValue?: string }>>; }
declare global { interface Window { BarcodeDetector?: new (options?: { formats?: string[] }) => BarcodeDetectorLike; } }

export function Login({ onLogin, onQrExchange, error }: Props) {
  const [username, setUsername] = useState(''); const [password, setPassword] = useState('');
  const [qrSecret, setQrSecret] = useState(''); const [displayName, setDisplayName] = useState(''); const [instrumentId, setInstrumentId] = useState('vocals');
  const [qrUsername, setQrUsername] = useState(''); const [qrPassword, setQrPassword] = useState('');
  const [loading, setLoading] = useState(false); const [qrMode, setQrMode] = useState(false);
  const [cameraOpen, setCameraOpen] = useState(false); const [cameraError, setCameraError] = useState<string | null>(null);
  const videoRef = useRef<HTMLVideoElement>(null); const streamRef = useRef<MediaStream | null>(null);
  const scanGenerationRef = useRef(0);
  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    const invitation = params.get('invitation');
    if (invitation) {
      setQrSecret(invitationToken(invitation));
      setQrMode(true);
      window.history.replaceState({}, document.title, `${window.location.pathname}${window.location.hash}`);
    }
  }, []);
  useEffect(() => {
    if (!qrMode) {
      scanGenerationRef.current += 1;
      streamRef.current?.getTracks().forEach((track) => track.stop());
      streamRef.current = null;
      setCameraOpen(false);
    }
  }, [qrMode]);
  useEffect(() => {
    if (!cameraOpen || !streamRef.current || !videoRef.current) return;
    videoRef.current.srcObject = streamRef.current;
    void videoRef.current.play().catch(() => undefined);
  }, [cameraOpen]);
  useEffect(() => () => { scanGenerationRef.current += 1; streamRef.current?.getTracks().forEach((track) => track.stop()); streamRef.current = null; }, []);
  const stopCamera = () => { scanGenerationRef.current += 1; streamRef.current?.getTracks().forEach((track) => track.stop()); streamRef.current = null; setCameraOpen(false); };
  const scanCamera = async () => {
    const generation = scanGenerationRef.current + 1; scanGenerationRef.current = generation;
    setCameraError(null);
    streamRef.current?.getTracks().forEach((track) => track.stop()); streamRef.current = null;
    if (!window.BarcodeDetector) { setCameraError('Camera QR scanning is not supported by this browser. Paste QR secret instead.'); return; }
    if (!navigator.mediaDevices?.getUserMedia) { setCameraError('Camera access is unavailable. Paste QR secret instead.'); return; }
    try {
      const stream = await navigator.mediaDevices.getUserMedia({ video: { facingMode: { ideal: 'environment' } }, audio: false });
      if (scanGenerationRef.current !== generation) { stream.getTracks().forEach((track) => track.stop()); return; }
      streamRef.current = stream; setCameraOpen(true);
      await new Promise((resolve) => window.requestAnimationFrame(resolve));
      if (scanGenerationRef.current !== generation || streamRef.current !== stream || !videoRef.current) { stream.getTracks().forEach((track) => track.stop()); return; }
      const detector = new window.BarcodeDetector({ formats: ['qr_code'] });
      for (let attempt = 0; attempt < 120 && streamRef.current && scanGenerationRef.current === generation; attempt += 1) {
        if (videoRef.current?.readyState === HTMLMediaElement.HAVE_ENOUGH_DATA) {
          const codes = await detector.detect(videoRef.current);
          if (scanGenerationRef.current !== generation || streamRef.current !== stream) return;
          const value = codes[0]?.rawValue?.trim();
          if (value) { setQrSecret(value); stopCamera(); return; }
        }
        await new Promise((resolve) => window.setTimeout(resolve, 100));
      }
      if (streamRef.current && scanGenerationRef.current === generation) { stopCamera(); setCameraError('No QR code detected. Paste QR secret or try again.'); }
    } catch {
      if (scanGenerationRef.current === generation) { stopCamera(); setCameraError('Camera scanner failed. Paste QR secret or try again.'); }
    }
  };
  const handleQrSubmit = async (e: React.FormEvent) => {
    e.preventDefault(); setLoading(true);
    try { await onQrExchange(qrSecret.trim(), displayName, instrumentId, qrUsername, qrPassword); }
    finally {
      setLoading(false); setQrSecret(''); setQrUsername(''); setQrPassword(''); setDisplayName(''); setInstrumentId('vocals'); stopCamera();
    }
  };
  const handlePasswordSubmit = async (e: React.FormEvent) => { e.preventDefault(); setLoading(true); try { await onLogin(username, password); } finally { setLoading(false); } };
  return <div className={styles.container}><div className={styles.card}>
    <h1 className={styles.title}>Open IEM</h1><p className={styles.subtitle}>Musician Monitor Control</p>
    <div role="tablist" aria-label="Sign-in methods"><button type="button" onClick={() => setQrMode(true)} aria-selected={qrMode}>QR onboarding</button><button type="button" onClick={() => setQrMode(false)} aria-selected={!qrMode}>Username login</button></div>
    {qrMode ? <form onSubmit={handleQrSubmit} className={styles.form}>
      <p>Scan QR code with camera or paste QR secret.</p>
      {cameraOpen && <><video ref={videoRef} className={styles.camera} muted playsInline aria-label="QR camera preview" /><button type="button" className={styles.secondaryButton} onClick={stopCamera}>Stop camera</button></>}
      {!cameraOpen && <button type="button" className={styles.secondaryButton} onClick={() => void scanCamera()} disabled={loading}>Scan with camera</button>}
      {cameraError && <p role="status" className={styles.error}>{cameraError}</p>}
      <label htmlFor="qr-secret" className={styles.label}>QR secret</label><input id="qr-secret" type="text" value={qrSecret} onChange={(e) => setQrSecret(e.target.value)} className={styles.input} required disabled={loading} autoComplete="off" />
      <label htmlFor="display-name" className={styles.label}>Display name</label><input id="display-name" type="text" value={displayName} onChange={(e) => setDisplayName(e.target.value)} className={styles.input} required disabled={loading} />
      <label htmlFor="instrument-id" className={styles.label}>Instrument</label><select id="instrument-id" value={instrumentId} onChange={(e) => setInstrumentId(e.target.value)} className={styles.input} disabled={loading}>{instruments.map(([id, label]) => <option key={id} value={id}>{label}</option>)}</select>
      <label htmlFor="qr-username" className={styles.label}>Username</label><input id="qr-username" value={qrUsername} onChange={(e) => setQrUsername(e.target.value)} className={styles.input} required disabled={loading} autoComplete="username" />
      <label htmlFor="qr-password" className={styles.label}>Password</label><input id="qr-password" type="password" value={qrPassword} onChange={(e) => setQrPassword(e.target.value)} className={styles.input} required disabled={loading} autoComplete="new-password" />
      {error && <p role="alert" className={styles.error}>{error}</p>}<button type="submit" className={styles.button} disabled={loading}>{loading ? 'Joining…' : 'Join with QR secret'}</button>
    </form> : <form onSubmit={handlePasswordSubmit} className={styles.form}>
      <label htmlFor="username" className={styles.label}>Username</label><input id="username" type="text" autoComplete="username" value={username} onChange={(e) => setUsername(e.target.value)} className={styles.input} required disabled={loading} />
      <label htmlFor="password" className={styles.label}>Password</label><input id="password" type="password" autoComplete="current-password" value={password} onChange={(e) => setPassword(e.target.value)} className={styles.input} required disabled={loading} />
      {error && <p role="alert" className={styles.error}>{error}</p>}<button type="submit" className={styles.button} disabled={loading}>{loading ? 'Signing in…' : 'Sign in'}</button>
    </form>}
  </div></div>;
}
