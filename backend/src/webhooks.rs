use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use hmac::{Hmac, Mac};
use reqwest::{redirect::Policy, StatusCode};
use serde_json::Value;
use sha2::Sha256;
use std::{net::IpAddr, time::Duration};
use url::Url;

pub fn validate_webhook_url(value: &str) -> Result<Url, &'static str> {
    let url = Url::parse(value).map_err(|_| "invalid webhookUrl")?;
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
        || url.port_or_known_default() != Some(443)
    {
        return Err(
            "webhookUrl must be an HTTPS URL on port 443 without credentials or a fragment",
        );
    }
    let host = url.host_str().expect("checked host");
    if host.eq_ignore_ascii_case("localhost")
        || host.ends_with(".localhost")
        || host.parse::<IpAddr>().is_ok_and(|ip| !is_public_ip(ip))
    {
        return Err("webhookUrl must use a public host");
    }
    Ok(url)
}

pub fn is_public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            let [a, b, c, _] = ip.octets();
            !(a == 0
                || a == 10
                || a == 127
                || (a == 100 && (64..=127).contains(&b))
                || (a == 169 && b == 254)
                || (a == 172 && (16..=31).contains(&b))
                || (a == 192 && b == 0 && c == 0)
                || (a == 192 && b == 0 && c == 2)
                || (a == 192 && b == 168)
                || (a == 198 && (b == 18 || b == 19))
                || (a == 198 && b == 51 && c == 100)
                || (a == 203 && b == 0 && c == 113)
                || a >= 224)
        }
        IpAddr::V6(ip) => {
            if let Some(mapped) = ip.to_ipv4_mapped() {
                return is_public_ip(IpAddr::V4(mapped));
            }
            let first = ip.segments()[0];
            !(ip.is_unspecified()
                || ip.is_loopback()
                || ip.is_multicast()
                || first & 0xfe00 == 0xfc00
                || first & 0xffc0 == 0xfe80
                || (first == 0x2001 && ip.segments()[1] == 0x0db8))
        }
    }
}

pub fn signing_secret(identity_secret: &[u8], subscription_id: uuid::Uuid) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(identity_secret).expect("valid HMAC key");
    mac.update(b"slimlytics-report-webhook|");
    mac.update(subscription_id.as_bytes());
    URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes())
}

pub async fn send_signed_json(
    destination: &str,
    payload: &Value,
    secret: &str,
) -> Result<StatusCode, String> {
    let url = validate_webhook_url(destination).map_err(str::to_owned)?;
    let host = url.host_str().expect("validated host").to_owned();
    let addresses: Vec<_> = tokio::net::lookup_host((host.as_str(), 443))
        .await
        .map_err(|error| format!("webhook DNS lookup failed: {error}"))?
        .collect();
    if addresses.is_empty() || addresses.iter().any(|address| !is_public_ip(address.ip())) {
        return Err("webhook DNS resolved to a non-public address".into());
    }
    let body = serde_json::to_vec(payload).map_err(|error| error.to_string())?;
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("valid HMAC key");
    mac.update(&body);
    let signature = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
    let client = reqwest::Client::builder()
        .redirect(Policy::none())
        .timeout(Duration::from_secs(10))
        .resolve(&host, addresses[0])
        .build()
        .map_err(|error| error.to_string())?;
    let response = client
        .post(url)
        .header("content-type", "application/json")
        .header("x-slimlytics-signature", format!("sha256={signature}"))
        .body(body)
        .send()
        .await
        .map_err(|error| format!("webhook request failed: {error}"))?;
    if response.status().is_success() {
        Ok(response.status())
    } else {
        Err(format!("webhook returned HTTP {}", response.status()))
    }
}
