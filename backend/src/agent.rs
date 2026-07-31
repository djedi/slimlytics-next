pub const SITES_READ: &str = "sites:read";
pub const SITES_WRITE: &str = "sites:write";
pub const ANALYTICS_READ: &str = "analytics:read";
pub const ANALYTICS_WRITE: &str = "analytics:write";
pub const INTEGRATIONS_READ: &str = "integrations:read";
pub const INTEGRATIONS_WRITE: &str = "integrations:write";

const ALLOWED_SCOPES: [&str; 6] = [
    SITES_READ,
    SITES_WRITE,
    ANALYTICS_READ,
    ANALYTICS_WRITE,
    INTEGRATIONS_READ,
    INTEGRATIONS_WRITE,
];

pub fn required_scope(method: &str, path: &str) -> &'static str {
    if method == "DELETE" && path == "/api/account/tokens/current" {
        return SITES_READ;
    }
    if path.contains("/integrations/") {
        return if method == "GET" {
            INTEGRATIONS_READ
        } else {
            INTEGRATIONS_WRITE
        };
    }
    let analytics_path = [
        "/overview",
        "/reports/",
        "/visitors",
        "/events",
        "/goals",
        "/funnels",
        "/annotations",
        "/report-subscriptions",
        "/insights/",
        "/export.csv",
        "/stream",
    ]
    .iter()
    .any(|segment| path.contains(segment));
    if analytics_path {
        if method == "GET" {
            ANALYTICS_READ
        } else {
            ANALYTICS_WRITE
        }
    } else if method == "GET" {
        SITES_READ
    } else {
        SITES_WRITE
    }
}

pub fn validate_idempotency_key(value: &str) -> Result<&str, &'static str> {
    if (8..=128).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
    {
        Ok(value)
    } else {
        Err("Idempotency-Key must be 8-128 URL-safe characters")
    }
}

pub fn validate_scopes(scopes: &[String]) -> Result<Vec<String>, &'static str> {
    if scopes.is_empty()
        || scopes.len() > ALLOWED_SCOPES.len()
        || scopes
            .iter()
            .any(|scope| !ALLOWED_SCOPES.contains(&scope.as_str()))
    {
        return Err("invalid API token scopes");
    }
    let mut result = scopes.to_vec();
    result.sort();
    result.dedup();
    Ok(result)
}
