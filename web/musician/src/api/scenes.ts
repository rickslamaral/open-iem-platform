export interface SceneSummary {
  id: string;
  name: string;
  activeRevision: number;
  updatedAt: number;
}

interface ScenesResponse { scenes: unknown }
interface ActiveResponse { scene: unknown | null }

function parseSummary(value: unknown): SceneSummary | null {
  if (typeof value !== 'object' || value === null) return null;
  const item = value as Record<string, unknown>;
  if (typeof item.id !== 'string' || typeof item.name !== 'string' ||
      !Number.isInteger(item.active_revision) || (item.active_revision as number) < 1 ||
      !Number.isInteger(item.updated_at) || (item.updated_at as number) < 0) return null;
  return { id: item.id, name: item.name, activeRevision: item.active_revision as number, updatedAt: item.updated_at as number };
}

async function getJson<T>(path: string, token: string): Promise<T> {
  const response = await fetch(path, { headers: { Authorization: 'Bearer ' + token }, credentials: 'include' });
  if (!response.ok) throw new Error(`Scenes failed: ${response.status}`);
  return await response.json() as T;
}

export async function fetchScenes(token: string): Promise<SceneSummary[]> {
  const payload = await getJson<ScenesResponse>('/api/v1/scenes', token);
  if (typeof payload !== 'object' || payload === null || !Array.isArray(payload.scenes)) throw new Error('Invalid scenes response');
  return payload.scenes.flatMap((scene) => { const parsed = parseSummary(scene); return parsed ? [parsed] : []; });
}

export async function fetchActiveSceneId(token: string): Promise<string | null> {
  const payload = await getJson<ActiveResponse>('/api/v1/scenes/active', token);
  if (typeof payload !== 'object' || payload === null || !('scene' in payload)) throw new Error('Invalid active scene response');
  if (payload.scene === null) return null;
  if (typeof payload.scene !== 'object' || payload.scene === null ||
      typeof (payload.scene as Record<string, unknown>).id !== 'string') {
    throw new Error('Invalid active scene response');
  }
  return (payload.scene as { id: string }).id;
}
