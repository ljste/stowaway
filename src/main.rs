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
        #[arg(long)]
        temp_dir: Option<String>,
    },
    Shell {
        #[arg(long)]
        block_net: bool,
        #[arg(long)]
        temp_dir: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { program, args, block_net, temp_dir } => {
            let sbx = Sandbox { block_net, temp_dir };
            sbx.run_command(&program, &args)?;
        }
        Commands::Shell { block_net, temp_dir } => {
            let sbx = Sandbox { block_net, temp_dir };
            sbx.open_shell()?;
        }
    }

    Ok(())
}