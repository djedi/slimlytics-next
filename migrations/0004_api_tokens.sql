CREATE TABLE api_tokens (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  name text NOT NULL CHECK (char_length(name) BETWEEN 1 AND 100),
  token_hash bytea NOT NULL UNIQUE CHECK (octet_length(token_hash) = 32),
  token_prefix text NOT NULL CHECK (char_length(token_prefix) BETWEEN 9 AND 16),
  last_used_at timestamptz,
  expires_at timestamptz NOT NULL,
  revoked_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now(),
  CHECK (expires_at > created_at)
);

CREATE INDEX api_tokens_user_created_idx ON api_tokens(user_id, created_at DESC);
CREATE INDEX api_tokens_active_user_idx ON api_tokens(user_id) WHERE revoked_at IS NULL;

CREATE UNIQUE INDEX sites_domain_unique_idx ON sites(lower(domain));
