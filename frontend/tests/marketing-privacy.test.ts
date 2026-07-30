import { cleanup, render, screen } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import PrivacyPage from '../src/routes/(marketing)/privacy/+page.svelte';
import DocsHub from '../src/routes/docs/+page.svelte';

const values = new Map<string, string>();
vi.stubGlobal('localStorage', {
  getItem: (key: string) => values.get(key) ?? null,
  setItem: (key: string, value: string) => values.set(key, value),
  removeItem: (key: string) => values.delete(key),
  clear: () => values.clear()
});

describe('marketing privacy and docs hub', () => {
  beforeEach(() => {
    cleanup();
    values.clear();
  });

  it('explains cookieless defaults and non-goals', () => {
    render(PrivacyPage);
    expect(screen.getByRole('heading', { name: /cookieless by default/i })).toBeInTheDocument();
    expect(screen.getAllByText(/no session replay/i).length).toBeGreaterThan(0);
    expect(screen.getByRole('link', { name: /create free account/i })).toHaveAttribute(
      'href',
      '/register'
    );
  });

  it('links docs hub to CLI and API references', () => {
    render(DocsHub);
    expect(screen.getByRole('link', { name: /open cli documentation/i })).toHaveAttribute(
      'href',
      '/docs/cli'
    );
    expect(screen.getByRole('link', { name: /open api reference/i })).toHaveAttribute(
      'href',
      '/docs/api'
    );
  });
});
