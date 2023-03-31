use clap::{Parser, Subcommand};
use color_eyre::Result;

mod show;

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
    Show,
    Set {
        /// Setup of outputs
        setup: Vec<String>,
    },
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    let outputs = show::Outputs::list()?;
    match args.cmds {
        Cmds::Show if args.verbose == 0 => println!("{}", outputs),
        Cmds::Show => println!("{:#}", outputs),
        Cmds::Set { setup } => outputs.set(&setup)?,
    }

    Ok(())
}
