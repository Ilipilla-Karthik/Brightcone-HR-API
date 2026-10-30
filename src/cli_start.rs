//! Migration-free command entry point.
//!
//! The stock Loco CLI (`loco_rs::cli::main`) is hard-coupled to
//! `MigratorTrait`. This project intentionally runs **without** database
//! migrations: the database connection is created in `boot::create_context`,
//! routes and workers are wired in `boot::run_app`, and the entity-first
//! schema synchronizer (`initializers::schema_sync`) runs during
//! `after_routes`. This module keeps the standard `start` behaviour with no
//! migration machinery anywhere in the startup path.
use loco_rs::app::Hooks;
use loco_rs::boot::{create_context, run_app, start, ServeParams, StartMode};
use loco_rs::environment::{resolve_from_env, Environment};

use crate::app::App;

/// Runs the CLI: [`App::load_config`] plus the runner with the parsed flags.
///
/// Supported usage (compatible with the usual Loco start flags):
/// `start [--server] [--server-and-worker] [--all] [--worker] [--scheduler]
/// [--environment <name>] [--port <n>] [--no-banner]`
///
/// # Errors
/// Propagates configuration, boot or serve failures from Loco.
pub async fn run() -> loco_rs::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut it = args.iter();
    let mut env_name = resolve_from_env();
    let mut port: Option<i32> = None;
    let mut no_banner = false;
    let mut mode = StartMode::ServerOnly;

    while let Some(arg) = it.next() {
        match arg.as_str() {
            "start" | "--server" => mode = StartMode::ServerOnly,
            "--server-and-worker" => mode = StartMode::ServerAndWorker,
            "--all" => mode = StartMode::All,
            "--worker" => mode = StartMode::WorkerOnly { tags: vec![] },
            "--scheduler" => mode = StartMode::ServerAndScheduler,
            "--environment" | "-e" => {
                if let Some(v) = it.next() {
                    env_name = v.clone();
                }
            }
            "--port" => {
                if let Some(v) = it.next() {
                    port = v.parse().ok();
                }
            }
            "--no-banner" => no_banner = true,
            // Unrecognised flags (e.g. supplied by tooling) are ignored so the
            // binary stays usable from wrappers.
            _ => {}
        }
    }

    let environment = Environment::from(env_name);
    let config = App::load_config(&environment).await?;
    let app_context = create_context::<App>(&environment, config).await?;
    let boot = run_app::<App>(&mode, app_context).await?;
    let serve_params = ServeParams {
        port: port.unwrap_or(boot.app_context.config.server.port),
        binding: boot.app_context.config.server.binding.clone(),
    };
    start::<App>(boot, serve_params, no_banner).await?;
    Ok(())
}
