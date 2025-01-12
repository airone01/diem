use clap_complete::{generate, Generator};
use diem::{Cli, Commands};

use clap::{Command, CommandFactory as _, Parser as _};

#[tokio::main]
async fn main() {
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
        Commands::Completions { shell } => {
            eprintln!("Generating completion file for {shell}...");
            print_completions(shell, &mut Cli::command());
        }
    }
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut std::io::stdout());
}
