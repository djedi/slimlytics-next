import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/svelte';
import { readFileSync } from 'node:fs';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import AuthForm from '../src/lib/components/AuthForm.svelte';

vi.mock('$env/dynamic/public', () => ({ env: {} }));
vi.mock('$app/navigation', () => ({ goto: vi.fn(() => Promise.resolve()) }));

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
  beforeEach(() => {
    cleanup();
    localStorage.clear();
    vi.clearAllMocks();
  });

  it('keeps the mobile homepage pitch and documentation destinations available', () => {
    render(AuthForm, { mode: 'login' });

    expect(
      screen.getByRole('heading', { name: /private analytics without the clutter/i })
    ).toBeInTheDocument();
    const mobileIntro = screen.getByRole('region', {
      name: /private analytics without the clutter/i
    });
    expect(within(mobileIntro).getByText('Cookieless by default')).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'API' })).toHaveAttribute('href', '/docs/api');
    expect(screen.getByRole('link', { name: 'CLI' })).toHaveAttribute('href', '/docs/cli');
  });

  it('lets mobile users reveal and conceal their password accessibly', async () => {
    render(AuthForm, { mode: 'login' });
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

  it('links between login and register modes', () => {
    render(AuthForm, { mode: 'login' });
    expect(screen.getByRole('link', { name: 'Create an account' })).toHaveAttribute(
      'href',
      '/register'
    );
  });

  it('redirects to the app when a session already exists', async () => {
    const { goto } = await import('$app/navigation');
    values.set('slimlytics_token', 'existing');
    render(AuthForm, { mode: 'login' });
    await waitFor(() => {
      expect(goto).toHaveBeenCalledWith('/app');
    });
  });
});
