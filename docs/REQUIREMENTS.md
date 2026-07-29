# Version 1 requirements traceability

This matrix distinguishes the implemented Version 1 product from explicitly deferred roadmap items.

| Requirement | Implementation area | Verification |
|---|---|---|
| Multiple sites and site settings | Backend sites API; frontend site management | API integration test; UI test |
| 28-day overview and comparisons | Overview query; dashboard cards/charts | Backend metrics tests; component test |
| Real-time Spy | SSE endpoint; Spy route | SSE test; browser QA |
| Pages/referrers/countries/devices/campaigns | Reporting queries and tables | Report tests; UI tests |
| Visitor timelines and custom events | Visitor/event endpoints | API integration test |
| Basic goals and revenue values | Goals API and UI | API test |
| UTM tracking | Collector and campaign report | Collector/report tests |
| Small first-party tracker | TypeScript tracker build | Bundle build and unit tests |
| SPA, outbound, download, and custom events | Tracker browser hooks | Vitest/jsdom tests |
| Cookieless and consent-aware defaults | Tracker and collector privacy modules | Privacy tests |
| Query redaction and no form capture | Tracker/collector normalization | Regression tests |
| Origin checks, write keys, rate limiting | Collector middleware | Rejection/limit tests |
| Authentication and site ownership | Auth middleware and membership queries | Authorization tests |
| CSV/JSON access | API and CSV endpoint | Content/type/auth tests |
| Real-time updates under normal load | Ingestion event bus and SSE | Integration test and browser QA |
| Docker deployment | Compose, Dockerfiles, Caddy | Container smoke test |
| Backup and restore | Guarded scripts | Backup/restore drill |
| CI and dependency updates | GitHub Actions and Dependabot | Green first CI run |
| Caddy/Nginx/Cloudflare proxy guidance | First-party proxy documentation | Manual config review |

## Deliberately deferred roadmap

These are listed under “Later Features” or “Non-Goals for the MVP” in the source specification and are not falsely advertised as Version 1 functionality:

- Funnels and retention cohorts
- Revenue attribution beyond goal values
- Scheduled reports and configurable external alerts
- Public/password-protected dashboards and expanded team roles
- Data imports
- Heatmaps, uptime monitoring, white-labeling, and plugins
- Session replay, form capture, cross-site identity, ad profiles, and invasive fingerprinting
- Native mobile analytics SDKs
