# stowaway

**Run untrusted programs in a disposable, isolated temporary environment – with optional macOS network sandboxing**

---

## Features

- Launches any program or shell within a **fresh, temporary HOME directory**
- Optional **custom temporary directory** location
- Optionally **blocks network access**:
  - On **macOS**, enforced using [`sandbox-exec`](https://developer.apple.com/documentation/security/sandbox_exec)  
  - On other OSes, no network isolation yet
- CLI inspired by `cargo` syntax
- Built with pure Rust

---

## Usage

### Run a command in sandbox

```bash
cargo run -- run [--block-net] [--temp-dir <path>] <program> -- [args...]
```

- `--block-net`: enforce no outbound network **if on macOS**
- `--temp-dir`: specify a custom path for the temporary directory
- Examples:

```bash
# Basic command
cargo run -- run ls -- -la

# With network blocking
cargo run -- run --block-net curl -- https://example.com

# With custom temporary directory
cargo run -- run --temp-dir ./my-sandbox ls -- -la

# Complex command with multiple arguments
cargo run -- run python -- -c "print('hello world')"
```

### Launch an interactive shell

```bash
cargo run -- shell [--block-net] [--temp-dir <path>]
```

- `--block-net`: disable outbound network in shell (macOS only)
- `--temp-dir`: specify a custom path for the temporary directory
- Examples:

```bash
# Start shell with network blocking
cargo run -- shell --block-net

# Start shell with custom sandbox directory
cargo run -- shell --temp-dir ./debug-sandbox
```

**Note**: The `--` separator is required before command arguments to prevent them from being interpreted as arguments to stowaway itself.