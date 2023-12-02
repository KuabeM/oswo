use std::path::PathBuf;

use clap::{Parser, Subcommand};
use color_eyre::{
    eyre::{self, Context},
    Result,
};

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
        /// Path to toml file containing predefined configurations. [$XDG_CONFIG_DIR/oswo.toml]
        #[arg(short, long)]
        cfg_file: Option<PathBuf>,
    },
    /// Print all pre-defined configurations.
    #[command(alias = "p")]
    Print {
        /// Path to toml file containing predefined configurations. [$XDG_CONFIG_DIR/oswo.toml]
        #[arg(short, long)]
        cfg_file: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    let default_cfg = dirs::config_dir()
        .unwrap_or("/etc/xdg/".into())
        .join("oswo.toml");
    let outputs = outputs::Outputs::list()?;
    match args.cmds {
        Cmds::Display if args.verbose == 0 => println!("{}", outputs),
        Cmds::Display => println!("{:#}", outputs),
        Cmds::Set { setup } => outputs.set_by_name(&setup)?,
        Cmds::Use { config, cfg_file } => {
            let cfg = cfg_file.unwrap_or(default_cfg);
            let cfgs = cfg::Cfgs::from_file(cfg).wrap_err("Failed to load configuration")?;
            let desired_outputs = cfgs
                .find(&config)
                .ok_or_else(|| eyre::eyre!("Found no setup for '{}'", config))?;
            outputs.set_models(&desired_outputs[..])?;
        }
        Cmds::Print { cfg_file } => {
            let cfg = cfg_file.unwrap_or(default_cfg);
            let cfgs = cfg::Cfgs::from_file(cfg).wrap_err("Failed to load configuration")?;
            println!("{:?}", cfgs);
        }
    }

    Ok(())
}
