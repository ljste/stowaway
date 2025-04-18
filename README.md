# stowaway

**Run programs in a disposable, isolated temporary environment – with macOS filesystem and network sandboxing.**

---

## Features

- Launches any program or shell within a **fresh, temporary HOME directory**.
- Optional **custom temporary directory** location using `--temp-dir`.
- **Sandboxing (macOS only):** Uses [`sandbox-exec`](https://developer.apple.com/documentation/security/sandbox_exec) on macOS with a dynamically generated, deny-by-default profile to provide:
    - **Filesystem Isolation:** Restricts file access, allowing essential system reads and confining writes primarily to the temporary HOME directory.
    - **Optional Network Blocking:** Use `--block-net` to deny outbound network connections.
    - **Path Permissions:** Use `--allow-read <PATH>` and `--allow-write <PATH>` to grant specific permissions to host paths (macOS only).
- Sandboxing features beyond temporary HOME are **not yet implemented** for other operating systems.
- CLI inspired by `cargo` syntax.
- Built with pure Rust.

---

## Usage

**Important:** On macOS, the sandbox profile is quite strict (`deny default`). While common paths are allowed, complex programs might require additional permissions not yet included or might fail due to sandbox restrictions. Use the `--allow-read` and `--allow-write` flags to grant necessary access.

### Run a command in sandbox

```bash
cargo run -- run [OPTIONS] <program> -- [args...]
```

**Options (macOS only unless specified):**

- `--block-net`: Enforce no outbound network connections.
- `--temp-dir <path>`: Specify a custom path for the temporary directory base (works on all OS).
- `--allow-read <path>`: Allow read access to the specified host path inside the sandbox. Can be used multiple times.
- `--allow-write <path>`: Allow read *and* write access to the specified host path inside the sandbox. Can be used multiple times.

**Examples:**

```bash
# Basic command (runs in temp HOME)
cargo run -- run ls -- -la

# Block network access
cargo run -- run --block-net curl -- [https://example.com](https://example.com)

# Allow reading a specific file
cargo run -- run --allow-read /path/to/your/file.txt cat -- /path/to/your/file.txt

# Allow writing to a specific directory
cargo run -- run --allow-write /path/to/output_dir touch -- /path/to/output_dir/newfile.txt

# Custom temporary directory
cargo run -- run --temp-dir ./my-sandbox ls -- -la

# Complex command with multiple arguments
cargo run -- run python -- -c "print('hello world')"
```

### Launch an interactive shell

```bash
cargo run -- shell [OPTIONS]
```

**Options (macOS only unless specified):**

- `--block-net`: Disable outbound network connections in the shell.
- `--temp-dir <path>`: Specify a custom path for the temporary directory base (works on all OS).
- `--allow-read <path>`: Allow read access to the specified host path inside the shell.
- `--allow-write <path>`: Allow read/write access to the specified host path inside the shell.

**Examples:**

```bash
# Start shell with network blocking
cargo run -- shell --block-net

# Start shell allowing access to a project directory
cargo run -- shell --allow-write /Users/lj/Projects/my_code
```

**Note**: The `--` separator is required before passing arguments to the `<program>` in the `run` subcommand to prevent them from being interpreted as arguments to `stowaway` itself.

---

## Limitations (macOS Sandboxing)

- The `deny default` profile is strict and may block legitimate operations needed by complex tools. Discovering all necessary `allow` rules can be challenging.
- Known issues:
    - Running `curl` *without* `--block-net` currently fails due to blocked system interactions needed for DNS resolution (persistent `file-read-metadata` denials for `/etc` and `/var`).
    - Using `--allow-read` for files within certain system directories (like `/tmp`) may not work as expected due to related directory access denials.
- The list of allowed system paths and operations may need further refinement for broader compatibility. Contribution and testing are welcome!
```

This version updates the features, adds the new flags to the usage examples, and includes a limitations section explaining the current status of the macOS sandbox. You can copy and paste this into your `README.md`.