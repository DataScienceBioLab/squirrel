# Squirrel Codebase Architecture

## Architecture Overview

After the restructuring, the Squirrel codebase follows a modular, multi-crate architecture that separates concerns and provides clear boundaries between components. This document outlines the architectural design, crate responsibilities, and interaction patterns.

## Architectural Principles

1. **Separation of Concerns**: Each crate has a well-defined responsibility.
2. **Minimal Dependencies**: Crates only depend on what they need.
3. **Core Stability**: The core crate provides stable, essential functionality.
4. **Proper Abstraction**: Interfaces and adapters facilitate testing and flexibility.
5. **Consistent Error Handling**: Unified error handling across all crates.

## Crate Architecture

```
┌────────────────────────────┐
│         squirrel-cli       │
└───────────┬────────────────┘
            │
┌───────────▼────────────────┐
│         squirrel-app       │
└───┬───────┬────────┬───────┘
    │       │        │
┌───▼───┐ ┌─▼──┐ ┌───▼────┐
│ core  │ │mcp │ │context │
└───────┘ └─┬──┘ └────────┘
            │
      ┌─────▼─────┐
      │monitoring │
      └───────────┘
```

## Key Components

### Core Crate

The `squirrel-core` crate provides fundamental types and error handling used throughout the system. It's intentionally minimal to prevent circular dependencies.

**Key Components:**
- `SquirrelError`: A comprehensive error type used across all crates
- Build information for versioning

### MCP Crate (Machine Context Protocol)

The `squirrel-mcp` crate implements the Machine Context Protocol, providing secure communication between components.

**Key Components:**
- `SecurityManager`: Handles authentication, authorization, and encryption
- `Protocol`: Defines message formats and handlers
- `Sync`: Manages state synchronization between components

### Security Module Improvements

We've significantly enhanced the security module with the following improvements:

1. **Role-Based Access Control (RBAC)**:
   - Implemented proper `Permission` handling in `create_role` methods
   - Enhanced role assignment with better error handling
   - Improved session management and token validation

2. **Authentication Flow**:
   ```
   ┌─────────┐     ┌─────────────────┐     ┌─────────┐     ┌─────────┐
   │ Client  │────►│ Authentication  │────►│ Session │────►│ Token   │
   └─────────┘     └─────────────────┘     └─────────┘     └─────────┘
                           │                     │
                           ▼                     ▼
                   ┌─────────────┐        ┌─────────────┐
                   │ Credentials │        │ Permissions │
                   │ Validation  │        │ Management  │
                   └─────────────┘        └─────────────┘
   ```

3. **Authorization Process**:
   - Enhanced token validation with proper expiration checks
   - Improved security level verification
   - Added fine-grained permission checking

4. **Error Handling**:
   - Consistent error types and messages
   - Proper propagation of security errors
   - Enhanced debugging information

### Error Handling Framework

The error handling system has been improved across all crates:

1. **Error Type Hierarchy**:
   ```
   SquirrelError
   ├── Security
   ├── Protocol
   ├── Context
   ├── Monitoring
   ├── Storage
   ├── Network
   └── General
   ```

2. **Error Conversion**:
   - Implemented proper `From` traits for error conversion
   - Ensured consistent error formatting
   - Added context information to errors for better debugging

3. **Error Recovery**:
   - Added mechanisms for recoverable vs. non-recoverable errors
   - Implemented retry logic for transient failures
   - Added severity levels for error handling

## Component Interactions

### Security Flow

The security module now follows a clear flow for authentication and authorization:

1. **Authentication**:
   - Client provides credentials
   - System validates credentials against stored data
   - System checks for too many failed attempts
   - System generates a session and token upon success

2. **Authorization**:
   - Client provides token with request
   - System validates token and checks expiration
   - System verifies security level is sufficient
   - System checks specific permissions if required
   - System grants or denies access based on results

3. **Session Management**:
   - Sessions track active users and their permissions
   - Sessions expire after a configurable time
   - Sessions store encryption keys for secure communication

## Data Flow

The data flow between components has been optimized:

```
Client Request
    ↓
Protocol Adapter
    ↓
Security Check
    ↓
Command Execution
    ↓
Response Generation
    ↓
Client Response
```

## Future Improvements

While we've made significant progress, some areas still need attention:

1. **Protocol Adapter**: Complete the implementation of protocol state handling
2. **Sync Module**: Finish the synchronization mechanisms
3. **Metrics Collection**: Fix RwLock issues in the monitoring module
4. **Command Registration**: Implement the command registry and factory

## Development Standards

To maintain code quality as the project evolves:

1. **Error Handling**: All public functions should return appropriate `Result` types
2. **Documentation**: All public APIs should be documented with examples
3. **Testing**: Unit tests for all components, integration tests for interactions
4. **Dependency Injection**: Use DI patterns instead of global state
5. **Async/Await**: Use proper async patterns throughout the codebase

## Cross-Cutting Concerns

### Error Handling

All crates use the common error types from `squirrel-core`:
- `SquirrelError` enum for all error variants
- `Result<T>` type alias for consistent return types
- Error context and metadata support

### Logging and Tracing

Consistent logging approach across all crates:
- Structured logging via the `tracing` crate
- Common log levels and formatting
- Contextual information in log entries

### Configuration

Hierarchical configuration approach:
- Core configuration from `squirrel-context`
- Crate-specific configuration types
- Environment-specific overrides

## Communication Patterns

### Direct Dependencies

Components depend directly on each other when appropriate:
```rust
// Example of a direct dependency
use squirrel_core::error::Result;
```

### Adapter Pattern

For more complex interactions, adapters provide abstraction:
```rust
// Example of adapter usage
let health_checker = create_checker_adapter();
health_checker.check_health().await?;
```

### Trait-Based Abstractions

Traits define interfaces between components:
```rust
// Example of a trait interface
#[async_trait]
pub trait HealthChecker {
    async fn check_health(&self) -> Result<HealthStatus>;
    // Other methods...
}
```

## Testing Strategy

1. **Unit Tests**: Each crate includes unit tests for its components.
2. **Integration Tests**: Tests that verify interactions between crates.
3. **Mocking**: Test utilities provide mock implementations of key traits.
4. **End-to-End Tests**: CLI and web interfaces have comprehensive tests.

## Future Architecture Considerations

1. **Plugin System**: Consider a plugin architecture for extensibility.
2. **Stronger Boundaries**: Further isolate components with clearer interfaces.
3. **Performance Optimization**: Identify and optimize critical paths.
4. **Distributed Architecture**: Support for distributed deployment scenarios.
5. **Service Mesh Integration**: Integration with service mesh technologies.

## Conclusion

The restructured architecture provides a solid foundation for future development. By splitting functionality into purpose-specific crates, we've improved maintainability, testability, and the ability to evolve the system over time. 