<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { env } from '$env/dynamic/public';
  import { BarChart3, CircleDot, Eye, EyeOff, Zap } from '@lucide/svelte';
  import { ApiClient } from '$lib/api';

  type AuthMode = 'login' | 'register';

  let { mode = 'login' as AuthMode }: { mode?: AuthMode } = $props();

  const demo = env.PUBLIC_DEMO_MODE === 'true';
  const api = new ApiClient(env.PUBLIC_API_BASE_URL || '/api', fetch, demo);

  let email = $state('');
  let password = $state('');
  let name = $state('');
  let passwordVisible = $state(false);
  let authError = $state('');
  let authBusy = $state(false);

  onMount(() => {
    const token = localStorage.getItem('slimlytics_token') ?? '';
    if (token || demo) void goto('/app');
  });

  async function authenticate() {
    authBusy = true;
    authError = '';
    try {
      const response =
        mode === 'login'
          ? await api.login(email, password)
          : await api.register(email, password, name);
      const token = response.accessToken ?? response.token ?? '';
      if (!token) throw new Error('The server did not return an access token.');
      localStorage.setItem('slimlytics_token', token);
      api.setToken(token);
      await goto('/app');
    } catch (reason) {
      authError = reason instanceof Error ? reason.message : 'Could not sign in.';
    } finally {
      authBusy = false;
    }
  }

  async function exploreDemo() {
    localStorage.setItem('slimlytics_token', 'demo');
    api.setToken('demo');
    await goto('/app');
  }
</script>

<main id="main" class="auth-shell">
  <section class="auth-pitch">
    <a class="brand" href="/" aria-label="Slimlytics home">
      <span class="brand-mark"><BarChart3 size={21} /></span>Slimlytics
    </a>
    <div>
      <p class="eyebrow">Privacy-first web analytics</p>
      <h1>Know what works.<br /><em>Skip the noise.</em></h1>
      <p>
        Focused traffic intelligence, live visitor activity, and goals — without invasive profiles or
        an overgrown interface.
      </p>
      <ul>
        <li><CircleDot size={15} /> Cookieless by default</li>
        <li><CircleDot size={15} /> Self-hosted and fast</li>
        <li><CircleDot size={15} /> Every metric in one glance</li>
      </ul>
    </div>
    <footer>Independent analytics for independent teams.</footer>
  </section>

  <section class="mobile-intro" aria-labelledby="mobile-intro-title">
    <a class="brand" href="/" aria-label="Slimlytics home">
      <span class="brand-mark"><BarChart3 size={20} /></span>Slimlytics
    </a>
    <div>
      <p class="eyebrow">Privacy-first web analytics</p>
      <h1 id="mobile-intro-title">Private analytics without the clutter.</h1>
      <p>Clear traffic insights. No invasive profiles.</p>
    </div>
    <ul aria-label="Slimlytics benefits">
      <li><CircleDot size={13} /> Cookieless by default</li>
      <li><Zap size={13} /> Live, focused insights</li>
    </ul>
  </section>

  <section class="auth-card" aria-labelledby="auth-title">
    <div class="auth-card-inner">
      <p class="eyebrow">{mode === 'login' ? 'Welcome back' : 'Start measuring'}</p>
      <h2 id="auth-title">
        {mode === 'login' ? 'Sign in to your workspace' : 'Create your account'}
      </h2>
      <p class="muted">
        {mode === 'login' ? 'Your sites are waiting.' : 'No card. No tracking cookies.'}
      </p>
      <form
        onsubmit={(event) => {
          event.preventDefault();
          void authenticate();
        }}
      >
        {#if mode === 'register'}
          <label>Full name<input bind:value={name} autocomplete="name" required /></label>
        {/if}
        <label
          >Email address<input
            type="email"
            bind:value={email}
            autocomplete="email"
            autocapitalize="none"
            spellcheck="false"
            inputmode="email"
            placeholder="you@company.com"
            required
          /></label
        >
        <div class="form-field">
          <label for="auth-password">Password</label>
          <span class="password-field">
            <input
              id="auth-password"
              type={passwordVisible ? 'text' : 'password'}
              bind:value={password}
              autocomplete={mode === 'login' ? 'current-password' : 'new-password'}
              minlength="12"
              required
            />
            <button
              class="password-toggle"
              type="button"
              aria-label={passwordVisible ? 'Hide password' : 'Show password'}
              aria-pressed={passwordVisible}
              onclick={() => (passwordVisible = !passwordVisible)}
            >
              {#if passwordVisible}<EyeOff size={18} />{:else}<Eye size={18} />{/if}
            </button>
          </span>
        </div>
        {#if authError}
          <div class="alert" role="alert">{authError}</div>
        {/if}
        <button class="primary wide" disabled={authBusy}>
          {authBusy ? 'Please wait…' : mode === 'login' ? 'Sign in' : 'Create account'}
        </button>
      </form>
      <p class="auth-switch">
        {mode === 'login' ? 'New to Slimlytics?' : 'Already have an account?'}
        <a href={mode === 'login' ? '/register' : '/login'}>
          {mode === 'login' ? 'Create an account' : 'Sign in'}
        </a>
      </p>
      {#if demo}
        <div class="divider"><span>or</span></div>
        <button class="secondary wide" onclick={() => void exploreDemo()}>
          Explore the demo dashboard
        </button>
      {/if}
      <nav class="auth-links" aria-label="Product documentation">
        <a href="/docs">Documentation</a>
        <span aria-hidden="true">·</span>
        <a href="/docs/cli">CLI</a>
        <span aria-hidden="true">·</span>
        <a href="/docs/api">API</a>
      </nav>
    </div>
  </section>
</main>
