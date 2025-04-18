use anyhow::{Context, Result};
use tempfile::{NamedTempFile, TempDir};
use std::{io::Write, path::{Path, PathBuf}, process::Command, fs};

const MACOS_DEFAULT_READ_ALLOWS: &[(&str, bool)] = &[
    ("/", true),
    ("/bin", false),
    ("/sbin", false),
    ("/usr/bin", false),
    ("/usr/sbin", false),
    ("/usr/lib", false),
    ("/usr/libexec", false),
    ("/usr/share", false),
    ("/usr/local/bin", false),
    ("/System/Library", false),
    ("/Library/Frameworks", false),
    ("/Library/Preferences", false),
    ("/System/Volumes/Data/Library/Frameworks", false),
    ("/System/Volumes/Data/Library/Preferences", false),
    ("/System/Volumes/Preboot/Cryptexes", false),
    ("/private/var/db/datadetectors", false),
    ("/private/var/db/timezone", false),
    ("/System/Cryptexes", false),
    ("/private/etc/ssl", false),
    ("/private/etc/resolv.conf", true),
    ("/private/tmp", false),
    ("/dev/random", true),
    ("/dev/urandom", true),
    ("/dev/null", true),
    ("/dev/zero", true),
    ("/dev/dtracehelper", true),
    ("/dev/tty", true),
];

const MACOS_ESSENTIAL_OPERATIONS: &[&str] = &[
    "process-fork",
    "process-exec",
    "signal (target self)",
    "sysctl-read",
    "mach-lookup",
    "system-socket",
    "pseudo-tty",
    "file-ioctl (literal \"/dev/dtracehelper\")",
    "file-write-data (literal \"/dev/dtracehelper\")",
    "ipc-posix-shm-read*",
    "file-read-metadata (subpath \"/private/etc\")",
    "file-read-metadata (subpath \"/private/var\")",
];

#[derive(Debug)]
pub struct Sandbox {
    pub block_net: bool,
    pub temp_dir: Option<String>,
    pub allow_read_paths: Vec<String>,
    pub allow_write_paths: Vec<String>,
}

impl Sandbox {
     fn create_temp_directory(&self) -> Result<TempDir> {
        let mut builder = tempfile::Builder::new();
        let tmp_dir = match &self.temp_dir {
            Some(base_path) => {
                let base_path_buf = PathBuf::from(base_path);
                fs::create_dir_all(&base_path_buf)
                    .with_context(|| format!("Failed to create specified temp base directory: {}", base_path))?;
                let canonical_base = base_path_buf.canonicalize()
                    .with_context(|| format!("Failed to get canonical path for temp base directory: {}", base_path))?;
                builder.prefix("stowaway_sandbox_").tempdir_in(canonical_base)
            }
            None => builder.prefix("stowaway_sandbox_").tempdir(),
        }
        .context("Failed to create temporary directory")?;
        Ok(tmp_dir)
    }

    pub fn run_command(&self, program_path: &str, args: &[String]) -> Result<()> {
        let tmp_home = self.create_temp_directory()
            .context("Failed to create temporary home directory")?;
        let tmp_home_path = tmp_home.path();
        let canonical_tmp_home = tmp_home_path.canonicalize()
            .context("Failed to get canonical path for temporary home directory")?;
        let tmp_home_path_str = canonical_tmp_home.to_str()
            .context("Temporary home path is not valid UTF-8")?;

        println!("Temporary HOME: {}", tmp_home_path_str);

        let maybe_profile = if cfg!(target_os = "macos") {
            println!("Generating macOS sandbox profile...");
            match self.create_temp_profile(&canonical_tmp_home, Some(program_path)) {
                 Ok(file) => {
                     println!("Using sandbox profile: {}", file.path().display());
                     Some(file)
                 },
                 Err(e) => {
                     eprintln!("Warning: Failed to create sandbox profile: {:?}. Filesystem/network restrictions will not be applied.", e);
                     None
                 }
             }
        } else {
            self.warn_on_non_macos_restrictions();
            None
        };

        if cfg!(target_os = "macos") && maybe_profile.is_none() {
             anyhow::bail!("Sandbox profile creation failed, aborting execution.");
        }

        let mut cmd = Self::prepare_command(program_path, args, &canonical_tmp_home, maybe_profile.as_ref())?;

        println!("Executing sandboxed command: HOME={} CWD={} {:?}", tmp_home_path_str, tmp_home_path_str, cmd);

        let status = cmd.status().with_context(|| format!("Failed to run command: {}", program_path))?;

        println!("Command finished with status: {}", status);
        std::process::exit(status.code().unwrap_or(1));
    }

