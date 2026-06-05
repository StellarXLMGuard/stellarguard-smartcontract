# StellarGuard Smart Contracts

Soroban smart contracts powering the StellarGuard protocol — multi-sig treasury, DAO governance, token vault, and access control.

## Tech Stack

- **Language**: Rust
- **SDK**: `soroban-sdk`
- **Build**: Cargo

## Quick Start

```bash
cargo build --all
cargo test --all
```

## Deploy

```bash
stellar contract build
stellar contract deploy --wasm target/wasm32-unknown-unknown/release/*.wasm --source <identity> --network testnet
```

## License

MIT — see [LICENSE](LICENSE).
