import { describe, it, expect } from 'vitest';
import { buildEnvelope, encodeEnvelope } from './envelope';
import { PROTOCOL_VERSION } from './types';

describe('buildEnvelope', () => {
  it('wraps payload with correct version', () => {
    const env = buildEnvelope({ type: 'GetState' });
    expect(env.version).toBe(PROTOCOL_VERSION);
    expect(env.payload).toEqual({ type: 'GetState' });
  });

  it('generates unique request_ids', () => {
    const a = buildEnvelope({ type: 'GetState' });
    const b = buildEnvelope({ type: 'GetState' });
    expect(a.request_id).not.toBe(b.request_id);
  });

  it('request_id is a valid UUID v4', () => {
    const { request_id } = buildEnvelope({ type: 'GetState' });
    expect(request_id).toMatch(
      /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i,
    );
  });
});

describe('encodeEnvelope', () => {
  it('serializes to valid JSON', () => {
    const json = encodeEnvelope({ type: 'SetChannelGain', data: { channel: 0, gain_db: -6.0 } });
    const parsed = JSON.parse(json);
    expect(parsed.version).toBe(PROTOCOL_VERSION);
    expect(parsed.payload.type).toBe('SetChannelGain');
    expect(parsed.payload.data.gain_db).toBe(-6.0);
  });

  it('mute message serializes correctly', () => {
    const json = encodeEnvelope({ type: 'SetChannelMute', data: { channel: 2, muted: true } });
    const parsed = JSON.parse(json);
    expect(parsed.payload.type).toBe('SetChannelMute');
    expect(parsed.payload.data.muted).toBe(true);
  });
});
