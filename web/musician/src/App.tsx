import { useState, useCallback, useEffect } from 'react';
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

  useEffect(() => {
    if (!ws.snapshot) return;
    const mix = ws.snapshot.mixes[0];
    if (!mix) return;
    const sendsByChannel = new Map(mix.sends.map((send) => [send.channel_index, send]));
    setChannels((current) => current.map((channel, index) => {
      const send = sendsByChannel.get(index);
      return send ? { gainDb: send.gain_db, muted: send.muted } : channel;
    }));
    setMasterGainDb(mix.master_gain_db);
  }, [ws.snapshot]);

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
      const mixIndex = ws.snapshot?.mixes[0]?.index;
      if (mixIndex === undefined) return;
      setChannels((prev) =>
        prev.map((c, i) => (i === ch ? { ...c, gainDb } : c)),
      );
      ws.send({ type: 'SetSendGain', data: { mix_index: mixIndex, channel_index: ch, gain_db: gainDb } });
    },
    [ws],
  );

  const handleChannelMute = useCallback(
    (ch: number, muted: boolean) => {
      const mixIndex = ws.snapshot?.mixes[0]?.index;
      if (mixIndex === undefined) return;
      setChannels((prev) =>
        prev.map((c, i) => (i === ch ? { ...c, muted } : c)),
      );
      ws.send({ type: 'SetSendMuted', data: { mix_index: mixIndex, channel_index: ch, muted } });
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
