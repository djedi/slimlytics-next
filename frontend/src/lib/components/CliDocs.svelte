<script lang="ts">
  import { BarChart3, Check, Copy, ExternalLink, ShieldCheck, Terminal } from '@lucide/svelte';
  import { cliCommands, installCommand } from '$lib/cli-docs';

  let copyStatus = $state('');
  async function copyInstall() {
    try {
      if (!navigator.clipboard) throw new Error('Clipboard unavailable');
      await navigator.clipboard.writeText(installCommand);
      copyStatus = 'Install command copied';
    } catch {
      copyStatus = 'Copy failed; select the command manually';
    }
    setTimeout(() => copyStatus = '', 2400);
  }
</script>

<svelte:head>
  <title>Slimlytics CLI documentation</title>
  <meta name="description" content="Install and use the Slimlytics command-line client for account, token, site, and first-party tracking management." />
</svelte:head>

<header class="docs-header">
  <a class="brand" href="/" aria-label="Slimlytics home"><span><BarChart3 size={20}/></span>Slimlytics</a>
  <nav aria-label="Documentation navigation">
    <a class="active" href="/docs/cli">CLI guide</a>
    <a href="/api/docs">Interactive API reference <ExternalLink size={14}/></a>
  </nav>
</header>

<main id="main" class="docs-shell">
  <aside aria-label="On this page">
    <strong>On this page</strong>
    <a href="#install">Install</a>
    <a href="#authenticate">Authenticate</a>
    <a href="#commands">Command reference</a>
    <a href="#automation">Agents and JSON</a>
    <a href="#configuration">Configuration</a>
    <a href="#security">Security</a>
  </aside>

  <article>
    <section class="hero">
      <div class="terminal-icon"><Terminal size={25}/></div>
      <p class="eyebrow">Command-line reference</p>
      <h1>Slimlytics CLI</h1>
      <p class="lede">Manage your account, personal API tokens, sites, and complete first-party tracking setup from a terminal or AI agent.</p>
      <div class="notice"><strong>Why the command is <code>slimlytics</code>, not <code>slim</code>:</strong> SlimToolkit already owns the popular <code>slim</code> command and has more than 23,000 GitHub stars. Avoiding that collision keeps installs predictable.</div>
    </section>

    <section id="install">
      <h2>Install</h2>
      <p>The installer defaults to the pinned <code>cli-v0.2.0</code> release tag and performs a locked Cargo release build. Requirements: Rust/Cargo, curl, tar, and a supported Unix shell.</p>
      <div class="code-block"><code>{installCommand}</code><button onclick={copyInstall} aria-label="Copy install command">{#if copyStatus === 'Install command copied'}<Check size={16}/>{:else}<Copy size={16}/>{/if}</button></div>
      <span class="sr-only" aria-live="polite">{copyStatus}</span>
      <pre><code>slimlytics --version
slimlytics --help</code></pre>
      <p>Install from a local checkout instead:</p>
      <pre><code>cargo install --locked --path cli</code></pre>
      <p>Cargo places the executable in <code>${'{CARGO_HOME:-$HOME/.cargo}'}/bin</code>. Ensure that directory is on <code>PATH</code>.</p>
    </section>

    <section id="authenticate">
      <h2>Authenticate</h2>
      <p>Interactive login exchanges your password for a short-lived session and then creates a durable, revocable personal API token:</p>
      <pre><code>slimlytics auth login --email you@example.com
slimlytics auth status</code></pre>
      <p>For automation, keep passwords and tokens out of process arguments:</p>
      <pre><code>printf '%s\n' "$SLIMLYTICS_PASSWORD" | \
  slimlytics auth login --email you@example.com --password-stdin

printf '%s\n' "$SLIMLYTICS_TOKEN" | \
  slimlytics auth use-token --token-stdin</code></pre>
      <p>Revoke the active personal token while signing out:</p>
      <pre><code>slimlytics auth logout --revoke</code></pre>
    </section>

    <section id="commands">
      <h2>Complete command reference</h2>
      <p>Global options can appear before or after the command: <code>--json</code> emits machine-readable output and <code>--api-url URL</code> selects a self-hosted API.</p>
      <div class="command-list">
        {#each cliCommands as command}
          <section class="command-card">
            <code>{command.usage}</code>
            <h3>{command.summary}</h3>
            <p>{command.details}</p>
            {#if command.options}<ul>{#each command.options as option}<li>{option}</li>{/each}</ul>{/if}
          </section>
        {/each}
      </div>
    </section>

    <section id="automation">
      <h2>AI agents and JSON output</h2>
      <p><code>site ensure</code> is the primary agent workflow. The server performs the ensure transaction atomically, so retries cannot create duplicate domains.</p>
      <pre><code>slimlytics --json site ensure example.com --server caddy</code></pre>
      <p>The versioned response envelope includes <code>schemaVersion</code>, <code>ok</code>, whether the site was created, all site settings, reverse-proxy configuration, the script snippet, verification URLs, and ordered steps.</p>
      <pre><code>{`{
  "schemaVersion": 1,
  "ok": true,
  "data": {
    "created": true,
    "site": { "id": "…", "domain": "example.com" },
    "tracking": {
      "serverConfig": "…",
      "snippet": "<script async src=\"/…js\"><\\/script>",
      "scriptTestUrl": "https://example.com/…js",
      "beaconTestUrl": "https://example.com/…"
    }
  }
}`}</code></pre>
      <p>Successful JSON goes to stdout. Errors go to stderr and return a nonzero exit status. Login, status, and listings never print password or token secrets.</p>
    </section>

    <section id="configuration">
      <h2>Configuration and environment</h2>
      <div class="table-wrap"><table><thead><tr><th>Name</th><th>Purpose</th></tr></thead><tbody>
        <tr><td><code>SLIMLYTICS_API_URL</code></td><td>API base URL. Defaults to <code>https://slimlytics.com</code>. Remote URLs must use HTTPS; HTTP is accepted only for loopback development.</td></tr>
        <tr><td><code>SLIMLYTICS_TOKEN</code></td><td>Use a personal API token directly instead of the saved credential file.</td></tr>
        <tr><td><code>SLIMLYTICS_CLI_REF</code></td><td>Installer source tag override. The standard installer pins <code>cli-v0.2.0</code>.</td></tr>
        <tr><td><code>CARGO_HOME</code></td><td>Controls Cargo's install directory.</td></tr>
      </tbody></table></div>
      <p>The credential file is stored in the platform configuration directory—for example, <code>~/Library/Application Support/slimlytics/auth.json</code> on macOS or <code>~/.config/slimlytics/auth.json</code> on Linux.</p>
    </section>

    <section id="security">
      <h2>Security guarantees</h2>
      <div class="security-grid">
        <div><ShieldCheck/><h3>Protected transport</h3><p>Remote APIs require HTTPS and the HTTP client refuses all redirects, preventing credentials from following downgrade or cross-origin responses.</p></div>
        <div><ShieldCheck/><h3>Private local storage</h3><p>Credential directories use mode 0700 and files mode 0600 on Unix. Writes are atomic and refuse symbolic-link targets.</p></div>
        <div><ShieldCheck/><h3>Revocable tokens</h3><p>Personal tokens are random, hashed at rest, expire, and can be revoked individually. Plaintext is returned only at creation.</p></div>
      </div>
      <p>The CLI generates configuration but deliberately does not SSH into unrelated servers, reload web servers, or modify website templates. The calling human or agent retains those privileges and rollback responsibilities.</p>
    </section>

    <footer>Need raw HTTP details? Open the <a href="/api/docs">interactive API reference</a> or download <a href="/api/openapi.json">OpenAPI JSON</a>.</footer>
  </article>
</main>

<style>
  :global(body) { background: #f7f8fb; color: #18202f; }
  .docs-header { height: 68px; padding: 0 clamp(20px,5vw,72px); display:flex; align-items:center; justify-content:space-between; border-bottom:1px solid #e3e7ef; background:white; position:sticky; top:0; z-index:10; }
  .brand { display:flex; align-items:center; gap:10px; color:#172033; text-decoration:none; font-weight:760; font-size:18px; }
  .brand span,.terminal-icon { display:grid; place-items:center; color:white; background:#7057e8; border-radius:9px; width:36px; height:36px; }
  nav { display:flex; gap:22px; align-items:center; } nav a { display:flex; gap:5px; align-items:center; color:#596276; text-decoration:none; font-size:14px; font-weight:650; } nav a.active, nav a:hover { color:#6047dc; }
  .docs-shell { max-width:1220px; margin:0 auto; display:grid; grid-template-columns:190px minmax(0,820px); gap:58px; padding:54px 28px 90px; }
  aside { position:sticky; top:100px; align-self:start; display:flex; flex-direction:column; gap:12px; font-size:14px; } aside strong { margin-bottom:7px; } aside a { color:#687185; text-decoration:none; } aside a:hover { color:#6047dc; }
  article>section { padding:35px 0; border-bottom:1px solid #e1e5ec; scroll-margin-top:85px; } .hero { padding-top:8px; }
  .eyebrow { margin:18px 0 6px; color:#7057e8; text-transform:uppercase; letter-spacing:.1em; font-size:12px; font-weight:800; }
  h1 { font-size:clamp(40px,6vw,64px); letter-spacing:-.045em; margin:0 0 14px; } h2 { font-size:29px; letter-spacing:-.025em; margin:0 0 15px; } h3 { font-size:16px; margin:16px 0 6px; } p,li { line-height:1.7; color:#535e72; } .lede { font-size:19px; max-width:680px; }
  .notice { margin-top:25px; padding:16px 18px; background:#f0edff; border:1px solid #dcd5ff; border-radius:10px; color:#41386c; line-height:1.6; }
  pre,.code-block { margin:16px 0; padding:17px 19px; background:#171b27; color:#eef1fa; border-radius:10px; overflow:auto; font-size:13px; line-height:1.65; } pre code { color:inherit; white-space:pre; }
  .code-block { display:flex; justify-content:space-between; gap:20px; align-items:start; } .code-block code { white-space:pre-wrap; } .code-block button { border:0; background:#2c3243; color:white; border-radius:7px; padding:8px; display:grid; cursor:pointer; }
  :not(pre):not(.code-block)>code, p code, td code, .notice code { padding:2px 5px; color:#5d43d5; background:#eeebff; border-radius:4px; font-size:.9em; }
  .command-list { display:grid; gap:13px; } .command-card { background:white; border:1px solid #e0e4eb; border-radius:12px; padding:19px 21px; box-shadow:0 3px 12px rgba(33,42,60,.035); } .command-card>code { display:block; color:#5a40d2; overflow-wrap:anywhere; font-size:13px; } .command-card p,.command-card ul { margin-bottom:0; } .command-card ul { padding-left:20px; }
  .table-wrap { overflow:auto; } table { width:100%; border-collapse:collapse; background:white; } th,td { padding:13px 14px; text-align:left; border:1px solid #e0e4eb; vertical-align:top; } th { background:#f0f2f6; }
  .security-grid { display:grid; grid-template-columns:repeat(3,1fr); gap:13px; } .security-grid div { padding:18px; background:white; border:1px solid #e0e4eb; border-radius:12px; } .security-grid :global(svg) { color:#7057e8; } .security-grid p { font-size:14px; }
  footer { margin-top:35px; padding:22px; border-radius:10px; background:#edeaff; color:#4d4475; } footer a { color:#5d43d5; font-weight:700; }
  @media(max-width:800px){ .docs-header nav a:first-child{display:none}.docs-shell{display:block;padding-top:35px}.docs-shell>aside{display:none}.security-grid{grid-template-columns:1fr}.docs-header{padding:0 18px} }
</style>
