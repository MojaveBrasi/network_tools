#![allow(unused_imports, dead_code)]
mod ip;
mod packet_cap;
use clap::{Parser, Subcommand};

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
            packet_cap::cmd_list();
        }
        Commands::Info { iface_name } => {
            packet_cap::cmd_info(iface_name);
        }
        Commands::Bind { iface_name } => {
            if let Some(i) = packet_cap::get_interface(&iface_name) {
                packet_cap::bind_and_listen(&i);
            }
        }
        _ => {}
    }
}
