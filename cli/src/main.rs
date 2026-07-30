use anyhow::{bail, Context, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::Serialize;
use slimlytics_cli::{
    default_auth_path, find_site, load_auth, normalize_api_url, normalize_domain, save_auth,
    tracking_setup, AntiAdblockInput, ApiClient, SiteInput, StoredAuth, DEFAULT_API_URL,
};
use std::{
    fs,
    io::{self, IsTerminal, Read},
    path::Path,
};
use uuid::Uuid;

#[derive(Parser)]
#[command(
    name = "slimlytics",
    version,
    about = "Manage Slimlytics accounts and tracking from the command line"
)]
struct Cli {
    #[arg(long, global = true, help = "Emit machine-readable JSON")]
    json: bool,
    #[arg(long, global = true, env = "SLIMLYTICS_API_URL")]
    api_url: Option<String>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Auth {
        #[command(subcommand)]
        command: AuthCommand,
    },
    Account {
        #[command(subcommand)]
        command: AccountCommand,
    },
    Token {
        #[command(subcommand)]
        command: TokenCommand,
    },
    Site {
        #[command(subcommand)]
        command: SiteCommand,
    },
    Tracking {
        #[command(subcommand)]
        command: TrackingCommand,
    },
}

#[derive(Subcommand)]
enum AuthCommand {
    Login(LoginArgs),
    UseToken(UseTokenArgs),
    Status,
    Logout {
        #[arg(long)]
        revoke: bool,
    },
}

#[derive(Args)]
struct LoginArgs {
    #[arg(long)]
    email: String,
    #[arg(
        long,
        help = "Read the password from standard input instead of prompting"
    )]
    password_stdin: bool,
    #[arg(long, default_value = "slimlytics-cli")]
    token_name: String,
    #[arg(long, default_value_t = 365)]
    expires_in_days: i64,
}

#[derive(Args)]
struct UseTokenArgs {
    #[arg(long, help = "Read a personal access token from standard input")]
    token_stdin: bool,
}

#[derive(Subcommand)]
enum AccountCommand {
    Show,
}

#[derive(Subcommand)]
enum TokenCommand {
    List,
    Revoke { id: Uuid },
}

#[derive(Subcommand)]
enum SiteCommand {
    List,
    Show {
        site: String,
    },
    Add(AddSiteArgs),
    #[command(about = "Create a domain if absent, then return tracking setup")]
    Ensure(AddSiteArgs),
    Delete {
        site: String,
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Args)]
struct AddSiteArgs {
    domain: String,
    #[arg(long)]
    name: Option<String>,
    #[arg(long, default_value = "UTC")]
    timezone: String,
    #[arg(long, default_value_t = 365)]
    retention_days: i32,
    #[arg(long = "origin")]
    allowed_origins: Vec<String>,
    #[arg(long, value_enum, default_value_t = Server::Caddy)]
    server: Server,
}

#[derive(Subcommand)]
enum TrackingCommand {
    Show { site: String },
    Configure(ConfigureTrackingArgs),
}

#[derive(Args)]
struct ConfigureTrackingArgs {
    site: String,
    #[arg(long, value_enum)]
    server: Server,
    #[arg(long)]
    js_path: Option<String>,
    #[arg(long)]
    beacon_path: Option<String>,
}

#[derive(Clone, Copy, ValueEnum)]
enum Server {
    Caddy,
    Nginx,
    Apache,
}
impl Server {
    fn as_str(self) -> &'static str {
        match self {
            Self::Caddy => "caddy",
            Self::Nginx => "nginx",
            Self::Apache => "apache",
        }
    }
}

