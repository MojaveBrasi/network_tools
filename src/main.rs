mod database;
mod network;

use clap::{Parser, Subcommand, ValueEnum};
use std::path::Path;
use std::time::Instant;

use crate::database::*;
use crate::network::*;

// I refactored the whole parser and now it's sexy
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(aliases = ["iface", "i"])]
    Interface {
        #[command(subcommand)]
        cmd: InterfaceCommands,
    },
    #[command(aliases = ["addr", "a"])]
    Address {
        #[command(subcommand)]
        cmd: AddressCommands,
    },
    #[command(aliases = ["db", "d"])]
    Database {
        #[command(subcommand)]
        cmd: DatabaseCommands,
    },
    #[command(aliases = ["set", "s"])]
    Settings {
        #[command(subcommand)]
        cmd: SettingsCommands,
    },
    #[command(alias = "b")]
    Bind { iface_name: String },
}

#[derive(ValueEnum, Clone, Copy)]
enum Scope {
    Local,
    Lan,
    Known,
}

#[derive(Subcommand)]
enum InterfaceCommands {
    /// Get info about a given interface
    Info { iface_name: String },
    /// List Interfaces in a given scope
    List {
        #[arg(value_enum, default_value_t = Scope::Local)]
        scope: Scope,
    },
}

#[derive(Subcommand)]
enum AddressCommands {
    /// List info of given addresss
    Info { addr_name: String },
    List {
        #[arg(value_enum, default_value_t = Scope::Local)]
        scope: Scope,
    },
}

#[derive(Subcommand)]
enum DatabaseCommands {
    /// Create db with given name in given dir
    /// Default name if none provided: depends on capture type
    /// Default directory if none provided: "." unless specified in settings
    Create { db_dir: String },
    /// List info of given database
    Info { db_name: String },
    /// List known databases
    List,
    /// List directory of last used database
    Dir,
    /// List size of given databases
    Size { db_name: String },
}

#[derive(Subcommand)]
enum SettingsCommands {
    /// Create Settings with given name
    Create { settings_name: String },
    /// List info of given settings
    Info { settings_name: String },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db_root = ".";
    let db_path = Path::new(db_root); //TODO: Allow user to change default path for sqlite db
    let cli = Cli::parse();
    let start = Instant::now();
    match &cli.cmd {
        Commands::Interface { cmd } => match cmd {
            InterfaceCommands::Info { iface_name } => interface_info(iface_name),
            InterfaceCommands::List { scope } => match scope {
                Scope::Local => interface_list_local(),
                Scope::Lan => todo!(),
                Scope::Known => todo!(),
            },
        },
        Commands::Address { cmd } => match cmd {
            AddressCommands::Info { addr_name } => todo!(),
            AddressCommands::List { scope } => match scope {
                Scope::Local => todo!(),
                Scope::Lan => todo!(),
                Scope::Known => todo!(),
            },
        },
        Commands::Database { cmd } => match cmd {
            DatabaseCommands::Create { db_dir } => {
                let db = create_db(db_dir).await?;
            }
            DatabaseCommands::Info { db_name } => todo!(),
            DatabaseCommands::List => {
                list_databases(db_path);
            }
            DatabaseCommands::Dir => {
                println!("Current primary database directory: {}", db_root);
            }
            DatabaseCommands::Size { db_name } => todo!(),
        },
        Commands::Settings { cmd } => match cmd {
            SettingsCommands::Create { settings_name } => todo!(),
            SettingsCommands::Info { settings_name } => todo!(),
        },
        Commands::Bind { iface_name } => {
            if let Some(iface) = get_interface(&iface_name) {
                let pool = create_sqlite_pool("test.db").await?;
                let (sender, receiver) = tokio::sync::mpsc::channel::<IpCapture>(1024);
                std::thread::spawn(move || {
                    network::bind_and_listen(&iface, sender);
                });
                tokio::signal::ctrl_c().await.unwrap();
            }
        }
    }

    let duration = start.elapsed();
    println!("Finished. Process ran for {:?}", duration);
    Ok(())
}
