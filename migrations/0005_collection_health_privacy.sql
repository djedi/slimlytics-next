ALTER TABLE events
  ADD COLUMN privacy_mode text NOT NULL DEFAULT 'standard',
  ADD COLUMN tracker_version text;

ALTER TABLE events
  ADD CONSTRAINT events_privacy_mode_check
    CHECK (privacy_mode IN ('standard', 'gpc')),
  ADD CONSTRAINT events_tracker_version_length_check
    CHECK (tracker_version IS NULL OR char_length(tracker_version) BETWEEN 1 AND 32);

CREATE TABLE collection_health (
  site_id uuid PRIMARY KEY REFERENCES sites(id) ON DELETE CASCADE,
  accepted_total bigint NOT NULL DEFAULT 0 CHECK (accepted_total >= 0),
  rejected_total bigint NOT NULL DEFAULT 0 CHECK (rejected_total >= 0),
  last_accepted_at timestamptz,
  last_rejected_at timestamptz,
  last_rejection_code text,
  last_tracker_version text,
  updated_at timestamptz NOT NULL DEFAULT now()
);
