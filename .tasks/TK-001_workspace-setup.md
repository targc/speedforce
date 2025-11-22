# Task: Set Up Rust Workspace Structure

**Status**: pending
**Dependencies**: None
**Estimated Effort**: small

## Objective

Create the Rust workspace structure with three crates (tunnel-protocol, tunnel-server, tunnel-client) and configure workspace-level dependencies.

## Context

The speedforce project follows a workspace architecture with shared dependencies. We need to establish the root workspace Cargo.toml and create directory structures for three separate crates that will share common dependencies like tokio, serde, and tracing. This follows the flat structure philosophy outlined in CLAUDE.md.

## Files to Modify/Create

- `/Users/tar/Documents/alpha/speedforce/Cargo.toml` - Update to workspace manifest
- `/Users/tar/Documents/alpha/speedforce/tunnel-protocol/Cargo.toml` - Create protocol crate manifest
- `/Users/tar/Documents/alpha/speedforce/tunnel-server/Cargo.toml` - Create server crate manifest
- `/Users/tar/Documents/alpha/speedforce/tunnel-client/Cargo.toml` - Create client crate manifest
- `/Users/tar/Documents/alpha/speedforce/tunnel-protocol/src/` - Create directory
- `/Users/tar/Documents/alpha/speedforce/tunnel-server/src/` - Create directory
- `/Users/tar/Documents/alpha/speedforce/tunnel-client/src/` - Create directory

## Detailed Steps

1. Replace root `/Users/tar/Documents/alpha/speedforce/Cargo.toml` with workspace configuration:
   - Add `[workspace]` section with members: `["tunnel-server", "tunnel-client", "tunnel-protocol"]`
   - Set `resolver = "2"`
   - Add `[workspace.dependencies]` section with:
     - `tokio = { version = "1.35", features = ["full"] }`
     - `serde = { version = "1.0", features = ["derive"] }`
     - `serde_json = "1.0"`
     - `base64 = "0.21"`
     - `tracing = "0.1"`
     - `tracing-subscriber = "0.3"`

2. Create directory structure:
   - `mkdir -p tunnel-protocol/src`
   - `mkdir -p tunnel-server/src`
   - `mkdir -p tunnel-client/src`

3. Create `tunnel-protocol/Cargo.toml` with:
   - Package name: "tunnel-protocol", version: "0.1.0", edition: "2021"
   - Dependencies: serde, serde_json, base64, tokio (all using workspace = true)

4. Create `tunnel-server/Cargo.toml` with:
   - Package name: "tunnel-server", version: "0.1.0", edition: "2021"
   - Dependencies: tunnel-protocol (path = "../tunnel-protocol"), tokio, tracing, tracing-subscriber (workspace = true)
   - Additional: axum = "0.7", tower = "0.4", hyper = "1.0"

5. Create `tunnel-client/Cargo.toml` with:
   - Package name: "tunnel-client", version: "0.1.0", edition: "2021"
   - Dependencies: tunnel-protocol (path = "../tunnel-protocol"), tokio, tracing, tracing-subscriber (workspace = true)
   - Additional: reqwest = "0.11"

6. Verify workspace structure with `cargo metadata --no-deps` (should list all three crates)

## Acceptance Criteria

- [ ] Root Cargo.toml is a workspace manifest with three members
- [ ] All workspace dependencies are defined with correct versions
- [ ] Three crate directories exist with proper Cargo.toml files
- [ ] Each crate Cargo.toml references workspace dependencies correctly
- [ ] tunnel-server and tunnel-client both depend on tunnel-protocol via path
- [ ] `cargo metadata --no-deps` runs successfully and lists all crates
- [ ] Directory structure matches the flat pattern in CLAUDE.md

## Reference

See CLAUDE.md sections:
- "Project Structure" (lines 30-47)
- "Dependencies" (lines 329-394)
