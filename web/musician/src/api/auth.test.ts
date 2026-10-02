import { beforeEach, describe, expect, it, vi } from 'vitest';
import { exchangeQr, refresh } from './auth';

describe('QR auth', () => {
  beforeEach(() => vi.restoreAllMocks());
  it('exchanges QR secret with profile fields and includes cookies', async () => {
    const fetchMock = vi.spyOn(globalThis, 'fetch').mockResolvedValue(new Response(JSON.stringify({ access_token: 'memory-token', role: 'Musician' }), { status: 201 }));
    await expect(exchangeQr({ qr_secret: 'secret', display_name: 'Alex', instrument_id: 'guitar' })).resolves.toEqual({ access_token: 'memory-token', role: 'Musician' });
    expect(fetchMock).toHaveBeenCalledWith('/api/v1/onboarding/qr/exchange', expect.objectContaining({ method: 'POST', credentials: 'include', body: JSON.stringify({ qr_secret: 'secret', display_name: 'Alex', instrument_id: 'guitar' }) }));
    expect(localStorage.length).toBe(0); expect(sessionStorage.length).toBe(0);
  });
  it('restores access token using HttpOnly cookie', async () => {
    const fetchMock = vi.spyOn(globalThis, 'fetch').mockResolvedValue(new Response(JSON.stringify({ access_token: 'restored' }), { status: 200 }));
    await expect(refresh()).resolves.toMatchObject({ access_token: 'restored' });
    expect(fetchMock).toHaveBeenCalledWith('/api/v1/auth/refresh', expect.objectContaining({ method: 'POST', credentials: 'include' }));
  });
  it('reports expired or validation failures without persisting secret', async () => {
    vi.spyOn(globalThis, 'fetch').mockResolvedValue(new Response(JSON.stringify({ message: 'QR secret expired or revoked' }), { status: 401 }));
    await expect(exchangeQr({ qr_secret: 'expired', display_name: 'Alex', instrument_id: 'guitar' })).rejects.toThrow(/expired or revoked/);
    expect(localStorage.length).toBe(0); expect(sessionStorage.length).toBe(0);
  });
});
