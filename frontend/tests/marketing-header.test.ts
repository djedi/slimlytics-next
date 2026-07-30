import { cleanup, render, screen, waitFor } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import MarketingHeader from '../src/lib/components/marketing/MarketingHeader.svelte';

const values = new Map<string, string>();
vi.stubGlobal('localStorage', {
  getItem: (key: string) => values.get(key) ?? null,
  setItem: (key: string, value: string) => values.set(key, value),
  removeItem: (key: string) => values.delete(key),
  clear: () => values.clear()
});

describe('marketing header', () => {
  beforeEach(() => {
    cleanup();
    values.clear();
  });

  it('links to pricing, privacy, docs, and signup when logged out', () => {
    render(MarketingHeader);
    expect(screen.getByRole('link', { name: 'Pricing' })).toHaveAttribute('href', '/pricing');
    expect(screen.getByRole('link', { name: 'Privacy' })).toHaveAttribute('href', '/privacy');
    expect(screen.getByRole('link', { name: 'Docs' })).toHaveAttribute('href', '/docs');
    expect(screen.getByRole('link', { name: 'Sign in' })).toHaveAttribute('href', '/login');
    expect(screen.getByRole('link', { name: 'Get started' })).toHaveAttribute('href', '/register');
  });

  it('shows open dashboard when a token is present', async () => {
    values.set('slimlytics_token', 'test-token');
    render(MarketingHeader);
    await waitFor(() => {
      expect(screen.getByRole('link', { name: 'Open dashboard' })).toHaveAttribute('href', '/app');
    });
  });
});
