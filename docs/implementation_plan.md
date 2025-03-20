# Crate Restructuring: Technical Implementation Plan

## Current Status

We have made significant progress in restructuring the Squirrel codebase by moving code from the monolithic `core` crate into dedicated, purpose-specific crates. We have resolved many compilation errors related to import issues, error handling, and dependency management.

## Completed Tasks

### 1. Fixed Security Module

- Fixed the `create_role` and `create_role_with_id` methods in `SecurityManagerImpl` to:
  - Accept `HashSet<Permission>` instead of string permissions
  - Properly convert between string permissions and `Permission` objects
  - Implement proper error handling with `SquirrelError::Security`

- Improved error handling in `assign_role` and `assign_role_by_name` methods:
  - Fixed borrow checker issues in `assign_role_by_name`
  - Ensured consistent error message formats
  - Properly released read locks before performing operations

- Enhanced `authenticate` and `authorize` methods:
  - Added clear error messages for authentication failures
  - Improved security level validation
  - Added proper session validation checks

### 2. Fixed Network Module

- Fixed the `NetworkError` to `SquirrelError` conversion in the monitoring crate
- Ensured proper error propagation from network components

### 3. MCP Module Improvements

- Updated `MCPError` enum to include missing variants
- Fixed error formatting in `Display` implementation
- Resolved conflicts between `thiserror::Error` derive and manual implementations

### 4. Import Path Fixes

- Updated imports in monitoring modules
- Fixed metric collection classes
- Correctly referenced core error types

### 5. MCP Sync Module Update

- Fixed `Mutex` import 
- Updated chrono duration handling with proper imports
- Fixed type conflicts in the sync module

## Remaining Tasks

### 1. Fix Remaining Protocol Adapter Issues

- Update protocol state handling
- Fix unsafe code in `impl_protocol.rs`
- Update message routing

### 2. Command Module Integration

- Create the missing registry and factory modules
- Update imports to use `squirrel_core::error` types

### 3. Context Crate

- Verify imports and fix any references to old module structures
- Ensure proper initialization sequence

### 4. Fix RwLock Issues in Monitoring Metrics

The metrics module has issues with the `RwLockReadGuard<'_, Vec<Metric>>` type:

1. Need to complete `Metric` struct Clone implementation
2. Update metrics collectors to properly handle `RwLockWriteGuard` types
3. Implement proper methods for metrics collection that don't require cloning

## Per-Crate Action Items

### Core Crate

✅ **Completed**:
- Minimal core crate with only error types and build information
- Removed app module and other functionality

**Remaining Tasks**:
- None - Core crate is complete

### Context Crate

**Remaining Tasks**:
- Fix import references to use proper paths
- Verify all cross-crate imports are correct

### Monitoring Crate

✅ **Completed**:
- Fixed `NetworkError` conversion
- Updated import references

**Remaining Tasks**:
- Fix metrics collection code to handle RwLock properly
- Complete error handling in metrics adapters

### MCP Crate

✅ **Completed**:
- Fixed security module error handling
- Updated `MCPError` enum and error handling
- Fixed `SecurityManagerImpl` methods

**Remaining Tasks**:
- Fix protocol adapter code
- Complete the sync module

### Commands Crate

**Remaining Tasks**:
- Implement missing modules (`registry.rs` and `factory.rs`)
- Fix all import references

### App Crate

**Remaining Tasks**:
- Verify imports are correct
- Ensure app initialization works with the new structure

## Testing Steps

After the code changes are made, we need to:

1. Run `cargo build` to ensure everything compiles
2. Run `cargo test` to verify that all tests pass
3. Manually test key CLI features to ensure they work
4. Verify that the web interface still works

## Timeline

1. **Phase 1** - Fix immediate build errors (1-2 days) ✅ **Partially Complete**
2. **Phase 2** - Complete the module migration (2-3 days)
3. **Phase 3** - Testing and verification (1-2 days)
4. **Phase 4** - Documentation and cleanup (1 day)

## Resources

- [Rust Module System Documentation](https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html)
- [Cargo Workspace Documentation](https://doc.rust-lang.org/cargo/reference/workspaces.html) 