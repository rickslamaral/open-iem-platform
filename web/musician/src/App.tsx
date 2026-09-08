import { useState, useCallback } from 'react';
import { Login } from './components/Login';
import { MixControl } from './components/MixControl';
import { ConnectionStatus } from './components/ConnectionStatus';
import { useWebSocket } from './hooks/useWebSocket';
import { login as apiLogin, logout as apiLogout } from './api/auth';
import styles from './App.module.css';

const CHANNEL_COUNT = 8;

interface ChannelState {
  gainDb: number;
  muted: boolean;
}

const defaultChannels = (): ChannelState[] =>
  Array.from({ length: CHANNEL_COUNT }, () => ({ gainDb: 0, muted: false }));

export default function App() {
  // Access token stored in React state only — never in localStorage
  const [token, setToken] = useState<string | null>(null);
  const [loginError, setLoginError] = useState<string | null>(null);
  const [channels, setChannels] = useState<ChannelState[]>(defaultChannels);
  const [masterGainDb, setMasterGainDb] = useState(0);

  const ws = useWebSocket(token);

  const handleLogin = useCallback(async (username: string, password: string) => {
    setLoginError(null);
    try {
      const res = await apiLogin({ username, password });
      setToken(res.access_token);
    } catch (err) {
      setLoginError(err instanceof Error ? err.message : 'Login failed');
    }
  }, []);

  const handleLogout = useCallback(async () => {
    ws.disconnect();
    setToken(null);
    setChannels(defaultChannels());
    setMasterGainDb(0);
    await apiLogout().catch(() => undefined);
  }, [ws]);

  const handleChannelGain = useCallback(
    (ch: number, gainDb: number) => {
      setChannels((prev) =>
        prev.map((c, i) => (i === ch ? { ...c, gainDb } : c)),
      );
      ws.send({ type: 'SetChannelGain', data: { channel: ch, gain_db: gainDb } });
    },
    [ws],
  );

  const handleChannelMute = useCallback(
    (ch: number, muted: boolean) => {
      setChannels((prev) =>
        prev.map((c, i) => (i === ch ? { ...c, muted } : c)),
      );
      ws.send({ type: 'SetChannelMute', data: { channel: ch, muted } });
    },
    [ws],
  );

  if (!token) {
    return <Login onLogin={handleLogin} error={loginError} />;
  }

  return (
    <div className={styles.appRoot}>
      <div className={styles.statusBar}>
        <ConnectionStatus status={ws.status} revision={ws.revision} />
      </div>
      <MixControl
        ws={ws}
        channels={channels}
        masterGainDb={masterGainDb}
        onChannelGain={handleChannelGain}
        onChannelMute={handleChannelMute}
        onMasterGain={setMasterGainDb}
        onLogout={handleLogout}
      />
    </div>
  );
}
