# Architecture

PayLink is a non-custodial payment-link protocol on Stellar. Contracts are the authoritative source of invoice state and asset custody.

## Repositories

- `paylink-contracts` — Soroban invoice and escrow contracts (this repo)
- `paylink-api` — NestJS API, event indexer, webhook delivery
- `paylink-web` — Next.js merchant dashboard and public checkout
- `paylink-sdk` — TypeScript SDK, payment button, webhook verification
- `paylink-docs` — Documentation and guides
- `paylink-website` — Marketing site

## Contract Architecture

The protocol operates through a single Soroban contract, `paylink-invoice`:

### Storage Layout

- **Instance storage** — admin address, next invoice ID counter
- **Persistent storage** — individual invoice entries (`Invoice(invoice_id)`), allowed asset entries (`AllowedAsset(asset)`)

### Invoice Lifecycle

```
Created ──pay──> Paid ──fulfill──> Fulfilled
  │                │
  ├─cancel──> Canceled
  │                │
  │                └─refund──> Refunded (escrow only)
  │
  └─expire──> Expired
```

### Payment Modes

**Direct payment**: Funds transfer from payer to recipient in the same transaction. Cannot be contract-refunded. Fulfillment is a soft state marker.

**Escrow payment**: Funds transfer from payer to the contract. Released to recipient on fulfillment. Refundable to payer by the merchant (before expiry) or by the payer (after expiry).

### Authorization

Every entry point requires explicit authorization:
- Admin: `set_asset_enabled`
- Merchant: `create_invoice`, `cancel_invoice`, `mark_fulfilled`, refund before expiry
- Payer: `pay_invoice`, refund after expiry
- Permissionless: `expire_invoice`, `get_invoice`
