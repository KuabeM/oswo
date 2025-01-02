use std::{collections::BTreeSet, path::PathBuf};

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
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();
    env_logger::init();

    let cfg = args.cfg_file.unwrap_or(Cfgs::default_path());
    let cfgs = Cfgs::from_file(cfg).wrap_err("Failed to load configuration")?;

    info!("subscribing to output changes");
    let event_ty = [EventType::Output];
    let connection = Connection::new()?;
    let sub = connection.subscribe(event_ty)?;

    let mut last_outputs = Outputs::list()?;
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
                let connected_names: BTreeSet<String> =
                    outputs.iter().map(|o| o.model().to_string()).collect();
                trace!("connected displays: {:?}", connected_names);
                let mut valid_cfgs = Vec::new();
                for (k, v) in cfgs.iter() {
                    let names: BTreeSet<_> = v.iter().map(|d| d.name.clone()).collect();
                    if names.is_subset(&connected_names) {
                        valid_cfgs.push((k, v));
                    }
                }
                valid_cfgs.sort_by_key(|a| a.1.len());
                trace!("relevant cfgs: {:?}", valid_cfgs);
                if let Some(best_cfg) = valid_cfgs.last() {
                    info!("activating config '{}'", best_cfg.0);
                    outputs.set_models(best_cfg.1)?;
                    last_outputs = outputs;
                }
            }
            _ => unreachable!("can't receive unsubscribed event"),
        }
    }

    Ok(())
}
