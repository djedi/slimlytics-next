ALTER TABLE events
  ADD COLUMN revenue_amount numeric(18,4),
  ADD COLUMN revenue_currency char(3),
  ADD COLUMN content_id text,
  ADD COLUMN content_type text,
  ADD COLUMN content_author text;

CREATE INDEX events_site_content_time_idx ON events(site_id, content_id, occurred_at DESC)
  WHERE content_id IS NOT NULL;
CREATE INDEX events_site_campaign_time_idx
  ON events(site_id, utm_source, utm_medium, utm_campaign, occurred_at DESC);

CREATE TABLE annotations (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  site_id uuid NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
  occurred_on date NOT NULL,
  label text NOT NULL CHECK (char_length(label) BETWEEN 1 AND 240),
  created_by uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX annotations_site_date_idx ON annotations(site_id, occurred_on DESC);

CREATE TABLE funnels (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  site_id uuid NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
  name text NOT NULL CHECK (char_length(name) BETWEEN 1 AND 120),
  steps jsonb NOT NULL CHECK (
    jsonb_typeof(steps) = 'array' AND jsonb_array_length(steps) BETWEEN 2 AND 10
  ),
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(site_id, name)
);
CREATE INDEX funnels_site_idx ON funnels(site_id);
