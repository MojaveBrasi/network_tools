#![allow(unused_imports, dead_code)]
mod pkt_cap;
use clap::{Args, Parser, Subcommand, builder::Str, command};

use crate::pkt_cap::bind_and_listen;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand)]
enum Commands {
    List,
    Info { iface_name: String },
    Bind { iface_name: String },
    Placeholder,
}

fn main() {
    let cli = Cli::parse();
    match &cli.cmd {
        Commands::List => {
            pkt_cap::list_interfaces();
        }
        Commands::Info { iface_name } => {
            if let Some(_i) = pkt_cap::get_interface(&iface_name) {
                // Just give info about interface unless it doesn't exist
            }
        }
        Commands::Bind { iface_name } => {
            if let Some(i) = pkt_cap::get_interface(&iface_name) {
                bind_and_listen(&i);
            }
        }
        _ => {}
    }
}
