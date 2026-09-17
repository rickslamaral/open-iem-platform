import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { SceneList } from './SceneList';

const scenes = [
  { id: 's1', name: 'Culto', activeRevision: 2, updatedAt: 2 },
  { id: 's2', name: 'Ensaio', activeRevision: 1, updatedAt: 1 },
];

describe('SceneList', () => {
  it('renders active scene and revisions', () => {
    render(<SceneList scenes={scenes} activeSceneId="s1" loading={false} error={null} onRefresh={() => undefined} />);
    expect(screen.getByText('Culto')).toBeVisible();
    expect(screen.getByText('revisão 2 · ativa')).toBeVisible();
    expect(screen.getByText('revisão 1')).toBeVisible();
  });

  it('shows loading, error and empty states', () => {
    const { rerender } = render(<SceneList scenes={[]} activeSceneId={null} loading={true} error={null} onRefresh={() => undefined} />);
    expect(screen.getByRole('status')).toHaveTextContent('Carregando cenas');
    rerender(<SceneList scenes={[]} activeSceneId={null} loading={false} error="Falha" onRefresh={() => undefined} />);
    expect(screen.getByRole('alert')).toHaveTextContent('Falha');
    rerender(<SceneList scenes={[]} activeSceneId={null} loading={false} error={null} onRefresh={() => undefined} />);
    expect(screen.getByText('Nenhuma cena disponível.')).toBeVisible();
  });

  it('refreshes and disables button while loading', async () => {
    const onRefresh = vi.fn();
    const user = userEvent.setup();
    const { rerender } = render(<SceneList scenes={[]} activeSceneId={null} loading={false} error={null} onRefresh={onRefresh} />);
    await user.click(screen.getByRole('button', { name: 'Atualizar cenas' }));
    expect(onRefresh).toHaveBeenCalledOnce();
    rerender(<SceneList scenes={[]} activeSceneId={null} loading={true} error={null} onRefresh={onRefresh} />);
    expect(screen.getByRole('button', { name: 'Atualizar cenas' })).toBeDisabled();
  });
});
