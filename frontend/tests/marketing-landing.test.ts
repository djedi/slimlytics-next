import { cleanup, render, screen } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import Page from '../src/routes/(marketing)/+page.svelte';

vi.mock('$env/dynamic/public', () => ({ env: {} }));

const values = new Map<string, string>();
vi.stubGlobal('localStorage', {
  getItem: (key: string) => values.get(key) ?? null,
  setItem: (key: string, value: string) => values.set(key, value),
  removeItem: (key: string) => values.delete(key),
  clear: () => values.clear()
});

describe('marketing landing page', () => {
  beforeEach(() => {
    cleanup();
    values.clear();
  });

  it('renders the hero pitch and primary signup CTA', () => {
    render(Page);

    expect(screen.getByRole('heading', { level: 1, name: /know what works/i })).toBeInTheDocument();
    expect(screen.getAllByRole('link', { name: /create free account/i })[0]).toHaveAttribute(
      'href',
      '/register'
    );
    expect(screen.getByRole('link', { name: /view pricing/i })).toHaveAttribute('href', '/pricing');
  });

  it('surfaces product features and privacy highlights', () => {
    render(Page);

    expect(screen.getByRole('heading', { name: /cookieless by default/i })).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: /real-time spy/i })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: /read the privacy model/i })).toHaveAttribute(
      'href',
      '/privacy'
    );
  });

  it('teases pricing plans on the homepage', () => {
    render(Page);
    expect(screen.getByText('Self-hosted')).toBeInTheDocument();
    expect(screen.getByText('Starter')).toBeInTheDocument();
    expect(screen.getByText('Pro')).toBeInTheDocument();
    expect(screen.getByRole('link', { name: /compare plans in detail/i })).toHaveAttribute(
      'href',
      '/pricing'
    );
  });
});
