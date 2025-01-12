use diem::{Cli, Commands};

use clap::Parser as _;

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Install { package } => {
            unimplemented!("Install package: {}", package);
        }
        Commands::Remove { package } => {
            unimplemented!("Uninstall package: {}", package);
        }
        Commands::Update { package } => {
            unimplemented!("Update package: {:?}", package);
        }
    }
}
