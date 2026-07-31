use slimlytics_backend::agent::{
    required_scope, validate_idempotency_key, validate_scopes, ANALYTICS_READ, ANALYTICS_WRITE,
    INTEGRATIONS_WRITE, SITES_READ, SITES_WRITE,
};

#[test]
fn validates_agent_token_scopes() {
    assert_eq!(
        validate_scopes(&[ANALYTICS_READ.to_string(), ANALYTICS_WRITE.to_string()]).unwrap(),
        vec![ANALYTICS_READ.to_string(), ANALYTICS_WRITE.to_string()]
    );
    assert!(validate_scopes(&["admin".to_string()]).is_err());
    assert!(validate_scopes(&[]).is_err());
}

#[test]
fn validates_stable_idempotency_keys() {
    assert_eq!(
        validate_idempotency_key("campaign-agent:550e8400-e29b-41d4-a716-446655440000").unwrap(),
        "campaign-agent:550e8400-e29b-41d4-a716-446655440000"
    );
    assert!(validate_idempotency_key("short").is_err());
    assert!(validate_idempotency_key("unsafe key\n").is_err());
}

#[test]
fn maps_rest_operations_to_least_privilege_scopes() {
    assert_eq!(required_scope("GET", "/api/sites"), SITES_READ);
    assert_eq!(
        required_scope("DELETE", "/api/account/tokens/current"),
        SITES_READ
    );
    assert_eq!(
        required_scope("GET", "/api/sites/id/overview"),
        ANALYTICS_READ
    );
    assert_eq!(
        required_scope("POST", "/api/sites/id/funnels"),
        ANALYTICS_WRITE
    );
    assert_eq!(
        required_scope("POST", "/api/sites/id/report-subscriptions"),
        ANALYTICS_WRITE
    );
    assert_eq!(
        required_scope("DELETE", "/api/sites/id/integrations/search-console"),
        INTEGRATIONS_WRITE
    );
    assert_eq!(required_scope("DELETE", "/api/sites/id"), SITES_WRITE);
}
