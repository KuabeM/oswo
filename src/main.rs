use std::path::PathBuf;

use clap::{Parser, Subcommand};
use color_eyre::Result;

mod cfg;
mod outputs;

/// Organise sway outputs (oswo).
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Show current configuration
    #[command(subcommand)]
    cmds: Cmds,
    /// Verbosity of output
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,
}

#[derive(Subcommand, Debug)]
enum Cmds {
    /// Show currently active configuration.
    #[command(alias = "d")]
    Display,
    /// Manually activate displays. Displays are arranged as order of args, left to right.
    #[command(alias = "s")]
    Set {
        /// Setup of outputs
        setup: Vec<String>,
    },
    /// Activate a pre-defined display configuration.
    #[command(alias = "u")]
    Use {
        /// Name of predefined configuration.
        config: String,
        /// Path to toml file containing predefined configurations.
        #[arg(short, long)]
        cfg_file: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    let cfg = std::fs::read_to_string("./cfgs.toml")?
        .parse::<toml::Table>()
        .unwrap();

    let outputs = outputs::Outputs::list()?;
    match args.cmds {
        Cmds::Display if args.verbose == 0 => println!("{}", outputs),
        Cmds::Display => println!("{:#}", outputs),
        Cmds::Set { setup } => outputs.set(&setup)?,
        Cmds::Use { config, cfg_file } => {
            let path = cfg_file.unwrap_or_else(|| "~/.config/oswo.toml".into());
            let cfg_str = std::fs::read_to_string(path)?;
            let cfgs: cfg::Cfgs = toml::from_str(cfg_str);
        }
    }

    Ok(())
}