#[tokio::main]
async fn main() {
    if let Err(error) = run(Cli::parse()).await {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<()> {
    let path = default_auth_path()?;
    match cli.command {
        Command::Auth { command } => {
            return auth_command(command, cli.api_url, cli.json, &path).await
        }
        command => {
            let auth = effective_auth(&path, cli.api_url.as_deref())?;
            let client = ApiClient::new(&auth.api_url, Some(auth.token))?;
            match command {
                Command::Account { command } => match command {
                    AccountCommand::Show => {
                        print_value(&client.account().await?, cli.json, |account| {
                            format!("{} ({})", account.email, account.id)
                        })
                    }
                },
                Command::Token { command } => token_command(&client, command, cli.json).await,
                Command::Site { command } => site_command(&client, command, cli.json).await,
                Command::Tracking { command } => tracking_command(&client, command, cli.json).await,
                Command::Auth { .. } => unreachable!(),
            }
        }
    }
}

async fn auth_command(
    command: AuthCommand,
    api_url: Option<String>,
    json: bool,
    path: &Path,
) -> Result<()> {
    match command {
        AuthCommand::Login(args) => {
            let api_url = normalize_api_url(api_url.as_deref().unwrap_or(DEFAULT_API_URL))?;
            let password = read_password(args.password_stdin)?;
            let client = ApiClient::new(&api_url, None)?;
            let session = client.login(&args.email, &password).await?;
            let created = client
                .create_api_token(&session, &args.token_name, args.expires_in_days)
                .await?;
            let auth = StoredAuth {
                api_url: api_url.clone(),
                token: created.token,
            };
            if let Err(error) = save_auth(path, &auth) {
                let _ = ApiClient::new(&api_url, Some(auth.token.clone()))?
                    .revoke_current_token()
                    .await;
                return Err(error);
            }
            let account = ApiClient::new(&api_url, Some(auth.token))?
                .account()
                .await?;
            print_value(&account, json, |value| {
                format!("Authenticated as {}", value.email)
            })
        }
        AuthCommand::UseToken(args) => {
            if !args.token_stdin && io::stdin().is_terminal() {
                bail!(
                    "use --token-stdin to read the token without exposing it in process arguments"
                );
            }
            let token = read_secret_stdin("personal access token")?;
            if !token.starts_with("slyt_") {
                bail!("invalid Slimlytics personal access token");
            }
            let api_url = normalize_api_url(api_url.as_deref().unwrap_or(DEFAULT_API_URL))?;
            let account = ApiClient::new(&api_url, Some(token.clone()))?
                .account()
                .await?;
            save_auth(path, &StoredAuth { api_url, token })?;
            print_value(&account, json, |value| {
                format!("Authenticated as {}", value.email)
            })
        }
        AuthCommand::Status => {
            let auth = effective_auth(path, api_url.as_deref())?;
            let account = ApiClient::new(&auth.api_url, Some(auth.token))?
                .account()
                .await?;
            print_value(&account, json, |value| {
                format!("Authenticated as {}", value.email)
            })
        }
        AuthCommand::Logout { revoke } => {
            let auth = load_auth(path)?;
            if revoke {
                ApiClient::new(&auth.api_url, Some(auth.token.clone()))?
                    .revoke_current_token()
                    .await?;
            }
            if path.exists() {
                fs::remove_file(path)?;
            }
            if json {
                println!("{{\"authenticated\":false}}");
            } else {
                println!("Logged out");
            }
            Ok(())
        }
    }
}

async fn token_command(client: &ApiClient, command: TokenCommand, json: bool) -> Result<()> {
    match command {
        TokenCommand::List => print_value(&client.tokens().await?, json, |tokens| {
            if tokens.is_empty() {
                "No active API tokens".into()
            } else {
                tokens
                    .iter()
                    .map(|token| format!("{}\t{}\t{}", token.id, token.name, token.token_prefix))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }),
        TokenCommand::Revoke { id } => {
            client.revoke_token(id).await?;
            if json {
                println!("{{\"revoked\":true,\"id\":\"{id}\"}}");
            } else {
                println!("Revoked {id}");
            }
            Ok(())
        }
    }
}

async fn site_command(client: &ApiClient, command: SiteCommand, json: bool) -> Result<()> {
    match command {
        SiteCommand::List => print_value(&client.sites().await?, json, |sites| {
            if sites.is_empty() {
                "No sites".into()
            } else {
                sites
                    .iter()
                    .map(|site| format!("{}\t{}\t{}", site.id, site.domain, site.name))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }),
        SiteCommand::Show { site } => {
            let sites = client.sites().await?;
            print_value(find_site(&sites, &site)?, json, |value| {
                format!(
                    "{}\nID: {}\nWrite key: {}",
                    value.domain, value.id, value.write_key
                )
            })
        }
        SiteCommand::Add(args) => provision_site(client, args, false, json).await,
        SiteCommand::Ensure(args) => provision_site(client, args, true, json).await,
        SiteCommand::Delete { site, yes } => {
            if !yes {
                bail!("refusing to delete without --yes");
            }
            let sites = client.sites().await?;
            let selected = find_site(&sites, &site)?;
            client.delete_site(selected.id).await?;
            if json {
                println!("{{\"deleted\":true,\"id\":\"{}\"}}", selected.id);
            } else {
                println!("Deleted {}", selected.domain);
            }
            Ok(())
        }
    }
}

async fn provision_site(
    client: &ApiClient,
    args: AddSiteArgs,
    ensure: bool,
    json: bool,
) -> Result<()> {
    let domain = normalize_domain(&args.domain)?;
    let origins = if args.allowed_origins.is_empty() {
        vec![format!("https://{domain}")]
    } else {
        args.allowed_origins
    };
    let input = SiteInput {
        name: args.name.unwrap_or_else(|| domain.clone()),
        domain,
        timezone: args.timezone,
        allowed_origins: origins,
        retention_days: args.retention_days,
    };
    let (created, site) = if ensure {
        let result = client.ensure_site(&input).await?;
        (result.created, result.site)
    } else {
        (true, client.create_site(&input).await?)
    };
    let configured = if site.anti_adblock_server != args.server.as_str() {
        client
            .configure_tracking(
                site.id,
                &AntiAdblockInput {
                    server_type: args.server.as_str().into(),
                    js_path: site.anti_adblock_js_path.clone(),
                    beacon_path: site.anti_adblock_beacon_path.clone(),
                },
            )
            .await?
    } else {
        site
    };
    let setup = tracking_setup(&configured, client.base_url())?;
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "schemaVersion": 1,
                "ok": true,
                "data": { "created": created, "site": configured, "tracking": setup }
            }))?
        );
    } else {
        let outcome = if created {
            "Created site."
        } else {
            "Site already exists."
        };
        println!("{outcome}\n\n{}", human_setup(&setup));
    }
    Ok(())
}

