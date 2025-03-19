# Next Steps for Codebase Stabilization

## Overview

After implementing several fixes, the codebase still has 134 errors. This document outlines specific next steps to systematically address the remaining issues.

## High Priority Issues

### 1. RwLock Type Consistency

Several errors involve mixing `std::sync::RwLock` and `tokio::sync::RwLock`:

```rust
error[E0277]: `Result<RwLockReadGuard<'_, NotificationConfig>, ...>` is not a future
error[E0277]: `Result<RwLockWriteGuard<'_, HashMap<..., ...>>, ...>` is not a future
```

**Actions:**
- In `monitoring/alerts/notify.rs`: Replace all usages of `std::sync::RwLock` with `tokio::sync::RwLock`, or remove `.await` calls for std::sync locks
- In `monitoring/network/mod.rs`: Standardize on one RwLock implementation
- Create helper functions to handle lock acquisition consistently

### 2. sysinfo API Mismatches

The code calls methods that don't exist in the sysinfo crate:

```rust
error[E0599]: no method named `disks` found for reference `&sysinfo::System` in the current scope
error[E0599]: no method named `networks` found for reference `&sysinfo::System` in the current scope
error[E0599]: no method named `refresh_networks_list` found for struct `sysinfo::System` in the current scope
```

**Actions:**
- Check the latest sysinfo documentation for proper method names
- Create wrapper methods for missing functionality, using what's available in sysinfo
- Update `monitoring/metrics/resource.rs` and `monitoring/network/system_info.rs` files

### 3. Error Type Conversions

Multiple errors where one error type can't be converted to another:

```rust
error[E0308]: mismatched types (expected `SquirrelError`, found `Result<(), ...>`)
error[E0277]: `?` couldn't convert the error to `error::SquirrelError`
```

**Actions:**
- Implement `From` trait for `NotificationError` to `SquirrelError`
- Update error enums to consistently use the same error types
- Fix incorrect error types in `mcp/mod.rs` where `AppInitializationError` is used instead of `MCPError`

### 4. Missing Debug Implementations

Trait objects are used in structs with `#[derive(Debug)]` but don't implement `Debug`:

```rust
error[E0277]: `(dyn context::ContextSubscriber + 'static)` doesn't implement `std::fmt::Debug`
error[E0277]: `(dyn mcp::persistence::Persistence + 'static)` doesn't implement `std::fmt::Debug`
```

**Actions:**
- Add `Debug` bounds to trait definitions
- Use manual `Debug` implementations instead of derive for structs with trait objects
- Create wrapper types with custom Debug implementations

## Medium Priority Issues

### 1. Incorrect Method Calls

Several errors involve calling methods that don't exist or have different names:

```rust
error[E0599]: no method named `whole_seconds` found for struct `time::OffsetDateTime`
error[E0599]: no method named `create_checker` found for struct `monitoring::health::HealthCheckerFactory`
```

**Actions:**
- In `monitoring/alerts/notify.rs`: Update usage of `whole_seconds()` to use a method that exists in `OffsetDateTime` 
- In `monitoring/health/mod.rs`: Fix references to non-existent methods in `HealthCheckerFactory`

### 2. Struct Field Errors

Missing or incorrect field accesses:

```rust
error[E0560]: struct `monitoring::metrics::resource::TeamResourceMetrics` has no field named `team_id`
error[E0609]: no field `id` on type `std::string::String`
```

**Actions:**
- Update struct initialization to use correct field names
- Update field accesses to use correct field names
- Check struct definitions to ensure they have the required fields

### 3. Expected Functions, Found Values

Several errors in the context module:

```rust
error[E0618]: expected function, found `context::ContextError`
```

**Actions:**
- Update error enum in `context/mod.rs` to use tuple variant instead of unit variant
- Fix all usages in the codebase that treat unit variants as functions

## Low Priority Issues

### 1. Unused Variables and Imports

Several unused imports and variables warnings:

```rust
error: unused variable: `key`
error: unused import: `AsyncReadExt`
```

**Actions:**
- Prefix unused variables with underscore
- Remove unused imports

### 2. Incompatible Types in If/Else Blocks

Returning different types from if/else branches:

```rust
error[E0308]: `if` and `else` have incompatible types
```

**Actions:**
- In `monitoring/mod.rs`: Ensure both branches return the same type
- Convert types to a common type before returning
- Use trait objects where appropriate

## Implementation Strategy

1. **Create isolated changes** for each category of error
2. **Test incrementally** after each set of related changes
3. **Document patterns** as they are established for future reference
4. **Update the standard procedures** document with lessons learned

## Files Requiring Most Attention

1. `monitoring/network/system_info.rs`
2. `monitoring/metrics/resource.rs` 
3. `monitoring/alerts/notify.rs`
4. `mcp/persistence/mod.rs`
5. `context/sync.rs`

## Timeline

Estimate for addressing all remaining issues: 3-5 days of focused effort, with the following breakdown:

1. RwLock Consistency Issues: 1 day
2. sysinfo API Mismatches: 1 day
3. Error Type Conversions: 1 day
4. Other Issues: 1-2 days

## Conclusion

While there are still numerous errors to resolve, we have a clear path forward. The stabilization work should continue with a focus on RwLock consistency and sysinfo API mismatches, as these represent the majority of the errors and will have the largest impact on codebase stability. 