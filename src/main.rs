#[allow(dead_code, unused_imports, unused_variables)]
mod packet_cap;
mod cap_db;

use std::path::Path;
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
        cmd: ListCmds,
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
enum ListCmds {
    Local, // list interfaces on local device only
    LAN,   // list interfaces on whole subnet
    Known, // List Known Addresses from anywhere
    Db,    // List sqlite databases. Always in project dir for now
}

#[tokio::main]
async fn main() {
    let database_path = Path::new("."); //TODO: Allow user to change default path for sqlite db
    let cli = Cli::parse();
    let start = Instant::now();
    match &cli.cmd {
        Commands::List { cmd } => {
            match cmd {
                ListCmds::Local => packet_cap::cmd_list(),
                ListCmds::LAN => packet_cap::cmd_list(), //TODO: Need different function for listing other devices' interfaces. Need to send ARPpacket
                ListCmds::Known => packet_cap::cmd_list(), //TODO: Collect Addrs & store in known AddrDB
                ListCmds::Db => cap_db::list_databases(database_path),
            }
        }
        Commands::Info { iface_name } => {
            packet_cap::cmd_info(iface_name);
        }
        Commands::Bind { iface_name } => {
            if let Some(i) = packet_cap::get_interface(&iface_name) {
                let (sender, mut _receiver) = tokio::sync::mpsc::channel::<Capture>(1024);
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
