export interface LoginRequest {
  username: string;
  password: string;
}

export interface TokenResponse {
  access_token: string;
  token_type: 'Bearer';
  expires_in: number;
}

export interface QrExchangeRequest {
  qr_secret: string;
  display_name: string;
  instrument_id: string;
}

export interface QrExchangeResponse {
  access_token: string;
  role: 'Musician';
}

const errorMessage = async (res: Response, operation: string): Promise<Error> => {
  let detail = '';
  try {
    const body: unknown = await res.json();
    if (typeof body === 'object' && body !== null && 'message' in body && typeof body.message === 'string') detail = body.message;
    else if (typeof body === 'object' && body !== null && 'error' in body && typeof body.error === 'string') detail = body.error;
  } catch {
    detail = await res.text().catch(() => '');
  }
  return new Error(`${operation}: ${res.status}${detail ? ` ${detail}` : ''}`);
};

export async function exchangeQr(req: QrExchangeRequest): Promise<QrExchangeResponse> {
  const res = await fetch('/api/v1/onboarding/qr/exchange', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    credentials: 'include',
    body: JSON.stringify(req),
  });
  if (!res.ok) throw await errorMessage(res, 'QR onboarding failed');
  return res.json() as Promise<QrExchangeResponse>;
}

/** POST /api/v1/auth/login — access token stored in-memory only, never localStorage */
export async function login(req: LoginRequest): Promise<TokenResponse> {
  const res = await fetch('/api/v1/auth/login', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    credentials: 'include', // include httpOnly refresh_token cookie
    body: JSON.stringify(req),
  });
  if (!res.ok) {
    const text = await res.text();
    throw new Error(`Login failed: ${res.status} ${text}`);
  }
  return res.json() as Promise<TokenResponse>;
}

/** POST /api/v1/auth/refresh — uses httpOnly cookie, no body required */
export async function refresh(): Promise<TokenResponse> {
  const res = await fetch('/api/v1/auth/refresh', {
    method: 'POST',
    credentials: 'include',
  });
  if (!res.ok) {
    throw new Error(`Refresh failed: ${res.status}`);
  }
  return res.json() as Promise<TokenResponse>;
}

/** POST /api/v1/auth/logout — revoke current access/refresh session. */
export async function logout(accessToken: string): Promise<void> {
  await fetch('/api/v1/auth/logout', {
    method: 'POST',
    headers: { Authorization: `Bearer ${accessToken}` },
    credentials: 'include',
  });
}
