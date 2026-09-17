import styles from './SceneList.module.css';
import type { SceneSummary } from '../api/scenes';

interface Props { scenes: SceneSummary[]; activeSceneId: string | null; loading: boolean; error: string | null }

export function SceneList({ scenes, activeSceneId, loading, error }: Props) {
  return <section className={styles.panel} aria-labelledby="scenes-title">
    <h2 id="scenes-title">Cenas</h2>
    {loading && <p role="status">Carregando cenas…</p>}
    {error && <p role="alert">{error}</p>}
    {!loading && !error && scenes.length === 0 && <p>Nenhuma cena disponível.</p>}
    {!loading && !error && scenes.length > 0 && <ul className={styles.list}>
      {scenes.map((scene) => <li key={scene.id} className={scene.id === activeSceneId ? styles.active : undefined}>
        <span>{scene.name}</span><small>revisão {scene.activeRevision}{scene.id === activeSceneId ? ' · ativa' : ''}</small>
      </li>)}
    </ul>}
  </section>;
}
