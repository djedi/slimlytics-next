import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import ReportTable from '../src/lib/components/ReportTable.svelte';
import WorldMap from '../src/lib/components/WorldMap.svelte';

describe('analytics components', () => {
  it('renders sortable report semantics and empty state', () => {
    const { rerender } = render(ReportTable, { title: 'Top pages', rows: [{ label: '/docs', value: 42 }] });
    expect(screen.getByRole('table', { name: 'Top pages' })).toBeInTheDocument();
    expect(screen.getByText('/docs')).toBeInTheDocument();
    rerender({ title: 'Top pages', rows: [] });
    expect(screen.getByText(/No report data/i)).toBeInTheDocument();
  });

  it('gives the world visualization a text alternative', () => {
    render(WorldMap, { visitors: [{ country: 'United States', code: 'US', x: 28, y: 42, count: 12 }] });
    expect(screen.getByRole('img', { name: /live visitor world map/i })).toBeInTheDocument();
    expect(screen.getAllByText(/United States: 12/)).toHaveLength(2);
  });
});
