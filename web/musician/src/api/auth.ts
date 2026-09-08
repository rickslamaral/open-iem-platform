export interface LoginRequest {
  username: string;
  password: string;
}

export interface TokenResponse {
  access_token: string;
  token_type: 'Bearer';
  expires_in: number;
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

/** POST /api/v1/auth/logout */
export async function logout(): Promise<void> {
  await fetch('/api/v1/auth/logout', {
    method: 'POST',
    credentials: 'include',
  });
}