    pub fn open_shell(&self) -> Result<()> {
        let tmp_home = self.create_temp_directory()
            .context("Failed to create temporary home directory")?;
        let tmp_home_path = tmp_home.path();
         let canonical_tmp_home = tmp_home_path.canonicalize()
            .context("Failed to get canonical path for temporary home directory")?;
        let tmp_home_path_str = canonical_tmp_home.to_str()
           .context("Temporary home path is not valid UTF-8")?;

        println!("Temporary HOME: {}", tmp_home_path_str);

        let shell_path_str = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
         let shell_path = which::which(&shell_path_str)
             .context(format!("Failed to find shell '{}' in PATH", shell_path_str))?;
         let absolute_shell_path = shell_path.canonicalize()
             .context(format!("Failed to get canonical path for shell '{}'", shell_path.display()))?;
         let absolute_shell_path_str = absolute_shell_path.to_str()
            .context("Shell path is not valid UTF-8")?;

        let maybe_profile = if cfg!(target_os = "macos") {
            println!("Generating macOS sandbox profile for shell...");
             match self.create_temp_profile(&canonical_tmp_home, Some(absolute_shell_path_str)) {
                 Ok(file) => {
                     println!("Using sandbox profile: {}", file.path().display());
                     Some(file)
                 },
                 Err(e) => {
                     eprintln!("Warning: Failed to create sandbox profile: {:?}. Filesystem/network restrictions will not be applied.", e);
                     None
                 }
             }
        } else {
             self.warn_on_non_macos_restrictions();
            None
        };

         if cfg!(target_os = "macos") && maybe_profile.is_none() {
             anyhow::bail!("Sandbox profile creation failed, aborting execution.");
         }

        let mut cmd = Self::prepare_command(absolute_shell_path_str, &[], &canonical_tmp_home, maybe_profile.as_ref())?;

        println!("Executing sandboxed shell: HOME={} CWD={} {:?}", tmp_home_path_str, tmp_home_path_str, cmd);

        let status = cmd.status().context("Failed to start shell")?;

        println!("Shell exited with status: {}", status);
        std::process::exit(status.code().unwrap_or(1));
    }

    fn warn_on_non_macos_restrictions(&self) {
         if !self.allow_read_paths.is_empty() || !self.allow_write_paths.is_empty() || self.block_net {
             eprintln!("Warning: Filesystem/network restrictions (--allow-read/--allow-write/--block-net) are currently only supported on macOS via sandbox-exec.");
         }
    }

    fn prepare_command(
        program_path: &str,
        args: &[String],
        tmp_home_path: &Path,
        maybe_profile: Option<&NamedTempFile>,
    ) -> Result<Command> {
        let mut cmd: Command;

        if cfg!(target_os = "macos") {
            if let Some(profile) = maybe_profile {
                cmd = Command::new("/usr/bin/sandbox-exec");
                cmd.arg("-f").arg(profile.path());
                cmd.arg(program_path);
                cmd.args(args);
            } else {
                cmd = Command::new(program_path);
                cmd.args(args);
            }
        } else {
            cmd = Command::new(program_path);
            cmd.args(args);
        }

        cmd.env("HOME", tmp_home_path);
        cmd.current_dir(tmp_home_path);

        if let Ok(path_var) = std::env::var("PATH") {
            cmd.env("PATH", path_var);
         }
         if let Ok(term_var) = std::env::var("TERM") {
             cmd.env("TERM", term_var);
         }

        Ok(cmd)
    }

