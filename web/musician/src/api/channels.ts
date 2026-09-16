export interface ChannelMetadata {
  index: number;
  name: string;
}

interface ChannelsResponse {
  channels: unknown;
}

/** Fetch authenticated channel labels. Invalid payloads fail closed to caller. */
export async function fetchChannelMetadata(token: string): Promise<ChannelMetadata[]> {
  const response = await fetch('/api/v1/channels', {
    headers: { Authorization: 'Bearer ' + token },
    credentials: 'include',
  });
  if (!response.ok) throw new Error('Channels failed: ' + response.status);
  const payload = (await response.json()) as ChannelsResponse;
  if (!payload || !Array.isArray(payload.channels)) throw new Error('Invalid channels response');
  const seen = new Set<number>();
  return payload.channels.flatMap((channel) => {
    if (typeof channel !== 'object' || channel === null) return [];
    const value = channel as { index?: unknown; name?: unknown };
    const name = value.name;
    const hasUnpairedSurrogate = typeof name === 'string' &&
      /[\uD800-\uDBFF](?![\uDC00-\uDFFF])|(?<![\uD800-\uDBFF])[\uDC00-\uDFFF]/u.test(name);
    if (typeof value.index !== 'number' || !Number.isInteger(value.index) ||
      value.index < 0 || value.index >= 8 || typeof name !== 'string' ||
      hasUnpairedSurrogate || name.trim().length === 0 ||
      new TextEncoder().encode(name).length > 64 || seen.has(value.index)) return [];
    seen.add(value.index);
    return [{ index: value.index, name }];
  });
}