async fn tracking_command(client: &ApiClient, command: TrackingCommand, json: bool) -> Result<()> {
    let sites = client.sites().await?;
    match command {
        TrackingCommand::Show { site } => {
            let setup = tracking_setup(find_site(&sites, &site)?, client.base_url())?;
            print_value(&setup, json, human_setup)
        }
        TrackingCommand::Configure(args) => {
            let selected = find_site(&sites, &args.site)?;
            let updated = client
                .configure_tracking(
                    selected.id,
                    &AntiAdblockInput {
                        server_type: args.server.as_str().into(),
                        js_path: args
                            .js_path
                            .unwrap_or_else(|| selected.anti_adblock_js_path.clone()),
                        beacon_path: args
                            .beacon_path
                            .unwrap_or_else(|| selected.anti_adblock_beacon_path.clone()),
                    },
                )
                .await?;
            let setup = tracking_setup(&updated, client.base_url())?;
            print_value(&setup, json, human_setup)
        }
    }
}

fn effective_auth(path: &Path, api_override: Option<&str>) -> Result<StoredAuth> {
    if let Ok(token) = std::env::var("SLIMLYTICS_TOKEN") {
        return Ok(StoredAuth {
            api_url: normalize_api_url(api_override.unwrap_or(DEFAULT_API_URL))?,
            token,
        });
    }
    let mut auth = load_auth(path)?;
    if let Some(value) = api_override {
        auth.api_url = normalize_api_url(value)?;
    }
    Ok(auth)
}

fn read_password(from_stdin: bool) -> Result<String> {
    if from_stdin {
        return read_secret_stdin("password");
    }
    if !io::stdin().is_terminal() {
        bail!("standard input is not a terminal; use --password-stdin");
    }
    rpassword::prompt_password("Slimlytics password: ").context("could not read password")
}

fn read_secret_stdin(name: &str) -> Result<String> {
    let mut value = String::new();
    io::stdin().read_to_string(&mut value)?;
    let value = value.trim_end_matches(['\r', '\n']).to_owned();
    if value.is_empty() {
        bail!("{name} cannot be empty");
    }
    Ok(value)
}

fn print_value<T: Serialize>(
    value: &T,
    json: bool,
    human: impl FnOnce(&T) -> String,
) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(value)?);
    } else {
        println!("{}", human(value));
    }
    Ok(())
}

fn human_setup(value: &slimlytics_cli::TrackingSetup) -> String {
    format!(
        "Site: {} ({})\n\n1. Add this server configuration and reload {}:\n\n{}\n\n2. Add this snippet before </body>:\n\n{}\n\n3. Test:\n   {}\n   {}",
        value.domain, value.site_id, value.server_type, value.server_config, value.snippet,
        value.script_test_url, value.beacon_test_url
    )
}
