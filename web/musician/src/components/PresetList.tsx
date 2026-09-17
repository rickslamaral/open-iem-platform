import styles from './SceneList.module.css';
import type { PresetSummary } from '../api/presets';

interface Props {
  presets: PresetSummary[];
  loading: boolean;
  error: string | null;
  onRefresh: () => void;
}

export function PresetList({ presets, loading, error, onRefresh }: Props) {
  return <section className={styles.panel} aria-labelledby="presets-title">
    <div className={styles.heading}>
      <h2 id="presets-title">Presets</h2>
      <button type="button" onClick={onRefresh} disabled={loading} aria-label="Atualizar presets">
        {loading ? 'Atualizando…' : 'Atualizar'}
      </button>
    </div>
    <p>Catálogo somente leitura. Aplicação de preset permanece indisponível.</p>
    {loading && <p role="status">Carregando presets…</p>}
    {error && <p role="alert">{error}</p>}
    {!loading && !error && presets.length === 0 && <p>Nenhum preset disponível.</p>}
    {!loading && !error && presets.length > 0 && <ul className={styles.list}>
      {presets.map((preset) => <li key={preset.id}>
        <span><strong>{preset.name}</strong><br /><small>{preset.kind} · {preset.description}</small></span>
        <small>somente leitura</small>
      </li>)}
    </ul>}
  </section>;
}
