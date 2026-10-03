export interface BandOption { id: number; name: string }
export async function fetchActiveBands(): Promise<BandOption[]> {
  const res = await fetch('/api/v1/onboarding/bands', { credentials: 'include' });
  if (!res.ok) throw new Error(`Bands failed: ${res.status}`);
  return res.json() as Promise<BandOption[]>;
}
