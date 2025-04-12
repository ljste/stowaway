use anyhow::{Context, Result};
use tempfile::{tempdir, NamedTempFile, TempDir};
use std::{io::Write, path::Path, process::Command, fs};

pub struct Sandbox {
    pub block_net: bool,
    pub temp_dir: Option<String>,
}

const SANDBOX_PROFILE: &str = r#"
(version 1)
(deny network-outbound)
(allow default)
"#;

impl Sandbox {
    fn create_temp_directory(&self) -> Result<TempDir> {
        match &self.temp_dir {
            Some(path) => {
                fs::create_dir_all(path)?;
                tempdir().context("Failed to create temporary directory")
            },
            None => tempdir().context("Failed to create temporary directory")
        }
    }

    pub fn run_command(&self, program: &str, args: &[String]) -> Result<()> {
        let tmp_home = self.create_temp_directory()
            .context("Failed to create temp dir")?;
        let tmp_home_path = tmp_home.path();

        let maybe_profile = if self.block_net && cfg!(target_os = "macos") {
            Some(create_temp_profile()?)
        } else { None };

        let mut cmd = Self::prepare_command(program, args, tmp_home_path, maybe_profile.as_ref())?;

        let status = cmd.status().with_context(|| format!("Failed to run: {}", program))?;
        std::process::exit(status.code().unwrap_or(1));
    }

    pub fn open_shell(&self) -> Result<()> {
        let tmp_home = self.create_temp_directory()
            .context("Failed to create temp dir")?;
        let tmp_home_path = tmp_home.path();

        let shell_path = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());

        let maybe_profile = if self.block_net && cfg!(target_os = "macos") {
            Some(create_temp_profile()?)
        } else { None };

        let mut cmd = Self::prepare_command(&shell_path, &[], tmp_home_path, maybe_profile.as_ref())?;

        let status = cmd.status().context("Failed to start shell")?;
        std::process::exit(status.code().unwrap_or(1));
    }

    fn prepare_command(
        program: &str,
        args: &[String],
        tmp_home_path: &Path,
        maybe_profile: Option<&NamedTempFile>,
    ) -> Result<Command> {
        if let Some(profile) = maybe_profile {
            let mut cmd = Command::new("sandbox-exec");
            cmd.arg("-f").arg(profile.path());
            cmd.arg(program);
            cmd.args(args);
            cmd.env("HOME", tmp_home_path);
            Ok(cmd)
        } else {
            let mut cmd = Command::new(program);
            cmd.args(args);
            cmd.env("HOME", tmp_home_path);
            Ok(cmd)
        }
    }
}

fn create_temp_profile() -> Result<NamedTempFile> {
    let mut file = NamedTempFile::new().context("Failed to create sandbox-exec profile tmp file")?;
    file.write_all(SANDBOX_PROFILE.as_bytes())
        .context("Failed writing sandbox profile")?;
    file.flush().ok();
    Ok(file)
}