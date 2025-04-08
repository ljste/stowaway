use clap::{Parser, Subcommand};
use anyhow::Result;

mod sandbox;
use sandbox::Sandbox;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run {
        program: String,
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
        #[arg(long)]
        block_net: bool,
    },
    Shell {
        #[arg(long)]
        block_net: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { program, args, block_net } => {
            let sbx = Sandbox { block_net };
            sbx.run_command(&program, &args)?;
        }
        Commands::Shell { block_net } => {
            let sbx = Sandbox { block_net };
            sbx.open_shell()?;
        }
    }

    Ok(())
}
