use slimlytics_backend::models::{AntiAdblockInput, CollectInput};

#[test]
fn omitted_custom_properties_default_to_object() {
    let input: CollectInput = serde_json::from_value(serde_json::json!({
        "url": "https://example.com/"
    }))
    .unwrap();
    assert_eq!(input.properties, serde_json::json!({}));
}

#[test]
fn anti_adblock_configuration_accepts_clicky_style_paths() {
    let input = AntiAdblockInput {
        server_type: "caddy".into(),
        js_path: "/456bbb63bb86.js".into(),
        beacon_path: "/0d31360a3101".into(),
    };
    assert!(input.validate().is_ok());
}

#[test]
fn anti_adblock_javascript_path_matches_database_length_limit() {
    let accepted = AntiAdblockInput {
        server_type: "caddy".into(),
        js_path: format!("/{}.js", "a".repeat(63)),
        beacon_path: "/0d31360a3101".into(),
    };
    let rejected = AntiAdblockInput {
        js_path: format!("/{}.js", "a".repeat(64)),
        ..accepted
    };
    assert!(rejected.validate().is_err());
}

#[test]
fn anti_adblock_configuration_rejects_unsafe_or_unsupported_values() {
    for input in [
        AntiAdblockInput {
            server_type: "iis".into(),
            js_path: "/456bbb63bb86.js".into(),
            beacon_path: "/0d31360a3101".into(),
        },
        AntiAdblockInput {
            server_type: "caddy".into(),
            js_path: "/x.js".into(),
            beacon_path: "/0d31360a3101".into(),
        },
        AntiAdblockInput {
            server_type: "nginx".into(),
            js_path: "/456bbb63bb86.js".into(),
            beacon_path: "/bad/path".into(),
        },
        AntiAdblockInput {
            server_type: "apache".into(),
            js_path: "/samepath.js".into(),
            beacon_path: "/samepath.js".into(),
        },
    ] {
        assert!(
            input.validate().is_err(),
            "unexpected valid input: {input:?}"
        );
    }
}
