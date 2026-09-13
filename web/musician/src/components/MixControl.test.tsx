import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { MixControl } from './MixControl';
import type { UseWebSocketResult } from '../hooks/useWebSocket';

const mockWs: UseWebSocketResult = {
  status: 'connected',
  revision: 5,
  snapshot: null,
  error: null,
  send: vi.fn(),
  disconnect: vi.fn(),
};

const defaults = {
  ws: mockWs,
  channels: Array.from({ length: 8 }, () => ({ gainDb: 0, muted: false })),
  masterGainDb: 0,
  masterMuted: false,
  panByChannel: Array.from({ length: 8 }, () => 0),
  onChannelGain: vi.fn(),
  onChannelMute: vi.fn(),
  onChannelPan: vi.fn(),
  onMasterGain: vi.fn(),
  onLogout: vi.fn(),
};

describe('MixControl', () => {
  it('renders 8 channels', () => {
    render(<MixControl {...defaults} />);
    expect(screen.getAllByTestId(/^channel-/).length).toBe(8);
  });

  it('renders master volume slider', () => {
    render(<MixControl {...defaults} />);
    expect(screen.getByLabelText(/master volume/i)).toBeTruthy();
  });

  it('shows logout button', () => {
    render(<MixControl {...defaults} />);
    expect(screen.getByRole('button', { name: /log out/i })).toBeTruthy();
  });

  it('shows ws error when present', () => {
    render(<MixControl {...defaults} ws={{ ...mockWs, error: 'Connection lost' }} />);
    expect(screen.getByRole('alert')).toBeTruthy();
    expect(screen.getByText(/connection lost/i)).toBeTruthy();
  });

  it('shows MASTER MUTED badge when masterMuted is true', () => {
    render(<MixControl {...defaults} masterMuted={true} />);
    expect(screen.getByText('MASTER MUTED')).toBeTruthy();
  });

  it('does not show MASTER MUTED badge when masterMuted is false', () => {
    render(<MixControl {...defaults} masterMuted={false} />);
    expect(screen.queryByText('MASTER MUTED')).toBeNull();
  });
});
