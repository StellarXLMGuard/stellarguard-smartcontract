# Security

PayLink is non-custodial. The contract handles asset transfers during payment, escrow, fulfillment, and refund. The API, SDK, and web app never hold private keys.

## Contract Security

### Authorization
- Every privileged path calls `require_auth()` on the correct stored authority
- Merchant is loaded from invoice storage, not trusted from caller arguments
- Admin is loaded from instance storage
- `__constructor` is one-time only

### State Machine
- Invoices cannot be paid twice
- Expired invoices cannot be paid
- Paid invoices cannot be canceled
- Only escrowed invoices can be refunded
- Non-escrow refunds are impossible by design (funds never held by contract)

### Asset Safety
- All tokens must be allowlisted by admin
- Amount is validated positive at creation
- Receiving address trustline may need to exist for non-XLM assets

### TTL Management
- Instance and persistent storage TTLs are extended in active write paths
- TTL expiry is not used as a security mechanism (explicit deadlines in data)

## API Security

- API never holds private keys or transaction signing abilities
- Webhook payloads are signed with HMAC-SHA256
- Merchant endpoints require wallet-signature authentication
- Invoice registration verifies on-chain ownership before accepting metadata

## Reporting

Report security vulnerabilities to the repository maintainers. Do not open public issues for security-sensitive findings.
