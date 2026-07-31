ALTER TABLE events
  ADD COLUMN automation_name text,
  ADD COLUMN automation_category text
    CHECK (automation_category IS NULL OR automation_category IN ('ai-crawler', 'crawler'));

CREATE INDEX events_site_automation_time_idx
  ON events(site_id, automation_category, automation_name, occurred_at DESC)
  WHERE automation_category IS NOT NULL;
