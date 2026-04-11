#[allow(dead_code, unused_imports, unused_variables)]
mod packet_cap;
use clap::{Parser, Subcommand};
use std::time::Instant;

use crate::packet_cap::Capture;

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
    Known, // List Known Addresses from anywhere
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let start = Instant::now();
    match &cli.cmd {
        Commands::List { cmd } => {
            match cmd {
                Scope::Local => packet_cap::cmd_list(),
                Scope::LAN => packet_cap::cmd_list(), //TODO: Need different function for listing other devices' interfaces. Need to send ARPpacket
                Scope::Known => packet_cap::cmd_list(), //TODO: Collect Addrs & store in known AddrDB
            }
        }
        Commands::Info { iface_name } => {
            packet_cap::cmd_info(iface_name);
        }
        Commands::Bind { iface_name } => {
            if let Some(i) = packet_cap::get_interface(&iface_name) {
                let (sender, mut _reciever) = tokio::sync::mpsc::channel::<Capture>(1024);
                std::thread::spawn(move || {
                    packet_cap::bind_and_listen(&i, sender);
                });
                //TODO: Tokio spawn join handle. Write to DB here
                tokio::signal::ctrl_c().await.unwrap();
            }
        }
        _ => {}
    }

    let duration = start.elapsed(); //TODO: Get this to print after a ctrl+c SIGTERM
    println!("Finished. Process ran for {:?}", duration);
}
