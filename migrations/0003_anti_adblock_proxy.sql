ALTER TABLE sites
  ADD COLUMN anti_adblock_server text NOT NULL DEFAULT 'caddy',
  ADD COLUMN anti_adblock_js_path text NOT NULL DEFAULT ('/' || encode(gen_random_bytes(6), 'hex') || '.js'),
  ADD COLUMN anti_adblock_beacon_path text NOT NULL DEFAULT ('/' || encode(gen_random_bytes(6), 'hex'));

ALTER TABLE sites
  ADD CONSTRAINT sites_anti_adblock_server_check
    CHECK (anti_adblock_server IN ('caddy', 'nginx', 'apache')),
  ADD CONSTRAINT sites_anti_adblock_js_path_check
    CHECK (anti_adblock_js_path ~ '^/[A-Za-z0-9][A-Za-z0-9._~-]{5,62}\.js$'),
  ADD CONSTRAINT sites_anti_adblock_beacon_path_check
    CHECK (anti_adblock_beacon_path ~ '^/[A-Za-z0-9][A-Za-z0-9._~-]{5,63}$'),
  ADD CONSTRAINT sites_anti_adblock_paths_differ_check
    CHECK (anti_adblock_js_path <> anti_adblock_beacon_path);
