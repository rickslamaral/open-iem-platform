import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { Login } from './Login';

describe('Login', () => {
  it('renders username and password fields', () => {
    render(<Login onLogin={vi.fn()} onQrExchange={vi.fn()} error={null} />);
    expect(screen.getByLabelText(/username/i)).toBeTruthy();
    expect(screen.getByLabelText(/password/i)).toBeTruthy();
  });

  it('renders submit button', () => {
    render(<Login onLogin={vi.fn()} onQrExchange={vi.fn()} error={null} />);
    expect(screen.getByRole('button', { name: /sign in/i })).toBeTruthy();
  });

  it('calls onLogin with credentials on submit', async () => {
    const onLogin = vi.fn().mockResolvedValue(undefined);
    render(<Login onLogin={onLogin} onQrExchange={vi.fn()} error={null} />);
    fireEvent.change(screen.getByLabelText(/username/i), { target: { value: 'testuser' } });
    fireEvent.change(screen.getByLabelText(/password/i), { target: { value: 'secret' } });
    fireEvent.click(screen.getByRole('button', { name: /sign in/i }));
    await waitFor(() => expect(onLogin).toHaveBeenCalledWith('testuser', 'secret'));
  });

  it('displays error message when error prop is set', () => {
    render(<Login onLogin={vi.fn()} onQrExchange={vi.fn()} error="Invalid credentials" />);
    expect(screen.getByRole('alert')).toBeTruthy();
    expect(screen.getByText(/invalid credentials/i)).toBeTruthy();
  });
  it('consumes invitation from fragment, clears URL, and does not persist secret', async () => {
    window.history.replaceState({}, '', '/musician/#invitation=fragment-secret');
    const onQrExchange = vi.fn().mockResolvedValue(undefined);
    render(<Login onLogin={vi.fn()} onQrExchange={onQrExchange} error={null} />);
    await waitFor(() => expect(screen.getByLabelText(/qr secret/i)).toHaveValue('fragment-secret'));
    expect(window.location.hash).toBe('');
    expect(window.location.search).not.toContain('access_token');
    expect(window.location.search).not.toContain('refresh_token');
    expect(localStorage.length).toBe(0);
    expect(sessionStorage.length).toBe(0);
  });

  it('offers camera scanning and reports unsupported browser', async () => {
    render(<Login onLogin={vi.fn()} onQrExchange={vi.fn()} error={null} />);
    fireEvent.click(screen.getByRole('button', { name: /QR onboarding/i }));
    fireEvent.click(screen.getByRole('button', { name: /scan with camera/i }));
    await waitFor(() => expect(screen.getByRole('status')).toHaveTextContent(/not supported/i));
  });

});
