// Dependencies
mod installer;
mod types;
use clap::Parser;
use installer::Servers;

/// Manage and create a Plutonium dedicated server.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Install server files to a given path
    #[arg(short, long, value_name="path")]
    server: Option<String>,

    /// Install server config to a given path
    #[arg(short, long, value_name="path")]
    config: Option<String>,

    /// Install IW4M Admin to a given path
    #[arg(short, long, value_name="path")]
    iw4m: Option<String>,

    /// Install IW4M Admin (log server) to a given path
    #[arg(short='l', long, value_name="path")]
    iw4m_log: Option<String>,

    /// Install Plutonium to a given path
    #[arg(short, long, value_name="path")]
    plutonium: Option<String>,

    /// Install a RCON client to a given path
    #[arg(short, long, value_name="path")]
    rcon: Option<String>,

    /// Specify the game version (must be provided if not only installing plutonium)
    #[arg(short, long, value_name="game")]
    engine: Option<Servers>
}

// Main
#[tokio::main]
async fn main() {
    // Parse the arguments
    let args = Args::parse();

    // Installation, if set...
    let engine = &args.engine;
    if args.server.is_some() {
        // Check version
        engine.as_ref().expect("specify the engine");

        // Install
        let server = args.server.unwrap();
        println!("Installing server files to {}.", server);
        installer::install_server(engine.as_ref().unwrap(), Some(&server)).await;
        println!("Installed server files.");
    }
    if args.config.is_some(){
        // Check version
        engine.as_ref().expect("specify the engine");

        // Install
        let config = args.config.unwrap();
        println!("Installing config files to {}.", config);
        installer::install_config(engine.as_ref().unwrap(), Some(&config)).await;
        println!("Installed config files.");
    }

    // No version needed for these...
    if args.iw4m.is_some() {
        let path = args.iw4m.unwrap();

        println!("Installing iw4m files to {}.", path);
        installer::install_iw4m(Some(&path)).await;
        println!("Installed iw4m files.");
    }
    if args.iw4m_log.is_some() {
        let path = args.iw4m_log.unwrap();

        println!("Installing iw4m log server files to {}.", path);
        installer::install_iw4m_log(Some(&path)).await;
        println!("Installed iw4m log server files.");
    }
    if args.plutonium.is_some() {
        let path = args.plutonium.unwrap();

        println!("Installing plutonium to {}.", path);
        installer::install_plutonium(Some(&path)).await;
        println!("Installed plutonium.");
    }
    if args.rcon.is_some() {
        let path = args.rcon.unwrap();

        println!("Installing rcon client to {}.", path);
        installer::install_rcon(Some(&path)).await;
        println!("Installed rcon client.");
    }
}
