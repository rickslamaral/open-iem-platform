import styles from './ConnectionStatus.module.css';
import type { WsStatus } from '../hooks/useWebSocket';

interface Props {
  status: WsStatus;
  revision: number | null;
}

const labels: Record<WsStatus, string> = {
  connected: 'Connected',
  connecting: 'Connecting…',
  disconnected: 'Disconnected',
  error: 'Error',
};

export function ConnectionStatus({ status, revision }: Props) {
  return (
    <div className={styles.container} aria-live="polite">
      <span className={`${styles.dot} ${styles[status]}`} aria-hidden="true" />
      <span className={styles.label}>{labels[status]}</span>
      {revision !== null && (
        <span className={styles.revision} data-testid="revision">
          rev {revision}
        </span>
      )}
    </div>
  );
}
