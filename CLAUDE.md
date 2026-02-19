# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

VirtualGhost is a single-binary Rust application that runs "Ghostly Term" — an isolated terminal with SSH — inside a Firecracker microVM. The host binary manages the VM lifecycle, connects via vsock, and presents a TUI.

## Build Commands

```bash
cargo check                                                    # Type-check host binary
cargo build                                                    # Build host binary (debug)
cargo build --release                                          # Build host binary (release, optimized for size)
cargo check --manifest-path guest/ghostly-agent/Cargo.toml     # Type-check guest agent
cargo build --manifest-path guest/ghostly-agent/Cargo.toml     # Build guest agent
```

The guest agent targets Linux (runs inside a Firecracker VM). Cross-compile with:
```bash
cargo build --manifest-path guest/ghostly-agent/Cargo.toml --target x86_64-unknown-linux-musl --release
```

## Architecture

```
Host Binary (virtualghost)
├── cli.rs           — clap CLI (run/config/clean subcommands)
├── config/          — TOML config, defaults, settings structs
├── vm/              — Firecracker process lifecycle, REST API client, models, asset extraction
├── network/         — Vsock host-side connection and byte-stream tunnel (unix-only)
├── ssh/             — russh SSH client, session/PTY management, ephemeral key generation
├── terminal/        — ratatui TUI: event loop (app.rs), rendering (ui.rs), input mapping, VT100 emulator (vterm.rs)
└── error.rs         — Unified error types with thiserror

Guest Binary (ghostly-agent)
└── server.rs        — Vsock listener + russh SSH server + PTY manager (linux-only)
```

**Data flow:** User input → crossterm → SSH client → vsock tunnel → Firecracker VM → guest agent (SSH server + PTY) → output flows back → VT parser → ratatui render

## Key Design Decisions

- **Vsock over TAP networking**: No iptables/root needed; Firecracker maps guest vsock ports to host Unix sockets
- **Ephemeral SSH keys**: Fresh Ed25519 keypair per session, injected via vsock sideband
- **Unix-gated code**: `vm/api.rs`, `vm/process.rs`, `vm/config.rs`, and `network/` are `#[cfg(unix)]` — the TUI and config work on any platform
- **Separate guest crate**: `guest/ghostly-agent/` is compiled independently for `x86_64-unknown-linux-musl`
- **VT emulation**: `vte` crate parses escape sequences into a separate `TermState` struct (avoids borrow issues), rendered by ratatui

## Conventions

- Error types live in `src/error.rs` using thiserror; each module has its own error enum that converts via `From`
- Firecracker API types in `src/vm/models.rs` mirror the REST API (serde Serialize/Deserialize)
- All async code uses tokio; SSH uses russh 0.48 with `ssh-key` crate types
- Release profile: LTO + strip + codegen-units=1 + opt-level="z" (size-optimized)
