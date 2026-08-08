# paylink-contracts

Soroban invoice and escrow contracts for PayLink Stellar.

## Contract

- **Crate**: `paylink-invoice`
- **Language**: Rust, `soroban-sdk`
- **Target**: `wasm32v1-none`

## Interface

### State-Changing Methods
| Method | Auth | Description |
|--------|------|-------------|
| `initialize(admin)` | deployer | One-time admin setup |
| `create_invoice(merchant, recipient, asset, amount, expiry, escrowed)` | merchant | Create invoice, returns ID |
| `pay_invoice(payer, invoice_id)` | payer | Pay invoice |
| `mark_fulfilled(merchant, invoice_id)` | merchant | Fulfill and release escrow |
| `refund_invoice(caller, invoice_id)` | merchant or payer | Refund escrowed payment |
| `cancel_invoice(merchant, invoice_id)` | merchant | Cancel before payment |
| `expire_invoice(invoice_id)` | none | Materialize expiry |
| `set_asset_enabled(asset, enabled)` | admin | Manage asset allowlist |

### Read Methods
| Method | Description |
|--------|-------------|
| `get_invoice(invoice_id)` | Full invoice state |
| `is_asset_enabled(asset)` | Asset allowlist check |
| `get_next_invoice_id()` | Counter read |

## Build

```bash
cargo build --release --target wasm32v1-none
cargo test --all
```

## Deploy

```bash
stellar contract deploy \
  --wasm target/wasm32v1-none/release/paylink_invoice.wasm \
  --source-account alice \
  --network testnet \
  -- \
  --admin <ADMIN_PUBLIC_KEY>
```
