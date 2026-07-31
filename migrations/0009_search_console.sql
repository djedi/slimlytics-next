CREATE TABLE oauth_states (
  state_hash text PRIMARY KEY,
  user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  site_id uuid NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
  expires_at timestamptz NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX oauth_states_expiry_idx ON oauth_states(expires_at);

CREATE TABLE search_console_integrations (
  site_id uuid PRIMARY KEY REFERENCES sites(id) ON DELETE CASCADE,
  property_url text,
  refresh_token_encrypted text NOT NULL,
  connected_by uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  last_synced_at timestamptz,
  last_error text,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE search_console_metrics (
  site_id uuid NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
  metric_date date NOT NULL,
  query text NOT NULL DEFAULT '',
  page text NOT NULL DEFAULT '',
  country text NOT NULL DEFAULT '',
  device text NOT NULL DEFAULT '',
  clicks double precision NOT NULL,
  impressions double precision NOT NULL,
  ctr double precision NOT NULL,
  position double precision NOT NULL,
  synced_at timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY(site_id,metric_date,query,page,country,device)
);
CREATE INDEX search_console_metrics_site_date_idx
  ON search_console_metrics(site_id,metric_date DESC);
