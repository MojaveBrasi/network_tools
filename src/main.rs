#[allow(dead_code, unused_imports)]
mod packet_cap;
use clap::{Parser, Subcommand};
use std::time::Instant;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand)]
enum Commands {
    List {
        #[command(subcommand)]
        cmd: Scope,
    },
    Info {
        iface_name: String,
    },
    Bind {
        iface_name: String,
    },
    Placeholder,
}

#[derive(Subcommand)]
enum Scope {
    Local, // list interfaces on local device only
    LAN,   // list interfaces on whole subnet
}

fn main() {
    let cli = Cli::parse();
    let start = Instant::now();
    match &cli.cmd {
        Commands::List { cmd } => {
            match cmd {
                Scope::Local => packet_cap::cmd_list(),
                Scope::LAN => packet_cap::cmd_list(), //TODO: Need different function. Send ARP
                                                      //packet
            }
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

    let duration = start.elapsed(); //TODO: Get this to print after a ctrl+c SIGTERM
    println!("Ran for {:?}", duration);
}
