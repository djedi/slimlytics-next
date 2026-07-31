use serde_json::json;
use slimlytics_backend::marketing::marketing_context;

#[test]
fn extracts_valid_revenue_and_content_context() {
    let context = marketing_context(&json!({
        "revenue": 49.95,
        "currency": "usd",
        "contentId": "post-123",
        "contentType": "article",
        "contentAuthor": "Marketing"
    }));

    assert_eq!(context.revenue_amount.as_deref(), Some("49.95"));
    assert_eq!(context.revenue_currency.as_deref(), Some("USD"));
    assert_eq!(context.content_id.as_deref(), Some("post-123"));
    assert_eq!(context.content_type.as_deref(), Some("article"));
    assert_eq!(context.content_author.as_deref(), Some("Marketing"));
}

#[test]
fn rejects_invalid_or_excessive_marketing_context() {
    let context = marketing_context(&json!({
        "revenue": -1,
        "currency": "dollars",
        "contentId": "x".repeat(300),
        "contentType": {"unsafe": true}
    }));

    assert_eq!(context.revenue_amount, None);
    assert_eq!(context.revenue_currency, None);
    assert_eq!(context.content_id, None);
    assert_eq!(context.content_type, None);
}
