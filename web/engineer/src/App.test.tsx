import { beforeEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import App from './App';

const json = (body: unknown, status = 200) => Promise.resolve({ ok: status < 400, status, text: () => Promise.resolve(JSON.stringify(body)), json: () => Promise.resolve(body) });

beforeEach(() => {
  vi.restoreAllMocks();
  vi.stubGlobal('fetch', vi.fn()
    .mockReturnValueOnce(json({ access_token: 'test-token' }))
    .mockReturnValueOnce(json({ sessions: [] }))
    .mockReturnValueOnce(json([{ mix_index: 0, user_id: 7, username: 'cantor' }]))
    .mockReturnValueOnce(json({ revision: 12 })));
});

describe('Engineer Console', () => {
  it('autentica e mostra estado operacional sem prometer áudio real', async () => {
    render(<App />);
    fireEvent.change(screen.getByLabelText('Usuário'), { target: { value: 'engineer' } });
    fireEvent.change(screen.getByLabelText('Senha'), { target: { value: 'password' } });
    fireEvent.click(screen.getByRole('button', { name: 'Entrar' }));
    expect(await screen.findByRole('heading', { name: 'Engineer Console' })).toBeTruthy();
    expect(screen.getByText('Áudio SIMULATED')).toBeTruthy();
    expect(await screen.findByText('cantor (ID 7)')).toBeTruthy();
    expect(screen.getByText('12')).toBeTruthy();
  });

  it('exibe falha de API ao carregar dashboard', async () => {
    vi.stubGlobal('fetch', vi.fn()
      .mockReturnValueOnce(json({ access_token: 'test-token' }))
      .mockReturnValueOnce(json({}, 503))
      .mockReturnValueOnce(json([]))
      .mockReturnValueOnce(json({ revision: 0 })));
    render(<App />);
    fireEvent.change(screen.getByLabelText('Usuário'), { target: { value: 'engineer' } });
    fireEvent.change(screen.getByLabelText('Senha'), { target: { value: 'password' } });
    fireEvent.click(screen.getByRole('button', { name: 'Entrar' }));
    await waitFor(() => expect(screen.getByRole('alert')).toHaveTextContent('503'));
  });
});
