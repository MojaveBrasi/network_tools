mod application_state;
mod database;
mod network;

use crate::application_state::State;
use crate::database::*;
use crate::network::*;
use clap::{Parser, Subcommand, ValueEnum};
use std::time::Instant;

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
    #[command(alias = "m")]
    Create { db_name: String },
    /// List info of given database
    #[command(alias = "i")]
    Info { db_name: String },
    /// List known databases
    #[command(alias = "l")]
    List,
    /// List directory of last used database
    Dir,
    /// List size of given databases
    #[command(alias = "s")]
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
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let state = State::init();
    let cli = Cli::parse();

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
            DatabaseCommands::Create { db_name } => {
                let db = create_db(db_name).await?;
            }
            DatabaseCommands::Info { db_name } => match database_info(db_name).await {
                Ok(schema) => {
                    println!("Rows in database: {}", schema.row_count);
                    for c in schema.rows {
                        println!(">>>{}", c);
                    }
                }
                Err(e) => {
                    println!("damn. {}", e);
                }
            },
            DatabaseCommands::List => {
                list_databases(&state.default_db_dir);
            }
            DatabaseCommands::Dir => {
                println!(
                    "Current primary database directory: '{}'",
                    state.default_db_dir
                );
            }
            DatabaseCommands::Size { db_name } => todo!(),
        },
        Commands::Settings { cmd } => match cmd {
            SettingsCommands::Create { settings_name } => todo!(),
            SettingsCommands::Info { settings_name } => todo!(),
        },
        Commands::Bind { iface_name } => {
            if let Some(iface) = get_interface(&iface_name) {
                tokio::spawn(async move {
                    let start = Instant::now();
                    tokio::signal::ctrl_c().await.unwrap();
                    let duration = start.elapsed();
                    println!("");
                    println!("<< Finished. Process ran for {:?} >>", duration);
                    std::process::exit(0);
                });
                let pool = create_sqlite_pool("test.db").await?;
                let (sender, receiver) = tokio::sync::mpsc::channel::<IpCapture>(1024);
                std::thread::spawn(move || {
                    println!("<<< STARTING PACKET CAPTURE >>>");
                    network::bind_and_listen(&iface, sender);
                });
                database::write_captures_to_db(receiver, pool).await;
            }
        }
    }

    Ok(())
}
