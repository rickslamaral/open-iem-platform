export interface LoginRequest {
  username: string;
  password: string;
}

export type AuthRole = 'ADMIN' | 'ENGINEER' | 'MUSICIAN';

export interface LoginResponse {
  access_token: string;
  role: AuthRole;
  must_change_password: boolean;
}

export interface RefreshResponse {
  access_token: string;
}

export interface QrExchangeRequest {
  qr_secret: string;
  display_name: string;
  instrument_id: string;
  username?: string;
  password?: string;
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
export async function login(req: LoginRequest): Promise<LoginResponse> {
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
  return res.json() as Promise<LoginResponse>;
}

/** POST /api/v1/auth/refresh — uses httpOnly cookie, no body required */
export async function refresh(): Promise<RefreshResponse> {
  const res = await fetch('/api/v1/auth/refresh', {
    method: 'POST',
    credentials: 'include',
  });
  if (!res.ok) {
    throw new Error(`Refresh failed: ${res.status}`);
  }
  return res.json() as Promise<RefreshResponse>;
}

/** POST /api/v1/auth/logout — revoke current access/refresh session. */
export async function logout(accessToken: string): Promise<void> {
  await fetch('/api/v1/auth/logout', {
    method: 'POST',
    headers: { Authorization: `Bearer ${accessToken}` },
    credentials: 'include',
  });
}
