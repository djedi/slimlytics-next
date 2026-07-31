use anyhow::{anyhow, bail, Context, Result};
use reqwest::{Method, StatusCode};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};
use url::Url;
use uuid::Uuid;

pub const DEFAULT_API_URL: &str = "https://slimlytics.com";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StoredAuth {
    pub api_url: String,
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Site {
    pub id: Uuid,
    pub name: String,
    pub domain: String,
    pub timezone: String,
    pub allowed_origins: Vec<String>,
    pub retention_days: i32,
    pub write_key: Uuid,
    pub server_write_key: Uuid,
    pub anti_adblock_server: String,
    pub anti_adblock_js_path: String,
    pub anti_adblock_beacon_path: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnsureSiteResponse {
    pub created: bool,
    pub site: Site,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteInput {
    pub name: String,
    pub domain: String,
    pub timezone: String,
    pub allowed_origins: Vec<String>,
    pub retention_days: i32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AntiAdblockInput {
    pub server_type: String,
    pub js_path: String,
    pub beacon_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: Uuid,
    pub email: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiTokenSummary {
    pub id: Uuid,
    pub name: String,
    pub token_prefix: String,
    pub last_used_at: Option<String>,
    pub expires_at: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiTokenCreated {
    pub id: Uuid,
    pub name: String,
    pub token_prefix: String,
    pub token: String,
    pub expires_at: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackingSetup {
    pub site_id: Uuid,
    pub domain: String,
    pub server_type: String,
    pub javascript_path: String,
    pub beacon_path: String,
    pub server_config: String,
    pub snippet: String,
    pub script_test_url: String,
    pub beacon_test_url: String,
    pub server_ingest_url: String,
    pub next_steps: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    token: String,
}

#[derive(Debug, Deserialize)]
struct ErrorEnvelope {
    error: ErrorDetail,
}

#[derive(Debug, Deserialize)]
struct ErrorDetail {
    code: String,
    message: String,
}

#[derive(Clone)]
pub struct ApiClient {
    http: reqwest::Client,
    base_url: String,
    token: Option<String>,
}

impl ApiClient {
    pub fn new(base_url: &str, token: Option<String>) -> Result<Self> {
        let base_url = normalize_api_url(base_url)?;
        Ok(Self {
            http: reqwest::Client::builder()
                .user_agent(concat!("slimlytics-cli/", env!("CARGO_PKG_VERSION")))
                .redirect(reqwest::redirect::Policy::none())
                .build()?,
            base_url,
            token,
        })
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    async fn request<T: DeserializeOwned, B: Serialize + ?Sized>(
        &self,
        method: Method,
        path: &str,
        body: Option<&B>,
        token: Option<&str>,
    ) -> Result<T> {
        let mut request = self
            .http
            .request(method, format!("{}{}", self.base_url, path));
        if let Some(value) = token.or(self.token.as_deref()) {
            request = request.bearer_auth(value);
        }
        if let Some(body) = body {
            request = request.json(body);
        }
        let response = request
            .send()
            .await
            .context("Slimlytics API request failed")?;
        let status = response.status();
        let bytes = response.bytes().await?;
        if !status.is_success() {
            if let Ok(error) = serde_json::from_slice::<ErrorEnvelope>(&bytes) {
                bail!("{}: {}", error.error.code, error.error.message);
            }
            bail!("Slimlytics API returned {status}");
        }
        serde_json::from_slice(&bytes).context("Slimlytics API returned invalid JSON")
    }

    async fn request_empty(&self, method: Method, path: &str) -> Result<()> {
        let mut request = self
            .http
            .request(method, format!("{}{}", self.base_url, path));
        if let Some(value) = self.token.as_deref() {
            request = request.bearer_auth(value);
        }
        let response = request
            .send()
            .await
            .context("Slimlytics API request failed")?;
        let status = response.status();
        if status.is_success() {
            return Ok(());
        }
        let bytes = response.bytes().await?;
        if let Ok(error) = serde_json::from_slice::<ErrorEnvelope>(&bytes) {
            bail!("{}: {}", error.error.code, error.error.message);
        }
        bail!("Slimlytics API returned {status}")
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<String> {
        #[derive(Serialize)]
        struct Login<'a> {
            email: &'a str,
            password: &'a str,
        }
        Ok(self
            .request::<TokenResponse, _>(
                Method::POST,
                "/api/auth/login",
                Some(&Login { email, password }),
                None,
            )
            .await?
            .token)
    }

    pub async fn create_api_token(
        &self,
        session_token: &str,
        name: &str,
        expires_in_days: i64,
    ) -> Result<ApiTokenCreated> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Input<'a> {
            name: &'a str,
            expires_in_days: i64,
            scopes: [&'a str; 6],
        }
        self.request(
            Method::POST,
            "/api/account/tokens",
            Some(&Input {
                name,
                expires_in_days,
                scopes: [
                    "sites:read",
                    "sites:write",
                    "analytics:read",
                    "analytics:write",
                    "integrations:read",
                    "integrations:write",
                ],
            }),
            Some(session_token),
        )
        .await
    }

    pub async fn account(&self) -> Result<Account> {
        self.request::<Account, serde_json::Value>(Method::GET, "/api/auth/me", None, None)
            .await
    }

    pub async fn tokens(&self) -> Result<Vec<ApiTokenSummary>> {
        self.request::<Vec<ApiTokenSummary>, serde_json::Value>(
            Method::GET,
            "/api/account/tokens",
            None,
            None,
        )
        .await
    }

    pub async fn revoke_token(&self, id: Uuid) -> Result<()> {
        self.request_empty(Method::DELETE, &format!("/api/account/tokens/{id}"))
            .await
    }

    pub async fn revoke_current_token(&self) -> Result<()> {
        self.request_empty(Method::DELETE, "/api/account/tokens/current")
            .await
    }

    pub async fn sites(&self) -> Result<Vec<Site>> {
        self.request::<Vec<Site>, serde_json::Value>(Method::GET, "/api/sites", None, None)
            .await
    }

    pub async fn create_site(&self, input: &SiteInput) -> Result<Site> {
        self.request(Method::POST, "/api/sites", Some(input), None)
            .await
    }

    pub async fn ensure_site(&self, input: &SiteInput) -> Result<EnsureSiteResponse> {
        self.request(Method::POST, "/api/sites/ensure", Some(input), None)
            .await
    }

    pub async fn configure_tracking(&self, id: Uuid, input: &AntiAdblockInput) -> Result<Site> {
        self.request(
            Method::PUT,
            &format!("/api/sites/{id}/anti-adblock"),
            Some(input),
            None,
        )
        .await
    }

    pub async fn delete_site(&self, id: Uuid) -> Result<()> {
        self.request_empty(Method::DELETE, &format!("/api/sites/{id}"))
            .await
    }
}

pub fn normalize_api_url(value: &str) -> Result<String> {
    let parsed = Url::parse(value.trim()).context("invalid Slimlytics API URL")?;
    if !matches!(parsed.scheme(), "http" | "https") || parsed.host_str().is_none() {
        bail!("Slimlytics API URL must use http or https");
    }
    if parsed.scheme() == "http" {
        let host = parsed.host_str().unwrap_or_default();
        let loopback = host.eq_ignore_ascii_case("localhost")
            || host
                .parse::<std::net::IpAddr>()
                .is_ok_and(|address| address.is_loopback());
        if !loopback {
            bail!("plaintext HTTP is allowed only for loopback development");
        }
    }
    if parsed.query().is_some() || parsed.fragment().is_some() {
        bail!("Slimlytics API URL cannot include a query or fragment");
    }
    if parsed.path() != "/" {
        bail!("Slimlytics API URL cannot include a path");
    }
    Ok(value.trim().trim_end_matches('/').to_owned())
}

pub fn normalize_domain(value: &str) -> Result<String> {
    let candidate = value.trim();
    let parsed = if candidate.contains("://") {
        Url::parse(candidate)?
    } else {
        Url::parse(&format!("https://{candidate}"))?
    };
    if parsed.scheme() != "https" && parsed.scheme() != "http" {
        bail!("domain must use http or https");
    }
    if parsed.path() != "/" || parsed.query().is_some() || parsed.fragment().is_some() {
        bail!("domain cannot contain a path, query, or fragment");
    }
    if parsed.port().is_some() {
        bail!("domain cannot contain a port");
    }
    let host = parsed.host_str().ok_or_else(|| anyhow!("invalid domain"))?;
    if !host.contains('.') && host != "localhost" {
        bail!("invalid domain");
    }
    if host.chars().any(char::is_whitespace) {
        bail!("invalid domain");
    }
    Ok(host.to_ascii_lowercase())
}

pub fn find_site<'a>(sites: &'a [Site], selector: &str) -> Result<&'a Site> {
    let id = Uuid::parse_str(selector).ok();
    let matches = sites
        .iter()
        .filter(|site| Some(site.id) == id || site.domain.eq_ignore_ascii_case(selector))
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [site] => Ok(*site),
        [] => bail!("site not found: {selector}"),
        _ => bail!("site selector is ambiguous: {selector}; use a site ID"),
    }
}

pub fn default_auth_path() -> Result<PathBuf> {
    if let Some(value) = std::env::var_os("SLIMLYTICS_CONFIG") {
        return Ok(PathBuf::from(value));
    }
    let directory =
        dirs::config_dir().ok_or_else(|| anyhow!("configuration directory unavailable"))?;
    Ok(directory.join("slimlytics").join("auth.json"))
}

pub fn save_auth(path: &Path, auth: &StoredAuth) -> Result<()> {
    if fs::symlink_metadata(path)
        .map(|metadata| metadata.file_type().is_symlink())
        .unwrap_or(false)
    {
        bail!("refusing to write authentication through a symbolic link");
    }
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("invalid auth file path"))?;
    fs::create_dir_all(parent)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
        use std::time::{SystemTime, UNIX_EPOCH};
        fs::set_permissions(parent, fs::Permissions::from_mode(0o700))?;
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let filename = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("auth.json");
        let temporary = parent.join(format!(".{filename}.{}.{nonce}.tmp", std::process::id()));
        let result: Result<()> = (|| {
            let mut options = fs::OpenOptions::new();
            options.create_new(true).write(true).mode(0o600);
            let mut file = options.open(&temporary)?;
            serde_json::to_writer_pretty(&mut file, auth)?;
            file.sync_all()?;
            fs::rename(&temporary, path)?;
            fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result?;
    }
    #[cfg(not(unix))]
    {
        fs::write(path, serde_json::to_vec_pretty(auth)?)?;
    }
    Ok(())
}

pub fn load_auth(path: &Path) -> Result<StoredAuth> {
    serde_json::from_slice(&fs::read(path).with_context(|| {
        format!(
            "not authenticated; run `slimlytics auth login` ({})",
            path.display()
        )
    })?)
    .context("invalid Slimlytics auth file")
}

pub fn tracking_setup(site: &Site, analytics_origin: &str) -> Result<TrackingSetup> {
    if !valid_proxy_path(&site.anti_adblock_js_path, true)
        || !valid_proxy_path(&site.anti_adblock_beacon_path, false)
        || site.anti_adblock_js_path == site.anti_adblock_beacon_path
    {
        bail!("Slimlytics API returned unsafe first-party tracking paths");
    }
    let analytics = normalize_api_url(analytics_origin)?;
    let domain = normalize_domain(&site.domain)?;
    let website = format!("https://{domain}");
    let bootstrap_path = format!(
        "/p/{}/{}",
        site.write_key,
        site.anti_adblock_beacon_path.trim_start_matches('/')
    );
    let collect_path = format!("/api/collect/{}", site.write_key);
    let bootstrap = format!("{analytics}{bootstrap_path}");
    let collect = format!("{analytics}{collect_path}");
    let server_config = match site.anti_adblock_server.as_str() {
        "caddy" => format!(
            "# Slimlytics first-party tracking\nhandle {} {{\n\trewrite {} {}\n\treverse_proxy {} {{\n\t\theader_up Host {{upstream_hostport}}\n\t\theader_up -Cookie\n\t\theader_up -Authorization\n\t\theader_down -Set-Cookie\n\t}}\n}}\n\nhandle {} {{\n\trewrite {} {}\n\treverse_proxy {} {{\n\t\theader_up Host {{upstream_hostport}}\n\t\theader_up -Cookie\n\t\theader_up -Authorization\n\t\theader_down -Set-Cookie\n\t}}\n}}",
            site.anti_adblock_js_path,
            site.anti_adblock_js_path,
            bootstrap_path,
            analytics,
            site.anti_adblock_beacon_path,
            site.anti_adblock_beacon_path,
            collect_path,
            analytics
        ),
        "nginx" => format!(
            "# Slimlytics first-party tracking\nlocation = {} {{\n    proxy_pass {};\n    proxy_set_header Host {};\n    proxy_set_header Cookie \"\";\n    proxy_set_header Authorization \"\";\n    proxy_set_header X-Forwarded-For $remote_addr;\n    proxy_hide_header Set-Cookie;\n    proxy_ssl_server_name on;\n    proxy_ssl_name {};\n}}\n\nlocation = {} {{\n    proxy_pass {};\n    proxy_set_header Host {};\n    proxy_set_header Cookie \"\";\n    proxy_set_header Authorization \"\";\n    proxy_set_header X-Forwarded-For $remote_addr;\n    proxy_hide_header Set-Cookie;\n    proxy_ssl_server_name on;\n    proxy_ssl_name {};\n}}",
            site.anti_adblock_js_path,
            bootstrap,
            Url::parse(&analytics)?.host_str().unwrap(),
            Url::parse(&analytics)?.host_str().unwrap(),
            site.anti_adblock_beacon_path,
            collect,
            Url::parse(&analytics)?.host_str().unwrap(),
            Url::parse(&analytics)?.host_str().unwrap()
        ),
        "apache" => format!(
            "# Slimlytics first-party tracking\n# Requires mod_proxy, mod_proxy_http, mod_ssl, and mod_headers.\nSSLProxyEngine On\nProxyPassMatch \"^{}$\" \"{}\"\nProxyPassMatch \"^{}$\" \"{}\"\n<LocationMatch \"^(?:{}|{})$\">\n    RequestHeader unset Cookie\n    RequestHeader unset Authorization\n    RequestHeader unset X-Forwarded-For\n    Header always unset Set-Cookie\n</LocationMatch>",
            regex_escape(&site.anti_adblock_js_path),
            bootstrap,
            regex_escape(&site.anti_adblock_beacon_path),
            collect,
            regex_escape(&site.anti_adblock_js_path),
            regex_escape(&site.anti_adblock_beacon_path)
        ),
        other => bail!("unsupported server type: {other}"),
    };
    Ok(TrackingSetup {
        site_id: site.id,
        domain: site.domain.clone(),
        server_type: site.anti_adblock_server.clone(),
        javascript_path: site.anti_adblock_js_path.clone(),
        beacon_path: site.anti_adblock_beacon_path.clone(),
        server_config,
        snippet: format!(r#"<script async src="{}"></script>"#, site.anti_adblock_js_path),
        script_test_url: format!("{website}{}", site.anti_adblock_js_path),
        beacon_test_url: format!("{website}{}", site.anti_adblock_beacon_path),
        server_ingest_url: format!("{analytics}/api/ingest"),
        next_steps: vec![
            "Install serverConfig in the website's Caddy, Nginx, or Apache configuration and reload the server.".into(),
            "Add snippet to every page before the closing </body> tag.".into(),
            "Open scriptTestUrl and beaconTestUrl; both must return HTTP 200.".into(),
        ],
    })
}

fn regex_escape(value: &str) -> String {
    let mut escaped = String::new();
    for character in value.chars() {
        if matches!(
            character,
            '.' | '+' | '*' | '?' | '^' | '$' | '(' | ')' | '[' | ']' | '{' | '}' | '|' | '\\'
        ) {
            escaped.push('\\');
        }
        escaped.push(character);
    }
    escaped
}

fn valid_proxy_path(value: &str, javascript: bool) -> bool {
    let Some(name) = value.strip_prefix('/') else {
        return false;
    };
    if name.contains('/') || name.len() > 67 {
        return false;
    }
    let stem = if javascript {
        let Some(stem) = name.strip_suffix(".js") else {
            return false;
        };
        stem
    } else {
        name
    };
    let max_stem_len = if javascript { 63 } else { 64 };
    (6..=max_stem_len).contains(&stem.len())
        && stem
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'~' | b'-'))
}

pub fn ensure_success_status(status: StatusCode) -> Result<()> {
    if status.is_success() {
        Ok(())
    } else {
        bail!("HTTP {status}")
    }
}
