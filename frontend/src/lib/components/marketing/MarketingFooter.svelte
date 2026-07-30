<script lang="ts">
  import { BarChart3 } from '@lucide/svelte';
</script>

<footer class="mkt-footer">
  <div class="mkt-footer-inner">
    <div class="mkt-footer-brand">
      <a class="brand" href="/" aria-label="Slimlytics home">
        <span class="brand-mark"><BarChart3 size={18} /></span>
        Slimlytics
      </a>
      <p>Privacy-first web analytics for independent teams. Self-host free or run hosted plans.</p>
    </div>
    <nav aria-label="Footer product">
      <strong>Product</strong>
      <a href="/#features">Features</a>
      <a href="/pricing">Pricing</a>
      <a href="/privacy">Privacy</a>
      <a href="/register">Create account</a>
    </nav>
    <nav aria-label="Footer documentation">
      <strong>Docs</strong>
      <a href="/docs">Overview</a>
      <a href="/docs/cli">CLI</a>
      <a href="/docs/api">API reference</a>
      <a href="/login">Sign in</a>
    </nav>
  </div>
  <p class="mkt-footer-copy">© {new Date().getFullYear()} Slimlytics. Independent analytics for independent teams.</p>
</footer>
