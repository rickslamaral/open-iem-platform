export interface PresetSummary {
  id: string;
  name: string;
  kind: string;
  description: string;
}

interface PresetsResponse { presets: unknown }

function parsePreset(value: unknown): PresetSummary | null {
  if (typeof value !== 'object' || value === null) return null;
  const item = value as Record<string, unknown>;
  if (typeof item.id !== 'string' || typeof item.name !== 'string' ||
      typeof item.kind !== 'string' || typeof item.description !== 'string') return null;
  return { id: item.id, name: item.name, kind: item.kind, description: item.description };
}

export async function fetchPresets(token: string): Promise<PresetSummary[]> {
  const response = await fetch('/api/v1/presets', {
    headers: { Authorization: 'Bearer ' + token },
    credentials: 'include',
  });
  if (!response.ok) throw new Error(`Presets failed: ${response.status}`);
  const payload = await response.json() as PresetsResponse;
  if (typeof payload !== 'object' || payload === null || !Array.isArray(payload.presets)) {
    throw new Error('Invalid presets response');
  }
  return payload.presets.flatMap((preset) => {
    const parsed = parsePreset(preset);
    return parsed ? [parsed] : [];
  });
}
