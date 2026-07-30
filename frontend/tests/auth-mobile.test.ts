import { cleanup, fireEvent, render, screen, within } from '@testing-library/svelte';
import { readFileSync } from 'node:fs';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import Page from '../src/routes/+page.svelte';

vi.mock('$env/dynamic/public', () => ({ env: {} }));

const values = new Map<string, string>();
const storage = {
  getItem: (key: string) => values.get(key) ?? null,
  setItem: (key: string, value: string) => values.set(key, value),
  removeItem: (key: string) => values.delete(key),
  clear: () => values.clear()
};
vi.stubGlobal('localStorage', storage);

const css = readFileSync('src/app.css', 'utf8');

describe('mobile authentication experience', () => {
  beforeEach(() => { cleanup(); localStorage.clear(); });

  it('keeps the mobile homepage pitch and documentation destinations available', () => {
    render(Page);

    expect(screen.getByRole('heading', { name: /private analytics without the clutter/i })).toBeInTheDocument();
    const mobileIntro = screen.getByRole('region', { name: /private analytics without the clutter/i });
    expect(within(mobileIntro).getByText('Cookieless by default')).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'API reference' })).toHaveAttribute('href', '/docs/api');
  });

  it('lets mobile users reveal and conceal their password accessibly', async () => {
    render(Page);
    const password = screen.getByLabelText('Password');
    const toggle = screen.getByRole('button', { name: 'Show password' });

    expect(password).toHaveAttribute('type', 'password');
    await fireEvent.click(toggle);
    expect(password).toHaveAttribute('type', 'text');
    expect(toggle).toHaveAccessibleName('Hide password');
  });

  it('defines a safe-viewport mobile surface and touch-sized controls', () => {
    expect(css).toMatch(/\.auth-shell\{[^}]*min-height:100svh/);
    expect(css).toMatch(/\.auth-card input[^}]*min-height:52px/);
    expect(css).toMatch(/\.password-toggle[^}]*min-width:48px[^}]*min-height:48px/);
  });
});
