import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { Login } from './Login';

describe('Login', () => {
  it('renders username and password fields', () => {
    render(<Login onLogin={vi.fn()} error={null} />);
    expect(screen.getByLabelText(/username/i)).toBeTruthy();
    expect(screen.getByLabelText(/password/i)).toBeTruthy();
  });

  it('renders submit button', () => {
    render(<Login onLogin={vi.fn()} error={null} />);
    expect(screen.getByRole('button', { name: /sign in/i })).toBeTruthy();
  });

  it('calls onLogin with credentials on submit', async () => {
    const onLogin = vi.fn().mockResolvedValue(undefined);
    render(<Login onLogin={onLogin} error={null} />);
    fireEvent.change(screen.getByLabelText(/username/i), { target: { value: 'testuser' } });
    fireEvent.change(screen.getByLabelText(/password/i), { target: { value: 'secret' } });
    fireEvent.click(screen.getByRole('button', { name: /sign in/i }));
    await waitFor(() => expect(onLogin).toHaveBeenCalledWith('testuser', 'secret'));
  });

  it('displays error message when error prop is set', () => {
    render(<Login onLogin={vi.fn()} error="Invalid credentials" />);
    expect(screen.getByRole('alert')).toBeTruthy();
    expect(screen.getByText(/invalid credentials/i)).toBeTruthy();
  });
});
