use std::path::PathBuf;

use clap::{command, Parser};
use color_eyre::{eyre::Context, Result};
use log::{info, trace};
use swayipc::{Connection, Event, EventType};

use liboswo::{Cfgs, Outputs};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to toml file containing predefined configurations. [$XDG_CONFIG_DIR/oswo.toml]
    #[arg(short, long)]
    cfg_file: Option<PathBuf>,
    /// Forward log messages to syslog.
    #[arg(short, long)]
    syslog: bool,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    if args.syslog {
        syslog::init(
            syslog::Facility::LOG_USER,
            log::LevelFilter::Info,
            Some(env!("CARGO_PKG_NAME")),
        )?;
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn"))
            .format_timestamp(None)
            .init();
    }

    let cfg = args.cfg_file.unwrap_or(Cfgs::default_path());
    let cfgs = Cfgs::from_file(cfg).wrap_err("Failed to load configuration")?;

    info!("subscribing to output changes");
    let event_ty = [EventType::Output];
    let connection = Connection::new()?;
    let sub = connection.subscribe(event_ty)?;

    let mut last_outputs = Outputs::list()?;
    last_outputs.activate_config(&cfgs)?;

    for event in sub {
        let event = event?;
        trace!("new output event");

        match event {
            Event::Output(_) => {
                let outputs = Outputs::list()?;
                if last_outputs == outputs {
                    trace!("no output changes");
                    continue;
                }
                outputs.activate_config(&cfgs)?;
                last_outputs = outputs;
            }
            _ => unreachable!("can't receive unsubscribed event"),
        }
    }

    Ok(())
}
