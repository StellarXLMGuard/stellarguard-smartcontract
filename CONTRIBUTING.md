# Contributing

## Setup

```bash
rustup target add wasm32v1-none
cargo build --release --target wasm32v1-none
cargo test --all
```

## Workflow

1. Pick an issue from the issue tracker
2. Fork the repo and create a feature branch
3. Write tests for your changes
4. Ensure `cargo fmt`, `cargo clippy`, and `cargo test --all` pass
5. Submit a PR against `main`

## Code Style

- `cargo fmt` enforces formatting
- `cargo clippy` must pass with zero warnings
- Use `#[contracttype]` enums for all storage keys
- Every privileged function must call `require_auth()` on the correct stored authority, not a caller-supplied address
- Emit typed events for every state change
- Extend storage TTLs in active write paths
