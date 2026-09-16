import { describe, expect, it, vi, beforeEach } from 'vitest';
import { fetchChannelMetadata } from './channels';

describe('fetchChannelMetadata', () => {
  beforeEach(() => vi.restoreAllMocks());
  it('fetches authenticated channel labels', async () => {
    const fetchMock = vi.spyOn(globalThis, 'fetch').mockResolvedValue(new Response(JSON.stringify({ channels: [{ index: 0, name: 'Lead vocal' }, { index: 7, name: 'Click' }] }), { status: 200 }));
    await expect(fetchChannelMetadata('token-1')).resolves.toEqual([{ index: 0, name: 'Lead vocal' }, { index: 7, name: 'Click' }]);
    expect(fetchMock).toHaveBeenCalledWith('/api/v1/channels', expect.objectContaining({ headers: { Authorization: 'Bearer token-1' } }));
  });
  it('rejects HTTP errors and invalid response shape', async () => {
    vi.spyOn(globalThis, 'fetch').mockResolvedValueOnce(new Response('', { status: 403 }));
    await expect(fetchChannelMetadata('token')).rejects.toThrow('Channels failed: 403');
    vi.spyOn(globalThis, 'fetch').mockResolvedValueOnce(new Response(JSON.stringify({ channels: 'bad' }), { status: 200 }));
    await expect(fetchChannelMetadata('token')).rejects.toThrow('Invalid channels response');
  });
  it('rejects names with unpaired UTF-16 surrogates', async () => {
    vi.spyOn(globalThis, 'fetch').mockResolvedValue(new Response(JSON.stringify({ channels: [{ index: 0, name: String.fromCharCode(0xd800) }] }), { status: 200 }));
    await expect(fetchChannelMetadata('token')).resolves.toEqual([]);
  });

  it('rejects names over 64 UTF-8 bytes', async () => {
    vi.spyOn(globalThis, 'fetch').mockResolvedValue(new Response(JSON.stringify({ channels: [{ index: 0, name: 'á'.repeat(40) }] }), { status: 200 }));
    await expect(fetchChannelMetadata('token')).resolves.toEqual([]);
  });

  it('drops malformed or out-of-range entries', async () => {
    vi.spyOn(globalThis, 'fetch').mockResolvedValue(new Response(JSON.stringify({ channels: [{ index: -1, name: 'bad' }, { index: 8, name: 'bad' }, { index: 1, name: '' }, { index: 2, name: 'Valid' }, { index: 2, name: 'Duplicate' }, null] }), { status: 200 }));
    await expect(fetchChannelMetadata('token')).resolves.toEqual([{ index: 2, name: 'Valid' }]);
  });
});
