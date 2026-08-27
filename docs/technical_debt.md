# Technical Debt

## Security

- Add rate limiting to the login endpoint.
- Use a dummy Argon2 verification path for unknown users to reduce timing-based account enumeration.
- Centralize and explicitly configure Argon2 parameters instead of relying on `Argon2::default()`.

## Sessions

- Add cleanup for expired sessions, either periodically or during authentication.

## Persistence

- Review SQLx error classification so connection, TLS, protocol, and network failures consistently map to repository unavailability.
- Standardize application timestamp precision (PostgreSQL TIMESTAMPTZ uses microsecond precision), ideally as part of the future Clock abstraction.

## Testing

- Add application-service tests for repository failure mappings where currently missing.

## Documentation

- Add Rustdoc documentation for public ports and clearly document their behavior and error contracts.


## Axum

- Axum Path<Uuid> extractor rejects malformed uuids, replace it with custom extractor to handle behavior explicitly.

## Features

- Consider invitation-based household membership with pending/accept/reject states instead of immediate membership assignment.

### Extract reusable authentication/account crate

**When:** After the first aims release.

The current accounts/authentication implementation contains functionality that could be reused across future Rust backend projects. Consider extracting the generic parts into a separate Rust crate and GitHub repository.

Potential candidates for extraction:

- Account/auth domain types:
  - `UserId`
  - `Email`
  - `DisplayName`
  - `PasswordHash`
  - `SessionId`
  - `SessionToken`
  - `Session`
- Repository and infrastructure ports:
  - `UserRepository`
  - `SessionRepository`
  - `PasswordHasher`
  - session token generation/hashing
- Generic application services:
  - user registration
  - login
  - logout/session invalidation
  - session authentication
- Generic implementations such as Argon2 password hashing.

Keep application-specific infrastructure in aims initially:

- SQLx/Postgres repositories and migrations
- Axum routes and handlers
- `CurrentUser` extractor
- cookie configuration
- aims-specific API errors and DTOs
- application bootstrap/wiring

The extracted crate should not depend on aims. aims should depend on the reusable crate.

Possible future structure:

```text
rust-auth/
├── domain/
├── application/
├── ports/
└── adapters/

- [ ] Review household event publishing before v1 release:
  - Reduce duplicated `ShoppingListChanged` publishing across application services.
  - Consider extracting a reusable application-level notification mechanism if additional household events are introduced.
  - Review whether events should only be published when an operation actually changes the shopping-list projection.
  - Review failure semantics when persistence succeeds but event publishing fails.