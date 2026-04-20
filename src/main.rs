mod cap_db;
mod packet_cap;

use clap::{Parser, Subcommand};
use std::path::Path;
use std::time::Instant;

use crate::cap_db::create_sqlite_pool;
use crate::packet_cap::{Capture, IpCapture};

/* TODO: Refactor the whole command line. Actually learn how
* to use Clap. The following mess of structs and enums is
* the method that Claude told me to use. Therefore it is
* AI slop. I haven't even read the Clap documentation. I will
* need to fix this before expanding the scope of the project */
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
        /*TODO: Refactor this command to provide info about other
         * objects in the program: mac addrs, ip addrs, interfaces,
         * databases, settings, maybe other stuff */
    },
    Bind {
        iface_name: String,
    },
    Create {
        #[command(subcommand)]
        cmd: CreateCmds,
    },
    Edit {
        #[command(subcommand)]
        cmd: EditCmds,
    },
}

#[derive(Subcommand)]
enum ListCmds {
    Local, // list interfaces on local device only
    LAN,   // list interfaces on whole subnet
    Known, // List Known Addresses from anywhere
    Db,    // List sqlite databases. Always in project dir for now
}

#[derive(Subcommand)]
enum CreateCmds {
    Db,
    Settings,
}

#[derive(Subcommand)]
enum EditCmds {
    Db,
    Settings,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db_root = ".";
    let db_path = Path::new(db_root); //TODO: Allow user to change default path for sqlite db
    let cli = Cli::parse();
    let start = Instant::now();
    match &cli.cmd {
        Commands::List { cmd } => {
            match cmd {
                ListCmds::Local => packet_cap::cmd_list(),
                ListCmds::LAN => packet_cap::cmd_list(), //TODO: Need different function for listing other devices' interfaces. Need to send ARPpacket
                ListCmds::Known => packet_cap::cmd_list(), //TODO: Collect Addrs & store in known AddrDB
                ListCmds::Db => cap_db::list_databases(db_path),
            }
        }
        Commands::Info { iface_name } => {
            packet_cap::cmd_info(iface_name);
        }
        Commands::Bind { iface_name } => {
            if let Some(iface) = packet_cap::get_interface(&iface_name) {
                let pool = create_sqlite_pool("test.db").await?;
                let (sender, receiver) = tokio::sync::mpsc::channel::<IpCapture>(1024);
                std::thread::spawn(move || {
                    packet_cap::bind_and_listen(&iface, sender);
                });
                tokio::signal::ctrl_c().await.unwrap();
            }
        }
        Commands::Create { cmd } => match cmd {
            CreateCmds::Db => {
                let db_filename = "test.db"; //TODO: Eventually get this from settings.json
                cap_db::create_db(&db_filename).await?;
            }
            _ => {}
        },
        _ => {}
    }

    let duration = start.elapsed(); //TODO: Get this to print after a ctrl+c SIGTERM
    println!("Finished. Process ran for {:?}", duration);
    Ok(())
}
