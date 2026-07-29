use thiserror::Error;
use url::Url;

#[derive(Debug, Error)]
pub enum PrivacyError {
    #[error("invalid URL")]
    InvalidUrl(#[from] url::ParseError),
    #[error("only http(s) URLs are accepted")]
    InvalidScheme,
}

/// Removes all query parameters and fragments before persistence.
pub fn sanitize_url(input: &str) -> Result<String, PrivacyError> {
    let mut url = Url::parse(input)?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(PrivacyError::InvalidScheme);
    }
    url.set_query(None);
    url.set_fragment(None);
    Ok(url.into())
}
