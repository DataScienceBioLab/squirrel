# Test Suite Modernization Plan

## Overview

This document outlines our approach to modernizing the Squirrel test suite after migrating from a singleton-based architecture to a dependency injection (DI) pattern. The migration has broken many of the existing tests that relied on global state and singleton patterns. We've also made significant improvements to the error handling and security modules that require updated tests.

## Goals

1. Replace/repair all broken tests
2. Establish a consistent pattern for DI-based testing
3. Improve test isolation and reliability
4. Make tests more maintainable
5. Improve test coverage
6. Verify recent security module improvements

## Progress Made

We've made significant progress in fixing compilation errors, particularly in the security and error handling modules:

### 1. Security Module
- Fixed role management functionality
- Improved error handling in authentication and authorization
- Updated permission handling with proper HashSet usage
- Fixed the borrow checker issues in role assignment methods

### 2. Error Framework
- Resolved conflicts between thiserror and manual implementations
- Added proper error variants to MCPError
- Fixed error conversion implementations
- Improved error message consistency

## Issues Identified

After analysis, we've identified several key issues with the current test suite:

### 1. Structural Issues

- ~~Module conflicts (e.g., `persistence.rs` vs `persistence/mod.rs`)~~ ✅ Fixed
- Missing dependencies (e.g., `tempfile` crate) ✅ Added
- Unresolved imports due to code structure changes
- Inconsistent test organization

### 2. DI Specific Issues

- Tests trying to call methods on outdated singletons
- Missing initialization for DI components
- Tests not properly using dependency-injected objects
- Incorrect usage of `Arc<RwLock<T>>` patterns

### 3. Future/Async Issues

- Missing `await` calls in async tests
- Type mismatches with future objects
- Incorrect error handling in async contexts

### 4. Compilation Errors

- Most critical security module errors have been fixed ✅
- Remaining compilation errors in the following modules:
  - Protocol adapter state handling
  - MCP sync functionality
  - Monitoring metrics collection

## Security Testing Requirements

After recent security module improvements, we need to add or update tests for:

1. **Role-Based Access Control (RBAC)**:
   - Test creating roles with HashSet<Permission>
   - Verify permission parsing and validation
   - Test role hierarchy and permission inheritance
   - Test converting between string and Permission objects

2. **Authentication Process**:
   - Test maximum attempts logic
   - Verify token generation and validation
   - Test session creation with proper security levels
   - Test credential validation

3. **Authorization Process**:
   - Test permission validation
   - Verify security level checking
   - Test expired token handling
   - Test session lookup and validation

## Strategy for Error Resolution

1. **Isolate our test code**: First, create a separate test crate or use feature flags to isolate our new tests from the problematic code
2. **Prioritize critical errors**: Identify and fix the most critical errors affecting the MCP module
3. **Coordinate with team**: Document all errors and coordinate with the team responsible for the affected modules
4. **Gradual repair**: Fix errors incrementally, starting with the most fundamental components

## Approach

We've implemented a comprehensive approach to address these issues:

### 1. Test Utilities Module

Created a `test_utils` module that provides:

- Mock implementations of key interfaces
- Test data generators
- Test environment factory
- Helper functions for common testing tasks

### 2. Test Directory Structure

Reorganized tests into a consistent structure:

```
crates/core/src/
├── module_name/
│   ├── tests/
│   │   ├── mod.rs
│   │   └── specific_test_files.rs
```

### 3. Test Implementation Pattern

Using a consistent pattern for all tests:

```rust
#[test]
async fn test_component_function() {
    // ARRANGE: Set up dependencies with explicit DI
    let dependency = Arc::new(RwLock::new(MockDependency::new()));
    let component = ComponentUnderTest::new(dependency.clone());
    
    // ACT: Call the function being tested
    let result = component.function_under_test().await;
    
    // ASSERT: Verify the behavior and state
    assert!(result.is_ok());
    
    // Verify dependency interaction
    let dep_state = dependency.read().await.get_state();
    assert_eq!(dep_state, expected_state);
}
```

### 4. Security Module Tests

The updated security module tests should include:

```rust
#[tokio::test]
async fn test_create_role_with_permissions() {
    // Arrange
    let config = SecurityConfig::default();
    let security_manager = SecurityManagerImpl::new(config).unwrap();
    
    let permissions = [
        Permission { 
            id: "p1".to_string(), 
            name: "read-users".to_string(), 
            resource: "users".to_string(), 
            action: Action::Read 
        },
        Permission { 
            id: "p2".to_string(), 
            name: "create-users".to_string(), 
            resource: "users".to_string(), 
            action: Action::Create 
        }
    ].iter().cloned().collect::<HashSet<_>>();
    
    // Act
    let result = security_manager.create_role(
        "admin".to_string(),
        Some("Administrator".to_string()),
        permissions.clone(),
        HashSet::new()
    ).await;
    
    // Assert
    assert!(result.is_ok());
    let role = result.unwrap();
    assert_eq!(role.name, "admin");
    assert_eq!(role.permissions.len(), 2);
    // Verify the permissions were correctly stored
    assert!(role.permissions.iter().any(|p| p.name == "read-users"));
    assert!(role.permissions.iter().any(|p| p.name == "create-users"));
}
```

### 5. Error Handling Tests

Updated error handling tests should verify proper error conversion:

```rust
#[test]
fn test_network_error_conversion() {
    // Arrange
    let network_error = NetworkError::System("Test error".to_string());
    
    // Act
    let squirrel_error: SquirrelError = network_error.into();
    
    // Assert
    match squirrel_error {
        SquirrelError::Network(msg) => {
            assert!(msg.contains("Test error"));
        }
        _ => panic!("Expected Network error variant"),
    }
}
```

## Implementation Status

| Module | Status | Notes |
|--------|--------|-------|
| test_utils | ✅ Completed | Created mock objects and factory |
| Commands | ✅ Completed | Tests use new DI pattern |
| Context | ✅ Completed | Tests use new async aware pattern |
| MCP Security | ✅ Completed | Tests updated for new permission handling |
| Context Adapter | ✅ Completed | Tests properly use async/await |
| MCP Protocol | ✅ Completed | Comprehensive tests for adapter and DI |
| MCP Transport | ✅ Completed | Complete test suite with DI pattern |
| MCP Sync | 🔄 In Progress | Needs significant refactoring |
| App Module | ❌ Not Started | Scheduled for next phase |
| Error Handling | ✅ Completed | Tests for new error conversion |

## Next Steps

1. Continue implementing tests for remaining modules
2. Fix identified structural issues (module conflicts)
3. Update Cargo.toml dependencies 
4. Implement integration tests between modules
5. Run and validate test suite
6. Document testing patterns for future development

## Best Practices

Moving forward, all tests should follow these principles:

1. **Use Dependency Injection**: No global state or singletons
2. **Proper Async Testing**: Use async/await correctly
3. **Test Isolation**: Each test should be independent
4. **Mock Dependencies**: Use mocks for external dependencies
5. **Clear AAA Pattern**: Arrange, Act, Assert
6. **Descriptive Test Names**: `test_should_do_something_when_condition`
7. **Error Testing**: Test both success and failure cases
8. **Security Testing**: Verify authentication and authorization properly

## Conclusion

By following this plan, we will modernize our test suite to match our new DI-based architecture and verify our recent improvements to the security and error handling modules. This will improve test reliability, maintainability, and help catch issues earlier in the development process. 