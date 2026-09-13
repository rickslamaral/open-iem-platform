import styles from './MixControl.module.css';
import { Channel } from './Channel';
import type { UseWebSocketResult } from '../hooks/useWebSocket';

const CHANNEL_COUNT = 8;
const DEFAULT_NAMES = [
  'Vocal', 'Guitar', 'Bass', 'Keys',
  'Drums L', 'Drums R', 'Click', 'Aux',
];

interface ChannelState {
  gainDb: number;
  muted: boolean;
}

interface Props {
  ws: UseWebSocketResult;
  channels: ChannelState[];
  masterGainDb: number;
  // TODO: sem mensagem SetMasterMuted no protocolo — exibição somente leitura
  masterMuted: boolean;
  panByChannel: number[];
  onChannelGain: (ch: number, gainDb: number) => void;
  onChannelMute: (ch: number, muted: boolean) => void;
  onChannelPan: (ch: number, pan: number) => void;
  onMasterGain: (gainDb: number) => void;
  onLogout: () => void;
}

export function MixControl({
  ws,
  channels,
  masterGainDb,
  masterMuted,
  panByChannel,
  onChannelGain,
  onChannelMute,
  onChannelPan,
  onMasterGain,
  onLogout,
}: Props) {
  return (
    <div className={styles.container}>
      <header className={styles.header}>
        <span className={styles.appName}>Open IEM</span>
        <div className={styles.headerRight}>
          {ws.error && (
            <span role="alert" className={styles.wsError}>
              {ws.error}
            </span>
          )}
          <button className={styles.logoutBtn} onClick={onLogout} aria-label="Log out">
            Logout
          </button>
        </div>
      </header>

      <div className={styles.master}>
        <div className={styles.masterRow}>
          <label className={styles.masterLabel}>
            Master Volume: {masterGainDb.toFixed(1)} dB
          </label>
          {/* Badge somente leitura — estado mudo master vindo do servidor */}
          {masterMuted && (
            <span className={styles.masterMutedBadge} role="status" aria-label="Master muted">
              MASTER MUTED
            </span>
          )}
        </div>
        <input
          type="range"
          min={-60}
          max={6}
          step={0.5}
          value={masterGainDb}
          onChange={(e) => onMasterGain(parseFloat(e.target.value))}
          className={styles.masterSlider}
          aria-label="Master volume"
        />
      </div>

      <div className={styles.channels}>
        {Array.from({ length: CHANNEL_COUNT }, (_, i) => (
          <Channel
            key={i}
            index={i}
            name={DEFAULT_NAMES[i] ?? `CH${i + 1}`}
            gainDb={channels[i]?.gainDb ?? 0}
            muted={channels[i]?.muted ?? false}
            pan={panByChannel[i] ?? 0}
            onGainChange={onChannelGain}
            onMuteToggle={onChannelMute}
            onPanChange={onChannelPan}
          />
        ))}
      </div>
    </div>
  );
}
