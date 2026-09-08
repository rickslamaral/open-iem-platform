import styles from './Channel.module.css';

const GAIN_MIN = -60;
const GAIN_MAX = 6;

export interface ChannelProps {
  index: number;
  name: string;
  gainDb: number;
  muted: boolean;
  onGainChange: (channel: number, gainDb: number) => void;
  onMuteToggle: (channel: number, muted: boolean) => void;
}

export function Channel({ index, name, gainDb, muted, onGainChange, onMuteToggle }: ChannelProps) {
  const displayGain = gainDb.toFixed(1);

  return (
    <div className={`${styles.channel} ${muted ? styles.muted : ''}`} data-testid={`channel-${index}`}>
      <div className={styles.header}>
        <span className={styles.number}>CH{String(index + 1).padStart(2, '0')}</span>
        <span className={styles.name}>{name}</span>
        <button
          className={`${styles.muteBtn} ${muted ? styles.muteActive : ''}`}
          onClick={() => onMuteToggle(index, !muted)}
          aria-pressed={muted}
          aria-label={`${muted ? 'Unmute' : 'Mute'} ${name}`}
        >
          {muted ? 'MUTED' : 'MUTE'}
        </button>
      </div>
      <div className={styles.gainRow}>
        <span className={styles.gainLabel}>{displayGain} dB</span>
        <input
          type="range"
          min={GAIN_MIN}
          max={GAIN_MAX}
          step={0.5}
          value={gainDb}
          onChange={(e) => onGainChange(index, parseFloat(e.target.value))}
          className={styles.slider}
          aria-label={`${name} gain`}
          disabled={muted}
        />
      </div>
    </div>
  );
}
