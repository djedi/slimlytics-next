import { cleanup, render, screen } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { pricingPlans } from '../src/lib/marketing/pricing';
import Page from '../src/routes/(marketing)/pricing/+page.svelte';

const values = new Map<string, string>();
vi.stubGlobal('localStorage', {
  getItem: (key: string) => values.get(key) ?? null,
  setItem: (key: string, value: string) => values.set(key, value),
  removeItem: (key: string) => values.delete(key),
  clear: () => values.clear()
});

describe('marketing pricing page', () => {
  beforeEach(() => {
    cleanup();
    values.clear();
  });

  it('renders all draft plans with signup CTAs', () => {
    render(Page);

    for (const plan of pricingPlans) {
      expect(screen.getAllByText(plan.name).length).toBeGreaterThan(0);
      expect(screen.getAllByText(plan.price).length).toBeGreaterThan(0);
    }

    const accountLinks = screen.getAllByRole('link', { name: /create account|get started free/i });
    expect(accountLinks.length).toBeGreaterThanOrEqual(3);
    expect(accountLinks.some((link) => link.getAttribute('href')?.startsWith('/register'))).toBe(
      true
    );
  });

  it('states that checkout is not required yet', () => {
    render(Page);
    expect(screen.getAllByText(/checkout is not required/i).length).toBeGreaterThan(0);
  });

  it('shows a feature comparison table', () => {
    render(Page);
    expect(screen.getByRole('table')).toBeInTheDocument();
    expect(screen.getByText(/cookieless tracking/i)).toBeInTheDocument();
  });
});
