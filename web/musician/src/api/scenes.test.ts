import { beforeEach, describe, expect, it, vi } from 'vitest';
import { fetchActiveSceneId, fetchScenes } from './scenes';

describe('scene API', () => {
  beforeEach(() => vi.restoreAllMocks());
  it('fetches authenticated summaries and active scene', async () => {
    const fetchMock = vi.spyOn(globalThis, 'fetch')
      .mockResolvedValueOnce(new Response(JSON.stringify({ scenes: [{ id: 's1', name: 'Culto', active_revision: 2, created_at: 1, updated_at: 2 }] }), { status: 200 }))
      .mockResolvedValueOnce(new Response(JSON.stringify({ scene: { id: 's1', name: 'Culto', revision: 2, schema_version: 1, config: { channels: [], mixes: [] } } }), { status: 200 }));
    await expect(fetchScenes('token')).resolves.toEqual([{ id: 's1', name: 'Culto', activeRevision: 2, updatedAt: 2 }]);
    await expect(fetchActiveSceneId('token')).resolves.toBe('s1');
    expect(fetchMock).toHaveBeenNthCalledWith(1, '/api/v1/scenes', expect.objectContaining({ headers: { Authorization: 'Bearer token' } }));
    expect(fetchMock).toHaveBeenNthCalledWith(2, '/api/v1/scenes/active', expect.objectContaining({ headers: { Authorization: 'Bearer token' } }));
  });
  it('fails closed on invalid response and accepts no active scene', async () => {
    vi.spyOn(globalThis, 'fetch')
      .mockResolvedValueOnce(new Response(JSON.stringify({ scenes: [{ id: 'bad', name: 'x', active_revision: 0, updated_at: 1 }] }), { status: 200 }))
      .mockResolvedValueOnce(new Response(JSON.stringify({ scene: null }), { status: 200 }));
    await expect(fetchScenes('token')).resolves.toEqual([]);
    await expect(fetchActiveSceneId('token')).resolves.toBeNull();
  });
  it('rejects HTTP errors', async () => {
    vi.spyOn(globalThis, 'fetch').mockResolvedValue(new Response('', { status: 403 }));
    await expect(fetchScenes('token')).rejects.toThrow('Scenes failed: 403');
  });
});
