import { beforeEach, describe, expect, it, vi } from 'vitest';
import { fetchPresets } from './presets';

describe('fetchPresets', () => {
  beforeEach(() => vi.restoreAllMocks());

  it('parses valid catalog and discards malformed entries', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(new Response(JSON.stringify({ presets: [
      { id: 'vocal', name: 'Vocal', kind: 'channel', description: 'Neutral' },
      { id: 42 },
    ] }), { status: 200 })));
    await expect(fetchPresets('token')).resolves.toEqual([
      { id: 'vocal', name: 'Vocal', kind: 'channel', description: 'Neutral' },
    ]);
  });

  it('rejects failed and malformed responses', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(new Response('no', { status: 403 })));
    await expect(fetchPresets('token')).rejects.toThrow('Presets failed: 403');
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(new Response(JSON.stringify({ presets: {} }), { status: 200 })));
    await expect(fetchPresets('token')).rejects.toThrow('Invalid presets response');
  });
});
