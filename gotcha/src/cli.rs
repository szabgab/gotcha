use std::marker::PhantomData;

use clap::{Args, Parser, Subcommand};
use serde::de::DeserializeOwned;
use tracing::info;

use crate::config::GotchaConfigLoader;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
pub struct Cli {
    #[clap(long, short)]
    profile: Option<String>,

    #[command(subcommand)]
    command: GotchaCommand,
}

#[derive(Subcommand, Debug)]
pub enum GotchaCommand {
    Run(RunOpts),

    #[clap(subcommand)]
    Migration(MigrationOpts),
}

#[derive(Subcommand, Debug)]
pub enum MigrationOpts {}

#[derive(Args, Debug)]
pub struct RunOpts {}

pub struct GotchaCli<F, CONFIG: DeserializeOwned = (), const HAS_SERVER_FN: bool = false> {
    server_fn: Option<F>,
    config: PhantomData<CONFIG>,
}

impl<F, FR, CONFIG> GotchaCli<F, CONFIG, false>
where
    F: (Fn(CONFIG) -> FR) + Sync + 'static,
    FR: std::future::Future<Output = Result<(), std::io::Error>> + 'static,
    CONFIG: DeserializeOwned,
{
    pub fn new() -> Self {
        Self {
            server_fn: None,
            config: PhantomData,
        }
    }

    pub fn server(self, f: F) -> GotchaCli<F, CONFIG, true> {
        GotchaCli {
            server_fn: Some(f),
            config: self.config,
        }
    }
}

impl<F, FR, CONFIG> GotchaCli<F, CONFIG, true>
where
    F: (Fn(CONFIG) -> FR) + Sync + 'static,
    FR: std::future::Future<Output = Result<(), std::io::Error>> + 'static,
    CONFIG: DeserializeOwned,
{
    pub async fn run(self) -> () {
        tracing_subscriber::fmt::init();
        let opts = Cli::parse();
        let profile = opts.profile.or(std::env::var("GOTCHA_ACTIVE_PROFILE").ok());
        info!("starting gotcha");
        match opts.command {
            GotchaCommand::Run(_) => {
                match profile.as_ref() {
                    Some(env) => info!("gotcha is loading with profile [{}]", env),
                    None => info!("gotcha is loading without any extra profile"),
                };
                let config: CONFIG = GotchaConfigLoader::load(profile);
                let server_fn = self.server_fn.unwrap();
                server_fn(config).await.expect("error");
            }
            GotchaCommand::Migration(_) => todo!(),
        }
    }
}
