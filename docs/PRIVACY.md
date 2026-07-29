# Privacy model

Slimlytics is cookieless by default and designed to answer site-analytics questions without building cross-site identity profiles.

## Default behavior

- No cross-site tracking
- No device fingerprinting
- No form-field capture
- No session replay
- Visitor identifiers are site-scoped and derived from privacy-reduced inputs with a rotating server secret
- Sensitive query parameters are removed before URLs are stored
- Do Not Track and Global Privacy Control are respected by the browser tracker
- Collection can be disabled until the host application grants consent

## Sensitive URL data

The collector strips common secret and identity parameters, including names containing:

- email
- password/passwd
- token
- auth
- session
- key/api_key
- code

Site owners should avoid putting personal or secret data into URLs at all. Configure an allowlist when a site uses unusual parameters.

## IP handling

Raw addresses are used only transiently for abuse controls and coarse location derivation when enabled. Persisted visitor identifiers must be one-way, site-scoped, and rotated. Application logs must not include raw authorization tokens, passwords, full IP addresses, or unredacted collector payloads.

## Data rights and retention

Site owners can configure retention, export site data, delete site data, and delete an anonymous visitor’s events. Retention jobs should run regularly and report failures through operational monitoring.

## Consent integration

Initialize the tracker with collection disabled when consent is legally or contractually required. Call the tracker consent API only after the user grants analytics consent. Revocation stops new events immediately; deleting prior data is a separate server-side operation.
