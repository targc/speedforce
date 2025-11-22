# Task: Fix Any Compilation Errors and Run Full Build

**Status**: pending
**Dependencies**: TK-010, TK-014
**Estimated Effort**: small

## Objective

Ensure the entire workspace compiles cleanly without errors or warnings, and all binaries can be built successfully.

## Context

After implementing all the core functionality, we need to verify that everything compiles together. This includes fixing any import issues, type mismatches, or API usage errors that might have been introduced. We'll also enable stricter warnings to catch potential issues early.

## Files to Modify/Create

- Any files with compilation errors (TBD based on build output)

## Detailed Steps

1. Run clean build of entire workspace:
   - `cd /Users/tar/Documents/alpha/speedforce`
   - `cargo clean`
   - `cargo build --workspace`
   - Capture any compilation errors

2. Fix any errors in tunnel-protocol:
   - `cargo check -p tunnel-protocol`
   - Address any missing imports, type errors, or trait bound issues
   - Ensure all public API is correctly exposed

3. Fix any errors in tunnel-server:
   - `cargo check -p tunnel-server`
   - Fix any axum-related errors (handler signatures, extractors, etc.)
   - Ensure TunnelConnection lifetime issues are resolved
   - Fix any RwLock/Arc usage errors

4. Fix any errors in tunnel-client:
   - `cargo check -p tunnel-client`
   - Fix any reqwest-related errors
   - Ensure error handling conversions are correct

5. Address all compiler warnings:
   - Run `cargo build --workspace` and review warnings
   - Fix unused imports, unused variables, dead code
   - Add `#[allow(dead_code)]` only where absolutely necessary

6. Test that binaries can be run:
   - `cargo build --bin tunnel-server`
   - `cargo build --bin tunnel-client`
   - Verify executables are created in `target/debug/`

7. Run in release mode to check for optimization issues:
   - `cargo build --release --workspace`
   - Verify no new errors appear in release mode

8. Document any significant fixes or workarounds

## Acceptance Criteria

- [ ] `cargo build --workspace` completes without errors
- [ ] No compilation errors in tunnel-protocol
- [ ] No compilation errors in tunnel-server
- [ ] No compilation errors in tunnel-client
- [ ] All compiler warnings are addressed or documented
- [ ] Both binaries can be built successfully
- [ ] Release build works without errors
- [ ] All dependencies are correctly specified in Cargo.toml files

## Reference

See CLAUDE.md sections:
- "Dependencies" (lines 329-394)
- All implementation details sections for correct API usage
