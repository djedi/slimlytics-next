ALTER TABLE api_tokens
  ADD COLUMN scopes text[] NOT NULL DEFAULT ARRAY['sites:read','analytics:read'];

ALTER TABLE api_tokens ADD CONSTRAINT api_tokens_scopes_valid CHECK (
  cardinality(scopes) BETWEEN 1 AND 6
  AND scopes <@ ARRAY[
    'sites:read','sites:write','analytics:read','analytics:write',
    'integrations:read','integrations:write'
  ]::text[]
);

CREATE TABLE agent_audit_log (
  id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  api_token_id uuid REFERENCES api_tokens(id) ON DELETE SET NULL,
  site_id uuid REFERENCES sites(id) ON DELETE CASCADE,
  action text NOT NULL,
  request_id text,
  input jsonb NOT NULL DEFAULT '{}',
  outcome text NOT NULL CHECK (outcome IN ('success','error')),
  created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX agent_audit_user_time_idx ON agent_audit_log(user_id,created_at DESC);
CREATE INDEX agent_audit_site_time_idx ON agent_audit_log(site_id,created_at DESC);

CREATE TABLE idempotency_keys (
  user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  site_id uuid NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
  operation text NOT NULL,
  idempotency_key text NOT NULL,
  response_status integer NOT NULL,
  response_body jsonb NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  expires_at timestamptz NOT NULL DEFAULT now()+interval '24 hours',
  PRIMARY KEY(user_id,site_id,operation,idempotency_key)
);
CREATE INDEX idempotency_keys_expiry_idx ON idempotency_keys(expires_at);
