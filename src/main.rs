use clap::{Parser, Subcommand};
use color_eyre::Result;

mod show;

/// Organise sway outputs (oswo).
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Show current configuration
    #[command(subcommand)]
    cmds: Option<Cmds>,
}

#[derive(Subcommand, Debug)]
enum Cmds {
    Show
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    if let Some(sub) = args.cmds {
        match sub {
            Cmds::Show => println!("{}", show::Outputs::list()?),
        }
    }

    Ok(())
}
