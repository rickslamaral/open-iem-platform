import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Channel } from './Channel';

const defaults = {
  index: 0,
  name: 'Vocal',
  gainDb: 0,
  muted: false,
  onGainChange: vi.fn(),
  onMuteToggle: vi.fn(),
};

describe('Channel', () => {
  it('renders channel number and name', () => {
    render(<Channel {...defaults} />);
    expect(screen.getByText('CH01')).toBeTruthy();
    expect(screen.getByText('Vocal')).toBeTruthy();
  });

  it('renders gain slider with correct value', () => {
    render(<Channel {...defaults} gainDb={-6} />);
    const slider = screen.getByRole('slider');
    expect((slider as HTMLInputElement).value).toBe('-6');
  });

  it('calls onGainChange when slider changes', () => {
    const onGainChange = vi.fn();
    render(<Channel {...defaults} onGainChange={onGainChange} />);
    fireEvent.change(screen.getByRole('slider'), { target: { value: '-12' } });
    expect(onGainChange).toHaveBeenCalledWith(0, -12);
  });

  it('renders mute button and calls onMuteToggle', () => {
    const onMuteToggle = vi.fn();
    render(<Channel {...defaults} onMuteToggle={onMuteToggle} />);
    fireEvent.click(screen.getByRole('button'));
    expect(onMuteToggle).toHaveBeenCalledWith(0, true);
  });

  it('shows MUTED label when muted', () => {
    render(<Channel {...defaults} muted={true} />);
    expect(screen.getByText('MUTED')).toBeTruthy();
  });

  it('disables slider when muted', () => {
    render(<Channel {...defaults} muted={true} />);
    expect((screen.getByRole('slider') as HTMLInputElement).disabled).toBe(true);
  });
});
