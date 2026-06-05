# Style Guide — StellarGuard

This document defines the coding standards for all contributions to the StellarGuard project.

---

## 🦀 Rust (Soroban Smart Contracts)

### Formatting

- Use `rustfmt` for all formatting — run `cargo fmt` before every commit.
- Use `clippy` for linting — run `cargo clippy` and resolve all warnings.

### Naming Conventions

| Item | Convention | Example |
|------|-----------|---------|
| Functions | `snake_case` | `create_proposal` |
| Types/Structs | `PascalCase` | `TreasuryConfig` |
| Constants | `SCREAMING_SNAKE_CASE` | `MAX_SIGNERS` |
| Enums | `PascalCase` variants | `ProposalStatus::Active` |
| Modules | `snake_case` | `mod treasury_logic;` |

### Contract Functions

```rust
// ✅ Good: descriptive, uses require_auth, returns Result
pub fn deposit(env: Env, from: Address, amount: i128) -> Result<(), Error> {
    from.require_auth();
    // logic...
    Ok(())
}

// ❌ Bad: no auth check, unclear naming, panics
pub fn d(env: Env, a: Address, n: i128) {
    // logic that panics on error
}
```

### Error Handling

- Define all errors in a `#[contracterror]` enum.
- Never use `.unwrap()` in contract code — always propagate errors.
- Use descriptive error variant names.

```rust
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    InsufficientFunds = 4,
    InvalidThreshold = 5,
    ProposalNotFound = 6,
    AlreadyVoted = 7,
    VotingClosed = 8,
}
```

### Storage

- Use `DataKey` enum for all storage keys.
- Prefer `Persistent` storage for user/state data.
- Use `Instance` storage for contract-wide config (admin, threshold).
- Set appropriate TTL values for persistent entries.

---

## 📘 TypeScript (Frontend)

### Formatting

- Use **Prettier** for formatting.
- Use **ESLint** with the Next.js recommended config.

### Naming Conventions

| Item | Convention | Example |
|------|-----------|---------|
| Components | `PascalCase` | `TreasuryCard.tsx` |
| Hooks | `camelCase` with `use` prefix | `useTreasury.ts` |
| Utilities | `camelCase` | `formatAddress.ts` |
| Constants | `SCREAMING_SNAKE_CASE` | `NETWORK_PASSPHRASE` |
| Types/Interfaces | `PascalCase` | `ProposalData` |

### Component Structure

```tsx
// ✅ Good: typed props, descriptive names
interface ProposalCardProps {
  proposal: ProposalData;
  onVote: (id: number, vote: boolean) => void;
}

export function ProposalCard({ proposal, onVote }: ProposalCardProps) {
  return (
    <div className="proposal-card">
      <h3>{proposal.title}</h3>
      <button onClick={() => onVote(proposal.id, true)}>
        Approve
      </button>
    </div>
  );
}
```

### File Organization

- One component per file.
- Co-locate styles with components when possible.
- Keep hooks in `hooks/` directory.
- Keep Soroban interaction helpers in `lib/`.

---

## 🏗️ TypeScript (NestJS Backend)

### Formatting

- Use **Prettier** for formatting (same config as frontend).
- Use **ESLint** with `@typescript-eslint` rules.

### Naming Conventions

| Item | Convention | Example |
|------|-----------|---------|
| Controllers | `PascalCase` + `Controller` suffix | `TreasuryController` |
| Services | `PascalCase` + `Service` suffix | `TreasuryService` |
| Modules | `PascalCase` + `Module` suffix | `AppModule` |
| DTOs / schemas | `PascalCase` | `CreateProposalDto` |
| Guards / Middleware | `PascalCase` + role suffix | `ApiKeyGuard`, `RequestLoggerMiddleware` |

### File Organization

- One class per file. File name mirrors the class name in `kebab-case`:
  - `TreasuryController` → `treasury.controller.ts`
  - `TreasuryService` → `treasury.service.ts`
- Group files by feature module under `src/<feature>/`:
  ```
  src/
    treasury/
      treasury.controller.ts
      treasury.service.ts
      treasury.service.test.ts
    governance/
    vault/
    middleware/
    guards/
  ```
- Shared infrastructure (middleware, guards, decorators) lives in its own top-level directory.

### Decorator Usage

```typescript
// ✅ Controllers: declare route prefix and apply Swagger tags
@ApiTags('treasury')
@Controller('treasury')
export class TreasuryController {

  // ✅ Route methods: declare HTTP verb, path, and Swagger metadata
  @ApiOperation({ summary: 'Get treasury balance' })
  @ApiResponse({ status: 200, description: 'Current balance' })
  @Get('balance')
  getBalance() { ... }

  // ✅ Write endpoints protected by guard — use @Public() to opt out
  @Post('propose')
  propose(@Body() dto: ProposeDto) { ... }
}
```

- Always annotate controllers and routes with Swagger decorators (`@ApiTags`, `@ApiOperation`, `@ApiResponse`).
- Use `@Public()` to mark endpoints that should bypass the global `ApiKeyGuard`.
- Inject services via constructor, not property injection.

### Zod Validation

- Define all input schemas with Zod in the service or a dedicated `*.schema.ts` file.
- Parse at the service boundary, not in the controller.

```typescript
import { z } from 'zod';

const ProposalSchema = z.object({
  contractId: z.string().min(1),
  limit:      z.number().int().positive().max(100).optional(),
});

// In the service:
const params = ProposalSchema.parse(raw);
```

- Never trust raw query/body values without parsing them through a Zod schema.
- Use `z.coerce.number()` for numeric query params (they arrive as strings in Express).

### Database Query Patterns

- Interact with Postgres through the `Pool` from `src/db.ts` — never create a new `Pool` directly inside a service.
- Prefer parameterised queries; never interpolate user input into SQL strings.

```typescript
// ✅ Good: parameterised
const result = await pool.query(
  'SELECT * FROM events WHERE contract_id = $1 ORDER BY id DESC LIMIT $2',
  [contractId, limit],
);

// ❌ Bad: SQL injection risk
const result = await pool.query(
  `SELECT * FROM events WHERE contract_id = '${contractId}'`,
);
```

- Wrap multi-statement operations in a transaction (`BEGIN` / `COMMIT` / `ROLLBACK`).
- Keep query logic in the service layer; controllers only validate, delegate, and format responses.

---

## 📝 General Rules

1. **No commented-out code** in commits.
2. **No `console.log`** in production code (use proper logging).
3. **Write tests** for all new functions.
4. **Document public APIs** with JSDoc (TS) or `///` doc comments (Rust).
5. **Keep functions small** — aim for single responsibility.
