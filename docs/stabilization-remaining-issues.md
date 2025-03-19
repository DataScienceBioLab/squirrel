# Remaining Stabilization Issues

## Progress So Far

We've made significant progress on the stabilization of the codebase by addressing several critical issues:

1. **Fixed ContextError and Error Type Implementations**:
   - Updated `ContextError` variants to include string parameters
   - Fixed `From<ContextError> for SquirrelError` implementation
   - Resolved `From<MCPError> for SquirrelError` conflict with existing implementation

2. **Fixed Type Mismatches**:
   - Corrected `RecoveryStrategy` implementation to handle `VecDeque<ContextSnapshot>` conversion to slice
   - Added proper error string messages for various error enum variants

3. **Fixed `Debug` Trait Implementations**:
   - Added `Debug` bounds to necessary trait definitions
   - Verified implementations for `FilePersistence` and other types 

## Remaining Issues

Based on our latest `cargo check`, the following issues still need to be addressed:

### 1. Sysinfo API Mismatches

The most significant category of errors involves missing or renamed methods from the sysinfo crate.
The compiler reports numerous errors like:

```
error[E0599]: no method named `networks` found for reference `&sysinfo::System` in the current scope
error[E0599]: no method named `disks` found for reference `&sysinfo::System` in the current scope
error[E0599]: no method named `refresh_networks` found for struct `sysinfo::System` in the current scope
```

The following methods need to be updated:
- `refresh_networks()` → `refresh_memory()` (or new equivalent)
- `networks()` method (missing implementation)
- `disks()` method (missing implementation)
- `thread_count()` → `thread_kind()` (or new equivalent)

This requires updating these calls throughout the codebase, especially in:
- `monitoring/network/mod.rs`
- `monitoring/network/system_info.rs`
- `monitoring/metrics/resource.rs`

### 2. Struct Field Mismatches

Several struct field mismatches need to be fixed:

```
error[E0560]: struct `NetworkStats` has no field named `interfaces`
error[E0560]: struct `NetworkStats` has no field named `timestamp`
error[E0560]: struct `NetworkStats` has no field named `total_rx_bytes`
error[E0560]: struct `NetworkStats` has no field named `total_tx_bytes`
```

These issues mainly appear in `monitoring/network/mod.rs` and require aligning the struct initialization with the actual struct definition.

### 3. Method Signature Mismatches

Several methods are called with incorrect arguments:

```
error[E0308]: mismatched types
   --> crates\core\src\monitoring\alerts\adapter.rs:164:38
    |
164 | ...   self.inner.send_notification(alert).await.map_err(|e| SquirrelError::Alert(format!("Failed t... 
    |                  ----------------- ^^^^^ expected `&AlertNotification`, found `Alert`
```

- `send_notification` expects `&AlertNotification`, but receives `Alert`
- `record_operation_duration` might need to be renamed to `record_operation` 

### 4. Type Mismatches in If/Else Branches

Several if/else branches return incompatible types:

```
error[E0308]: `if` and `else` have incompatible types
   --> crates\core\src\monitoring\mod.rs:516:13
    |
    | expected `Arc<AlertManagerAdapter>`, found `Arc<DefaultAlertManager>`
```

These issues require either:
- Creating adapters for the incompatible types
- Modifying the factory methods to return consistent types
- Changing the type constraints in affected functions

### 5. Function Call Issues in MCPSync

The `MCPSync::new` function is called incorrectly:

```
error[E0061]: this function takes 4 arguments but 1 argument was supplied
  --> crates\core\src\mcp\context_manager.rs:61:26
   |
61 |         let sync = match MCPSync::new(sync_config).await {
```

This needs to be fixed by providing all required arguments or modifying the function signature.

### 6. Unused Variables and Imports

There are numerous unused variables and imports throughout the codebase that should be cleaned up:

```
error: unused variable: `key`
error: unused import: `AsyncReadExt`
```

### 7. Summation Error in Resource Metrics

There's a type conversion issue when summing values:

```
error[E0277]: a value of type `f64` cannot be made by summing an iterator over elements of type `u64`
```

This requires fixing the type conversion in the affected code.

## Recommended Approach

1. **First Priority**: Fix the sysinfo API mismatches
   - Update all calls to `refresh_networks`, `networks`, and `disks` methods
   - Implement proper wrappers in `system_info.rs` if needed

2. **Second Priority**: Address struct field mismatches
   - Update `NetworkStats` struct initialization to match its definition
   - Update the implementation to return correct fields

3. **Third Priority**: Fix method signature mismatches
   - Align the argument types for `send_notification` and similar methods
   - Update return type handling for async functions

4. **Fourth Priority**: Fix if/else branch compatibility
   - Create type adapters or modify factory methods for consistent types

5. **Fifth Priority**: Fix the `MCPSync` function call
   - Provide all required arguments or update the function signature

6. **Sixth Priority**: Clean up unused variables and imports
   - This can be left for last as it doesn't affect functionality

## Specific File Recommendations

Based on the error messages, these files need the most attention:

1. `monitoring/network/mod.rs` - Numerous sysinfo API and struct field issues
2. `monitoring/network/system_info.rs` - Many sysinfo API mismatches
3. `monitoring/metrics/resource.rs` - Several sysinfo API and type conversion issues
4. `mcp/context_manager.rs` - Issues with MCPSync function call and mismatched types
5. `monitoring/alerts/adapter.rs` - Method signature mismatches
6. `monitoring/mod.rs` - Type mismatches in if/else branches 