mod audit;
mod commands;
mod detect;
mod profile;
mod util;

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use std::io;
use std::path::PathBuf;
use util::config::AppConfig;
use util::output::OutputOpts;
use util::xdg::XdgPaths;

#[derive(Parser, Debug)]
#[command(
    name = "browserprivacy",
    author = "r3dg0d",
    version,
    about = "Read-only privacy auditor for Chromium- and Firefox-family profiles on Linux",
    long_about = "Audits WebRTC, DoH, proxy, cookie policy, storage presence counts, \
service workers, site permissions, and extension inventory.\n\n\
NEVER extracts passwords, auth tokens, session cookies, or credentials. \
Login Data / key4.db / cookies DB values are skipped by design."
)]
struct Cli {
    #[arg(long, global = true)]
    json: bool,
    #[arg(long, short = 'v', global = true)]
    verbose: bool,
    #[arg(long, short = 'q', global = true)]
    quiet: bool,
    #[arg(long, global = true, env = "BROWSERPRIVACY_CONFIG")]
    config: Option<PathBuf>,
    #[arg(long, global = true)]
    dry_run: bool,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Audit all discovered profiles (default)
    Audit,
    /// List discovered browser profiles
    #[command(name = "list-profiles")]
    ListProfiles,
    /// Full privacy report with remediation suggestions
    Report,
    /// Generate shell completions
    Completions {
        #[arg(value_enum)]
        shell: Shell,
    },
}

fn init_tracing(verbose: bool, quiet: bool) {
    let level = if quiet {
        "error"
    } else if verbose {
        "debug"
    } else {
        "info"
    };
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(level)),
        )
        .with_writer(std::io::stderr)
        .try_init();
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    init_tracing(cli.verbose, cli.quiet);
    let out = OutputOpts {
        json: cli.json,
        quiet: cli.quiet,
        verbose: cli.verbose,
    };

    let command = cli.command.unwrap_or(Commands::Audit);

    if let Commands::Completions { shell } = command {
        let mut cmd = Cli::command();
        generate(shell, &mut cmd, "browserprivacy", &mut io::stdout());
        return Ok(());
    }

    let paths = XdgPaths::new()?;
    let cfg_path = cli
        .config
        .clone()
        .unwrap_or_else(|| paths.default_config_file());
    let _ = paths.ensure_all();
    let cfg = AppConfig::load(Some(&cfg_path))?;

    match command {
        Commands::Audit => commands::audit::run(&out, &cfg, cli.dry_run)?,
        Commands::ListProfiles => commands::list_profiles::run(&out, &cfg)?,
        Commands::Report => commands::report::run(&out, &cfg, cli.dry_run)?,
        Commands::Completions { .. } => unreachable!(),
    }
    Ok(())
}
