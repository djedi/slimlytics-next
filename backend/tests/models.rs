use slimlytics_backend::models::CollectInput;

#[test]
fn omitted_custom_properties_default_to_object() {
    let input: CollectInput = serde_json::from_value(serde_json::json!({
        "url": "https://example.com/"
    }))
    .unwrap();
    assert_eq!(input.properties, serde_json::json!({}));
}
