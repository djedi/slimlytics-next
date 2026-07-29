CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE users (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  email text NOT NULL UNIQUE CHECK (email = lower(email)),
  password_hash text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE sites (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  name text NOT NULL,
  domain text NOT NULL,
  timezone text NOT NULL DEFAULT 'UTC',
  allowed_origins text[] NOT NULL DEFAULT '{}',
  retention_days integer NOT NULL DEFAULT 365 CHECK (retention_days BETWEEN 1 AND 3650),
  write_key uuid NOT NULL UNIQUE DEFAULT gen_random_uuid(),
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TYPE membership_role AS ENUM ('owner', 'admin', 'viewer');
CREATE TABLE site_memberships (
  site_id uuid NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
  user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  role membership_role NOT NULL DEFAULT 'viewer',
  created_at timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (site_id, user_id)
);
CREATE INDEX site_memberships_user_idx ON site_memberships(user_id, site_id);

CREATE TYPE traffic_class AS ENUM ('human', 'bot', 'internal');
CREATE TABLE events (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  site_id uuid NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
  occurred_at timestamptz NOT NULL DEFAULT now(),
  received_at timestamptz NOT NULL DEFAULT now(),
  visitor_id text NOT NULL,
  session_id text NOT NULL,
  event_name text NOT NULL DEFAULT 'pageview',
  url text NOT NULL,
  path text NOT NULL,
  referrer text,
  referrer_host text,
  title text,
  country_code char(2),
  device_type text,
  browser text,
  os text,
  utm_source text,
  utm_medium text,
  utm_campaign text,
  utm_term text,
  utm_content text,
  properties jsonb NOT NULL DEFAULT '{}',
  traffic_class traffic_class NOT NULL DEFAULT 'human',
  CHECK (jsonb_typeof(properties) = 'object')
);
CREATE INDEX events_site_time_idx ON events(site_id, occurred_at DESC);
CREATE INDEX events_site_path_time_idx ON events(site_id, path, occurred_at DESC);
CREATE INDEX events_site_visitor_time_idx ON events(site_id, visitor_id, occurred_at DESC);
CREATE INDEX events_site_session_idx ON events(site_id, session_id);
CREATE INDEX events_site_event_time_idx ON events(site_id, event_name, occurred_at DESC);
CREATE INDEX events_properties_gin_idx ON events USING gin(properties);

CREATE TABLE goals (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  site_id uuid NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
  name text NOT NULL,
  event_name text NOT NULL,
  path_pattern text,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(site_id, name)
);
CREATE INDEX goals_site_idx ON goals(site_id);

CREATE TABLE goal_completions (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  goal_id uuid NOT NULL REFERENCES goals(id) ON DELETE CASCADE,
  event_id uuid NOT NULL REFERENCES events(id) ON DELETE CASCADE,
  site_id uuid NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
  visitor_id text NOT NULL,
  occurred_at timestamptz NOT NULL,
  UNIQUE(goal_id, event_id)
);
CREATE INDEX goal_completions_site_time_idx ON goal_completions(site_id, occurred_at DESC);

CREATE TABLE stream_events (
  id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  site_id uuid NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
  event_id uuid NOT NULL REFERENCES events(id) ON DELETE CASCADE,
  payload jsonb NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX stream_events_site_id_idx ON stream_events(site_id, id DESC);
