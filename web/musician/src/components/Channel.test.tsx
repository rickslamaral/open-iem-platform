import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Channel } from './Channel';

const defaults = {
  index: 0,
  name: 'Vocal',
  gainDb: 0,
  muted: false,
  pan: 0,
  onGainChange: vi.fn(),
  onMuteToggle: vi.fn(),
  onPanChange: vi.fn(),
};

describe('Channel', () => {
  it('renders channel number and name', () => {
    render(<Channel {...defaults} />);
    expect(screen.getByText('CH01')).toBeTruthy();
    expect(screen.getByText('Vocal')).toBeTruthy();
  });

  it('renders gain slider with correct value', () => {
    render(<Channel {...defaults} gainDb={-6} />);
    const sliders = screen.getAllByRole('slider');
    const gainSlider = sliders.find((s) => (s as HTMLInputElement).getAttribute('aria-label') === 'Vocal gain');
    expect((gainSlider as HTMLInputElement).value).toBe('-6');
  });

  it('calls onGainChange when slider changes', () => {
    const onGainChange = vi.fn();
    render(<Channel {...defaults} onGainChange={onGainChange} />);
    const gainSlider = screen.getByLabelText('Vocal gain');
    fireEvent.change(gainSlider, { target: { value: '-12' } });
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
    const sliders = screen.getAllByRole('slider');
    sliders.forEach((s) => expect((s as HTMLInputElement).disabled).toBe(true));
  });

  it('renders pan slider with correct value', () => {
    render(<Channel {...defaults} pan={0.5} />);
    const panSlider = screen.getByLabelText('Vocal pan');
    expect((panSlider as HTMLInputElement).value).toBe('0.5');
  });

  it('calls onPanChange when pan slider changes', () => {
    const onPanChange = vi.fn();
    render(<Channel {...defaults} onPanChange={onPanChange} />);
    const panSlider = screen.getByLabelText('Vocal pan');
    fireEvent.change(panSlider, { target: { value: '-0.5' } });
    expect(onPanChange).toHaveBeenCalledWith(0, -0.5);
  });

  it('shows pan label C when pan is 0', () => {
    render(<Channel {...defaults} pan={0} />);
    expect(screen.getByText('C')).toBeTruthy();
  });

  it('shows pan label with L when pan is negative', () => {
    render(<Channel {...defaults} pan={-0.5} />);
    expect(screen.getByText('-0.50 L')).toBeTruthy();
  });

  it('shows pan label with R when pan is positive', () => {
    render(<Channel {...defaults} pan={0.3} />);
    expect(screen.getByText('+0.30 R')).toBeTruthy();
  });

  it('disables pan slider when muted', () => {
    render(<Channel {...defaults} muted={true} pan={0.2} />);
    const panSlider = screen.getByLabelText('Vocal pan');
    expect((panSlider as HTMLInputElement).disabled).toBe(true);
  });
});
