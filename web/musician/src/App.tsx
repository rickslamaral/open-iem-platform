import { useState, useCallback, useEffect, useRef } from 'react';
import { Login } from './components/Login';
import { MixControl } from './components/MixControl';
import { ConnectionStatus } from './components/ConnectionStatus';
import { useWebSocket } from './hooks/useWebSocket';
import { login as apiLogin, logout as apiLogout } from './api/auth';
import styles from './App.module.css';
import { fetchChannelMetadata } from './api/channels';
import { fetchActiveSceneId, fetchScenes } from './api/scenes';
import type { SceneSummary } from './api/scenes';
import { SceneList } from './components/SceneList';
import { PresetList } from './components/PresetList';
import { fetchPresets } from './api/presets';

const CHANNEL_COUNT = 8;

interface ChannelState {
  gainDb: number;
  muted: boolean;
}

const defaultChannels = (): ChannelState[] =>
  Array.from({ length: CHANNEL_COUNT }, () => ({ gainDb: 0, muted: false }));

const defaultPan = (): number[] =>
  Array.from({ length: CHANNEL_COUNT }, () => 0);

export default function App() {
  // Token de acesso armazenado apenas em estado React — nunca em localStorage
  const [token, setToken] = useState<string | null>(null);
  const [loginError, setLoginError] = useState<string | null>(null);
  const [channels, setChannels] = useState<ChannelState[]>(defaultChannels);
  // masterGainDb é read-only para músico — derivado do snapshot, não de estado local

  // Pan por canal: -1.0 (esquerda) a +1.0 (direita)
  const [panByChannel, setPanByChannel] = useState<number[]>(defaultPan);
  const [channelNames, setChannelNames] = useState<string[]>([]);
  const [scenes, setScenes] = useState<SceneSummary[]>([]);
  const [activeSceneId, setActiveSceneId] = useState<string | null>(null);
  const [scenesLoading, setScenesLoading] = useState(false);
  const [scenesError, setScenesError] = useState<string | null>(null);
  const [presets, setPresets] = useState<import('./api/presets').PresetSummary[]>([]);
  const [presetsLoading, setPresetsLoading] = useState(false);
  const [presetsError, setPresetsError] = useState<string | null>(null);
  const presetsRequestRef = useRef(0);
  const scenesRequestRef = useRef(0);

  const ws = useWebSocket(token);

  // Leitura somente: master_gain_db e master_muted vêm do servidor (Músico não pode mutar master)
  const masterGainDb = ws.snapshot?.mixes[0]?.master_gain_db ?? 0;
  const masterMuted = ws.snapshot?.mixes[0]?.master_muted ?? false;

  useEffect(() => {
    if (!ws.snapshot) return;
    const mix = ws.snapshot.mixes[0];
    if (!mix) return;
    const sendsByChannel = new Map(mix.sends.map((send) => [send.channel_index, send]));
    setChannels((current) => current.map((channel, index) => {
      const send = sendsByChannel.get(index);
      return send ? { gainDb: send.gain_db, muted: send.muted } : channel;
    }));
    // Sincroniza pan de cada canal a partir do snapshot
    setPanByChannel((current) => current.map((p, index) => {
      const send = sendsByChannel.get(index);
      return send !== undefined ? send.pan : p;
    }));
    // master_gain_db derivado diretamente de ws.snapshot — sem estado local
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

  useEffect(() => {
    if (!token) return;
    let active = true;
    void fetchChannelMetadata(token).then((metadata) => {
      if (!active) return;
      const names: string[] = [];
      metadata.forEach(({ index, name }) => { names[index] = name; });
      setChannelNames(names);
    }).catch(() => {
      if (active) setChannelNames([]);
    });
    return () => { active = false; };
  }, [token]);

  const refreshPresets = useCallback((accessToken: string) => {
    const requestId = ++presetsRequestRef.current;
    setPresetsLoading(true);
    setPresetsError(null);
    void fetchPresets(accessToken).then((items) => {
      if (requestId === presetsRequestRef.current) setPresets(items);
    }).catch((err) => {
      if (requestId === presetsRequestRef.current) {
        setPresetsError(err instanceof Error ? err.message : 'Falha ao carregar presets');
      }
    }).finally(() => {
      if (requestId === presetsRequestRef.current) setPresetsLoading(false);
    });
  }, []);

  const refreshScenes = useCallback((accessToken: string) => {
    const requestId = ++scenesRequestRef.current;
    setScenesLoading(true);
    setScenesError(null);
    void Promise.all([fetchScenes(accessToken), fetchActiveSceneId(accessToken)]).then(([items, current]) => {
      if (requestId !== scenesRequestRef.current) return;
      setScenes(items); setActiveSceneId(current);
    }).catch((err) => {
      if (requestId === scenesRequestRef.current) {
        setScenesError(err instanceof Error ? err.message : 'Falha ao carregar cenas');
      }
    }).finally(() => {
      if (requestId === scenesRequestRef.current) setScenesLoading(false);
    });
  }, []);

  useEffect(() => {
    if (!token) return;
    setScenesLoading(true);
    setScenesError(null);
    setScenes([]);
    setActiveSceneId(null);
    setPresets([]);
    setPresetsLoading(true);
    setPresetsError(null);
    refreshScenes(token);
    refreshPresets(token);
  }, [token, refreshPresets, refreshScenes]);

  const handleLogout = useCallback(async () => {
    ++scenesRequestRef.current;
    ++presetsRequestRef.current;
    ws.disconnect();
    setToken(null);
    setChannels(defaultChannels());
    setPanByChannel(defaultPan());
    setChannelNames([]);
    setScenes([]);
    setActiveSceneId(null);
    setPresets([]);
    setPresetsLoading(false);
    setPresetsError(null);
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

  const handleChannelPan = useCallback(
    (ch: number, pan: number) => {
      const mixIndex = ws.snapshot?.mixes[0]?.index;
      if (mixIndex === undefined) return;
      setPanByChannel((prev) => prev.map((p, i) => (i === ch ? pan : p)));
      ws.send({ type: 'SetSendPan', data: { mix_index: mixIndex, channel_index: ch, pan } });
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
        channelNames={channelNames}
        masterGainDb={masterGainDb}
        masterMuted={masterMuted}
        panByChannel={panByChannel}
        onChannelGain={handleChannelGain}
        onChannelMute={handleChannelMute}
        onChannelPan={handleChannelPan}
        onLogout={handleLogout}
      />
      <SceneList scenes={scenes} activeSceneId={activeSceneId} loading={scenesLoading} error={scenesError} onRefresh={() => refreshScenes(token)} />
      <PresetList presets={presets} loading={presetsLoading} error={presetsError} onRefresh={() => refreshPresets(token)} />
    </div>
  );
}
