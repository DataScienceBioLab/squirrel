# Crate Restructuring Documentation

## Overview

This document outlines the restructuring process we've undertaken to modularize the Squirrel codebase by splitting it into dedicated crates. The primary goal was to improve code organization, maintainability, and allow for better dependency management.

## Crate Structure

We have restructured the codebase into the following crates:

| Crate | Purpose |
|-------|---------|
| `squirrel-core` | Shared error types and build information |
| `squirrel-context` | Context management functionality |
| `squirrel-monitoring` | System monitoring and metrics |
| `squirrel-mcp` | Machine Context Protocol implementation |
| `squirrel-commands` | Command handling and registry |
| `squirrel-test-utils` | Testing utilities and helpers |
| `squirrel-app` | Application components and lifecycle |
| `squirrel-context-adapter` | Adapter for context interactions |
| `squirrel-cli` | Command-line interface |
| `squirrel-web` | Web interface and API |

## Core Crate Rationale

### Why We're Keeping `squirrel-core`

The `squirrel-core` crate has been kept as a minimal, foundational crate for several strategic reasons:

1. **Shared Error Types**: The core crate maintains common error types and utilities that are used throughout the ecosystem. This ensures consistent error handling across all crates.

2. **Build Information**: It provides centralized build metadata and version information that can be accessed by all other crates.

3. **Dependency Chain**: Having a minimal core crate avoids circular dependencies and creates a cleaner dependency graph.

4. **API Stability**: The core components are expected to change less frequently, providing a stable foundation for the rest of the ecosystem.

### Core Crate Structure

The minimized core crate now contains:

```
crates/core/
├── src/
│   ├── lib.rs         # Core module definitions and re-exports
│   ├── error.rs       # Shared error types and utilities
│   └── build_info.rs  # Build metadata (included via build script)
├── build.rs           # Build script for compile-time information
└── Cargo.toml         # Core crate manifest
```

## Migration Process

The migration from a monolithic core crate to dedicated crates involved several steps:

1. **Creating New Crates**: We created separate crates for each major component with their own `Cargo.toml` files and dependency specifications.

2. **Moving Code**: We moved code from `crates/core/src/*` to the appropriate crate directories, maintaining the module structure.

3. **Fixing Imports**: We updated import paths throughout the codebase to reference the new crate structure.

4. **Cleaning Up Core**: We removed unnecessary modules from the core crate, keeping only essential functionality.

5. **Adding Re-exports**: We added proper re-exports in each crate's `lib.rs` to maintain a clean public API.

## Import Patterns

During the migration, we had to update several import patterns:

| Old Pattern | New Pattern | Example |
|-------------|-------------|---------|
| `use crate::error::Result` | `use squirrel_core::error::Result` | Error types |
| `use crate::monitoring::metrics` | `use crate::metrics` (in monitoring crate) | Internal crate imports |
| `use crate::monitoring::metrics` | `use squirrel_monitoring::metrics` (in other crates) | Cross-crate imports |
| `use crate::error::SquirrelError` | `use squirrel_core::error::SquirrelError` | Error handling |

## Benefits of the New Structure

1. **Clearer Module Boundaries**: Each crate has a well-defined responsibility.

2. **Improved Compilation Times**: Smaller crates lead to faster incremental compilation.

3. **Better Dependency Management**: Each crate only depends on what it needs.

4. **Parallel Development**: Teams can work on different crates without conflicts.

5. **Easier Testing**: Smaller, focused modules are easier to test in isolation.

6. **Cleaner API Surface**: Each crate exposes only what's necessary.

## Future Considerations

1. **Further Refinement**: Some crates may need additional refinement as the project evolves.

2. **Documentation Updates**: Continuing to update documentation to reflect the new structure.

3. **Integration Testing**: Ensuring the separated components work well together.

4. **API Stability**: Defining a policy for API stability across crate boundaries.

## Appendix: Key Files Modified

- Core crate's `lib.rs`: Updated to reflect its minimal role
- All crates' `Cargo.toml` files: Updated dependencies
- Import statements throughout the codebase
- Binary files moved to their appropriate crates 