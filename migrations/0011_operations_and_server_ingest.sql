ALTER TABLE sites
  ADD COLUMN server_write_key uuid NOT NULL UNIQUE DEFAULT gen_random_uuid();

ALTER TABLE events
  ADD COLUMN ingestion_source text NOT NULL DEFAULT 'browser'
    CHECK (ingestion_source IN ('browser', 'server')),
  ADD COLUMN source_event_id text;

CREATE INDEX events_site_source_time_idx
  ON events(site_id, ingestion_source, occurred_at DESC);
CREATE UNIQUE INDEX events_server_source_id_idx
  ON events(site_id, source_event_id) WHERE source_event_id IS NOT NULL;

CREATE TABLE daily_site_rollups (
  site_id uuid NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
  metric_date date NOT NULL,
  page_views bigint NOT NULL,
  visitors bigint NOT NULL,
  sessions bigint NOT NULL,
  custom_events bigint NOT NULL,
  bot_requests bigint NOT NULL,
  ai_crawler_requests bigint NOT NULL,
  revenue numeric(18,4) NOT NULL,
  refreshed_at timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY(site_id, metric_date)
);
CREATE INDEX daily_site_rollups_date_idx
  ON daily_site_rollups(metric_date DESC, site_id);

CREATE TABLE report_subscriptions (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  site_id uuid NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
  created_by uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  name text NOT NULL CHECK (char_length(name) BETWEEN 1 AND 120),
  webhook_url text NOT NULL CHECK (char_length(webhook_url) BETWEEN 10 AND 2048),
  frequency text NOT NULL CHECK (frequency IN ('daily', 'weekly')),
  anomaly_only boolean NOT NULL DEFAULT false,
  enabled boolean NOT NULL DEFAULT true,
  next_run_at timestamptz NOT NULL,
  last_sent_at timestamptz,
  last_status text CHECK (last_status IS NULL OR last_status IN ('success', 'error', 'skipped')),
  last_error text,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(site_id, name)
);
CREATE INDEX report_subscriptions_due_idx
  ON report_subscriptions(next_run_at) WHERE enabled;

CREATE TABLE report_deliveries (
  id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  subscription_id uuid NOT NULL REFERENCES report_subscriptions(id) ON DELETE CASCADE,
  site_id uuid NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
  payload jsonb NOT NULL,
  status text NOT NULL CHECK (status IN ('success', 'error', 'skipped')),
  response_status integer,
  error text,
  created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX report_deliveries_subscription_time_idx
  ON report_deliveries(subscription_id, created_at DESC);
