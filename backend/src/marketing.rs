use serde_json::Value;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MarketingContext {
    pub revenue_amount: Option<String>,
    pub revenue_currency: Option<String>,
    pub content_id: Option<String>,
    pub content_type: Option<String>,
    pub content_author: Option<String>,
}

pub fn marketing_context(properties: &Value) -> MarketingContext {
    let revenue_amount = properties
        .get("revenue")
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite() && (0.0..=1_000_000_000_000.0).contains(value))
        .map(|value| value.to_string());
    let revenue_currency = properties
        .get("currency")
        .and_then(Value::as_str)
        .filter(|value| value.len() == 3 && value.bytes().all(|byte| byte.is_ascii_alphabetic()))
        .map(str::to_ascii_uppercase);
    MarketingContext {
        revenue_amount,
        revenue_currency,
        content_id: short_string(properties, "contentId"),
        content_type: short_string(properties, "contentType"),
        content_author: short_string(properties, "contentAuthor"),
    }
}

fn short_string(properties: &Value, key: &str) -> Option<String> {
    properties
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty() && value.len() <= 240)
        .map(str::to_owned)
}
