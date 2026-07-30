export interface CliCommandDoc {
  usage: string;
  summary: string;
  details: string;
  options?: string[];
}

export const installCommand = `curl --proto '=https' --tlsv1.2 -fsSL \\
  https://raw.githubusercontent.com/djedi/slimlytics-next/cli-v0.2.0/scripts/install-cli.sh | sh`;

export const cliCommands: CliCommandDoc[] = [
  {
    usage: 'slimlytics auth login --email EMAIL [--password-stdin] [--token-name NAME] [--expires-in-days DAYS]',
    summary: 'Sign in and create a durable personal API token.',
    details: 'The password obtains a short-lived session JWT. The CLI exchanges it for a personal API token, stores only that token, and never saves the password.',
    options: ['--password-stdin reads the password without exposing it in process arguments.', '--token-name defaults to slimlytics-cli.', '--expires-in-days accepts 1–3650 and defaults to 365.']
  },
  {
    usage: 'slimlytics auth use-token [--token-stdin]',
    summary: 'Import an existing personal API token.',
    details: 'Reads a slyt_ token from piped standard input, verifies it against the account endpoint, then stores it securely. It does not prompt or accept the token as a process argument; --token-stdin is required when invoking it from a terminal.'
  },
  {
    usage: 'slimlytics auth status',
    summary: 'Verify the saved credential and show the current account.',
    details: 'Returns a nonzero status when the token is absent, expired, revoked, or rejected.'
  },
  {
    usage: 'slimlytics auth logout [--revoke]',
    summary: 'Remove local authentication.',
    details: 'With --revoke, the exact personal token authenticating the request is revoked before the local credential file is removed.'
  },
  {
    usage: 'slimlytics account show',
    summary: 'Show the authenticated account.',
    details: 'Human output shows email and account ID. --json also returns the creation timestamp. Accepts either a saved token or SLIMLYTICS_TOKEN.'
  },
  {
    usage: 'slimlytics token list',
    summary: 'List active personal API tokens.',
    details: 'Human output shows IDs, names, and non-secret prefixes. --json also returns creation, expiration, and last-use timestamps. Token secrets are never listed.'
  },
  {
    usage: 'slimlytics token revoke TOKEN_UUID',
    summary: 'Immediately revoke a personal API token.',
    details: 'Revocation is account-scoped and takes effect on the next request.'
  },
  {
    usage: 'slimlytics site list',
    summary: 'List all sites in the account.',
    details: 'JSON output returns the complete persisted site settings.'
  },
  {
    usage: 'slimlytics site show SITE',
    summary: 'Show one site by UUID or exact domain.',
    details: 'A UUID is recommended for scripts. Ambiguous selectors fail rather than choosing silently.'
  },
  {
    usage: 'slimlytics site add DOMAIN [--name NAME] [--timezone TZ] [--retention-days DAYS] [--origin URL]... [--server SERVER]',
    summary: 'Create a site and return its tracking setup.',
    details: 'SERVER is caddy, nginx, or apache. Repeat --origin to authorize multiple browser origins.'
  },
  {
    usage: 'slimlytics site ensure DOMAIN [--name NAME] [--timezone TZ] [--retention-days DAYS] [--origin URL]... [--server SERVER]',
    summary: 'Atomically create or reuse a canonical domain.',
    details: 'This is the preferred idempotent operation for agents. It always returns the site, tracking snippet, server configuration, test URLs, and ordered installation steps.'
  },
  {
    usage: 'slimlytics site delete SITE --yes',
    summary: 'Delete a site and its analytics data.',
    details: 'The explicit --yes flag prevents accidental interactive or agent deletion.'
  },
  {
    usage: 'slimlytics tracking show SITE',
    summary: 'Generate the current first-party tracking setup.',
    details: 'Returns hardened reverse-proxy configuration, the minimal script tag, and script/beacon verification URLs.'
  },
  {
    usage: 'slimlytics tracking configure SITE --server SERVER [--js-path PATH] [--beacon-path PATH]',
    summary: 'Persist and render first-party tracking settings.',
    details: 'Paths must be same-origin single-segment paths; the JavaScript path must end in .js and must differ from the beacon path.'
  }
];
