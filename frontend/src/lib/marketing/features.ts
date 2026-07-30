export const productFeatures = [
  {
    title: 'Cookieless by default',
    body: 'Measure traffic without tracking cookies, invasive fingerprints, or form capture.'
  },
  {
    title: 'Real-time Spy',
    body: 'Watch live visitor activity over Server-Sent Events as people move through your site.'
  },
  {
    title: 'Reports that matter',
    body: 'Pages, referrers, countries, devices, and campaigns — ranked and ready to export.'
  },
  {
    title: 'Goals & custom events',
    body: 'Track the actions that matter: signups, downloads, outbound clicks, and more.'
  },
  {
    title: 'First-party proxy',
    body: 'Serve the tracker from your domain with Clicky-style paths so ad blockers interfere less.'
  },
  {
    title: 'Self-host in minutes',
    body: 'Ship with Docker Compose, Caddy, PostgreSQL, and a small Rust API built for speed.'
  }
] as const;

export const howItWorks = [
  {
    step: '1',
    title: 'Create your account',
    body: 'Register in under a minute. No card required to start measuring.'
  },
  {
    step: '2',
    title: 'Add a site & install the snippet',
    body: 'Copy a tiny first-party script — or generate Caddy, Nginx, or Apache proxy config.'
  },
  {
    step: '3',
    title: 'Read live, private insights',
    body: 'Open the dashboard for overview metrics, Spy, reports, and goals.'
  }
] as const;

export const privacyHighlights = [
  'No cross-site tracking or advertising profiles',
  'No session replay or form-field capture',
  'Sensitive query parameters redacted before storage',
  'Do Not Track and Global Privacy Control respected',
  'Site-scoped visitor IDs with rotating secrets'
] as const;
