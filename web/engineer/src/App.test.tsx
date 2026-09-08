import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import App from './App';

describe('App (Engineer scaffold)', () => {
  it('renders Engineer Console heading', () => {
    render(<App />);
    expect(screen.getByText(/engineer console/i)).toBeTruthy();
  });

  it('mentions Phase 6', () => {
    render(<App />);
    expect(screen.getAllByText(/phase 6/i).length).toBeGreaterThan(0);
  });
});
