import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { PresetList } from './PresetList';

const presets = [{ id: 'vocal', name: 'Vocal', kind: 'channel', description: 'Neutral' }];

describe('PresetList', () => {
  it('renders read-only catalog without mutation controls', () => {
    render(<PresetList presets={presets} loading={false} error={null} onRefresh={() => undefined} />);
    expect(screen.getByText('Vocal')).toBeVisible();
    expect(screen.getByText('somente leitura')).toBeVisible();
    expect(screen.queryByRole('button', { name: /aplicar|editar|deletar/i })).toBeNull();
  });

  it('shows loading, error and empty states', () => {
    const { rerender } = render(<PresetList presets={[]} loading={true} error={null} onRefresh={() => undefined} />);
    expect(screen.getByRole('status')).toHaveTextContent('Carregando presets');
    rerender(<PresetList presets={[]} loading={false} error='Falha' onRefresh={() => undefined} />);
    expect(screen.getByRole('alert')).toHaveTextContent('Falha');
    rerender(<PresetList presets={[]} loading={false} error={null} onRefresh={() => undefined} />);
    expect(screen.getByText('Nenhum preset disponível.')).toBeVisible();
  });

  it('refreshes and disables button while loading', async () => {
    const onRefresh = vi.fn();
    const user = userEvent.setup();
    render(<PresetList presets={[]} loading={false} error={null} onRefresh={onRefresh} />);
    await user.click(screen.getByRole('button', { name: 'Atualizar presets' }));
    expect(onRefresh).toHaveBeenCalledOnce();
  });
});