     fn create_temp_profile(&self, tmp_home_path: &Path, program_path: Option<&str>) -> Result<NamedTempFile> {
        let mut profile_content = String::from("(version 1)\n");
        profile_content.push_str("(deny default)\n\n");

        for op in MACOS_ESSENTIAL_OPERATIONS {
            profile_content.push_str(&format!("(allow {})\n", op));
        }
        profile_content.push_str("\n");


        profile_content.push_str("(allow file-read*\n");
        for (path, is_literal) in MACOS_DEFAULT_READ_ALLOWS {
             let is_root = path.as_bytes() == b"/";
             let is_dev = path.starts_with("/dev/");

             let processed_path = if is_dev || is_root {
                 path.to_string()
             } else {
                 match PathBuf::from(path).canonicalize() {
                     Ok(p) => p.to_string_lossy().to_string(),
                     Err(_) => path.to_string(),
                 }
             };

             let rule_type = if *is_literal { "literal" } else { "subpath" };
             profile_content.push_str(&format!("    ({} \"{}\")\n", rule_type, escape_sbpl_string(&processed_path)));
        }
        profile_content.push_str(")\n\n");


        if let Some(prog_path) = program_path {
             let prog_path_str = PathBuf::from(prog_path).to_string_lossy().to_string();
             profile_content.push_str(&format!("(allow file-read* (literal \"{}\"))\n", escape_sbpl_string(&prog_path_str)));

             if let Some(parent_dir) = PathBuf::from(prog_path).parent() {
                 let parent_dir_str = parent_dir.to_string_lossy();
                 profile_content.push_str(&format!("(allow file-read* (subpath \"{}\"))\n", escape_sbpl_string(&parent_dir_str)));
             }
             profile_content.push_str("\n");
         }

        let tmp_home_str = tmp_home_path.to_string_lossy();
        profile_content.push_str(&format!("(allow file* (subpath \"{}\"))\n", escape_sbpl_string(&tmp_home_str)));

        if let Some(parent_dir) = tmp_home_path.parent() {
             let parent_dir_str = parent_dir.to_string_lossy();
              profile_content.push_str(&format!("(allow file-read* (subpath \"{}\"))\n", escape_sbpl_string(&parent_dir_str)));
        }
        profile_content.push_str("\n");


        if !self.allow_read_paths.is_empty() {
             profile_content.push_str("; User allowed read paths\n");
             profile_content.push_str("(allow file-read*\n");
             for path_str in &self.allow_read_paths {
                 self.add_path_rule(&mut profile_content, path_str, false)?;
             }
             profile_content.push_str(")\n\n");
         }
         if !self.allow_write_paths.is_empty() {
              profile_content.push_str("; User allowed write paths\n");
              profile_content.push_str("(allow file*\n");
              for path_str in &self.allow_write_paths {
                  self.add_path_rule(&mut profile_content, path_str, true)?;
              }
              profile_content.push_str(")\n\n");
         }


        if self.block_net {
            profile_content.push_str("(deny network-outbound)\n");
        } else {
            profile_content.push_str("(allow network*)\n");
        }
        profile_content.push_str("\n");


        let mut file = NamedTempFile::new().context("Failed to create sandbox-exec profile tmp file")?;
        file.write_all(profile_content.as_bytes())
            .context("Failed writing sandbox profile")?;
        file.flush().context("Failed to flush sandbox profile")?;

        Ok(file)
    }

     fn add_path_rule(&self, profile_content: &mut String, path_str: &str, _is_write: bool) -> Result<()> {
         match PathBuf::from(path_str).canonicalize() {
             Ok(canonical_path) => {
                 let cpath_str = canonical_path.to_string_lossy();
                  let rule_type = if canonical_path.is_dir() { "subpath" } else { "literal" };
                 profile_content.push_str(&format!("    ({} \"{}\")\n", rule_type, escape_sbpl_string(&cpath_str)));
             },
             Err(e) => {
                  eprintln!("Warning: Could not canonicalize user-specified path '{}', skipping rule: {}", path_str, e);
             }
         }
         Ok(())
     }
}

fn escape_sbpl_string(s: &str) -> String {
     s.replace("\"", "\\\"")
}