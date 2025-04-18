use clap::{Parser, Subcommand};
use anyhow::{Context, Result};
use std::path::PathBuf;

mod sandbox;
use sandbox::Sandbox;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser, Debug, Clone)]
struct SandboxOptions {
    #[arg(long)]
    block_net: bool,
    #[arg(long)]
    temp_dir: Option<PathBuf>,
    #[arg(long = "allow-read", value_name = "PATH")]
    allow_read_paths: Vec<PathBuf>,
    #[arg(long = "allow-write", value_name = "PATH")]
    allow_write_paths: Vec<PathBuf>,
}


#[derive(Subcommand)]
enum Commands {
    Run {
        program: String,
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
        #[command(flatten)]
        sandbox_opts: SandboxOptions,
    },
    Shell {
         #[command(flatten)]
        sandbox_opts: SandboxOptions,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { program, args, sandbox_opts } => {
             let allow_read_strings = sandbox_opts.allow_read_paths.into_iter()
                .filter_map(|p| p.to_str().map(String::from))
                .collect();
             let allow_write_strings = sandbox_opts.allow_write_paths.into_iter()
                .filter_map(|p| p.to_str().map(String::from))
                .collect();
             let temp_dir_string = sandbox_opts.temp_dir.as_ref().and_then(|p| p.to_str()).map(String::from);


            let sbx = Sandbox {
                block_net: sandbox_opts.block_net,
                temp_dir: temp_dir_string,
                allow_read_paths: allow_read_strings,
                allow_write_paths: allow_write_strings,
            };

            let program_path = which::which(&program)
                .context(format!("Failed to find program '{}' in PATH", program))?;
            let absolute_program_path = program_path.canonicalize()
                .context(format!("Failed to get canonical path for '{}'", program_path.display()))?;
            let program_str = absolute_program_path.to_str()
                .context("Program path is not valid UTF-8")?;

            println!("Running '{}' in sandbox...", program_str);
            sbx.run_command(program_str, &args)?;
        }
        Commands::Shell { sandbox_opts } => {
             let allow_read_strings = sandbox_opts.allow_read_paths.into_iter()
                .filter_map(|p| p.to_str().map(String::from))
                .collect();
             let allow_write_strings = sandbox_opts.allow_write_paths.into_iter()
                .filter_map(|p| p.to_str().map(String::from))
                .collect();
            let temp_dir_string = sandbox_opts.temp_dir.as_ref().and_then(|p| p.to_str()).map(String::from);

            let sbx = Sandbox {
                block_net: sandbox_opts.block_net,
                temp_dir: temp_dir_string,
                allow_read_paths: allow_read_strings,
                allow_write_paths: allow_write_strings,
            };

             let shell_path_str = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
             println!("Starting shell '{}' in sandbox...", shell_path_str);
             sbx.open_shell()?;
        }
    }

    Ok(())
}