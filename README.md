# stowaway

**Run untrusted programs in a disposable, isolated temporary environment – with optional macOS network sandboxing**

---

## Features

- Launches any program or shell within a **fresh, temporary HOME directory**
- Optionally **blocks network access**:
  - On **macOS**, enforced using [`sandbox-exec`](https://developer.apple.com/documentation/security/sandbox_exec)  
  - On other OSes, no network isolation yet
- CLI inspired by `cargo` syntax
- Built with pure Rust

---

## Usage

### Run a command in sandbox

```bash
cargo run -- run [--block-net] <program> [args...]
```

- `--block-net`: enforce no outbound network **if on macOS**
- Example:

```bash
cargo run -- run curl https://example.com
```

```bash
cargo run -- run --block-net curl https://example.com
```

### Launch an interactive shell

```bash
cargo run -- shell [--block-net]
```

- `--block-net`: disable outbound network in shell (macOS only)
- Example:

```bash
cargo run -- shell --block-net
```

---

## Notes & Caveats

- **Network block only works on macOS** right now  
  Uses a built-in `sandbox-exec` profile denying outbound connections.
- The network sandbox:
  - Blocks all TCP/UDP outgoing traffic
  - Works at **kernel level**: it's enforceable
  - May break programs that require Internet access
- On **Linux** and other platforms, `--block-net` is currently **NO-OP**
- All sandboxed processes run inside a **new empty temp `$HOME`**
- The sandbox **does not** isolate other capabilities:
  - File system access (besides HOME)
  - CPU/RAM resource usage
  - Process capabilities
- For even more isolation (namespaces, chroots etc.), external tools or future improvements are recommended.

---

## Requirements

- **Rust toolchain**
- **macOS** for network blocking (no extra config needed)
- On macOS, `sandbox-exec` is used automatically

---

## Installation / Building

```bash
git clone https://github.com/ljste/stowaway.git
cd stowaway
cargo build --release
```

Run with:

```bash
cargo run -- [COMMAND]
```

---

## Planned / TODO

- Real network-isolation on Linux (via namespaces, seccomp, firejail, nsjail etc.)
- Filesystem mounts isolation
- Configurable sandbox policies
- Profiles to customize restrictions
- Integration tests

---

## Disclaimer

This is **NOT a security boundary** for running malicious code.  
It is an experiment and a helper for contained environments.  
Never rely solely on this for isolating dangerous binaries.

---

## License

MIT