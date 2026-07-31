ALTER TABLE events
  ADD COLUMN region text,
  ADD COLUMN city text,
  ADD COLUMN continent_code char(2),
  ADD COLUMN browser_version text,
  ADD COLUMN os_version text;

CREATE INDEX events_site_region_time_idx ON events(site_id, region, occurred_at DESC);
CREATE INDEX events_site_city_time_idx ON events(site_id, city, occurred_at DESC);
