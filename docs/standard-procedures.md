# Standard Procedures for Squirrel Development

## Overview

This document outlines the standard procedures and best practices for developing and maintaining the Squirrel codebase. These guidelines ensure consistency, maintainability, and stability across the project.

## Dependency Injection

### Adapter Pattern Implementation

All components should follow the adapter pattern for dependency injection:

1. **Create an adapter struct**:
   ```rust
   pub struct ComponentAdapter {
       inner: Option<Arc<Component>>,
   }
   ```

2. **Implement initialization methods**:
   ```rust
   impl ComponentAdapter {
       pub fn new() -> Self {
           Self { inner: None }
       }
       
       pub fn with_component(component: Arc<Component>) -> Self {
           Self { inner: Some(component) }
       }
       
       pub fn initialize(&mut self) -> Result<()> {
           if self.is_initialized() {
               return Err(ComponentError::AlreadyInitialized);
           }
           
           let component = Component::new();
           self.inner = Some(Arc::new(component));
           Ok(())
       }
       
       pub fn is_initialized(&self) -> bool {
           self.inner.is_some()
       }
   }
   ```

3. **Delegate method calls with proper error handling**:
   ```rust
   impl ComponentAdapter {
       pub fn perform_operation(&self, data: &str) -> Result<String> {
           match &self.inner {
               Some(component) => component.perform_operation(data),
               None => Err(ComponentError::NotInitialized.into())
           }
       }
   }
   ```

4. **Provide factory functions**:
   ```rust
   pub fn create_component_adapter() -> Arc<ComponentAdapter> {
       let mut adapter = ComponentAdapter::new();
       adapter.initialize().expect("Failed to initialize component adapter");
       Arc::new(adapter)
   }
   ```

## Error Handling

### Error Types

1. **Use domain-specific error enums**:
   ```rust
   #[derive(Debug, Error)]
   pub enum ComponentError {
       #[error("Component not initialized")]
       NotInitialized,
       
       #[error("Component already initialized")]
       AlreadyInitialized,
       
       #[error("Invalid operation: {0}")]
       InvalidOperation(String),
   }
   ```

2. **Implement From traits** for error conversion:
   ```rust
   impl From<ComponentError> for SquirrelError {
       fn from(err: ComponentError) -> Self {
           SquirrelError::Component(err.to_string())
       }
   }
   ```

3. **Use the ? operator** for error propagation when possible.

4. **Provide descriptive error messages** that help with troubleshooting.

## Asynchronous Programming

1. **RwLock Consistency**:
   - Use `tokio::sync::RwLock` for async contexts
   - Use `std::sync::RwLock` for synchronous contexts
   - Never mix the two in the same component

2. **Await Usage**:
   - Only use `.await` with tokio async methods
   - Handle std::sync locks with normal blocking operations

3. **Error Handling in Async**:
   - Use `.map_err()` for converting errors in async contexts
   - Return futures that resolve to `Result<T, Error>`

## Testing

1. **Unit Tests**:
   - Test each component in isolation
   - Use dependency injection to provide mock dependencies
   - Follow AAA pattern (Arrange, Act, Assert)

2. **Test Coverage**:
   - Test both success and error cases
   - Test boundary conditions
   - Test thread safety in async contexts

3. **Mock Objects**:
   - Create mock objects for dependencies
   - Verify interactions with mock objects
   - Set up expectations for mock behavior

## Code Style

1. **Documentation**:
   - Document all public API elements
   - Use /// for documentation comments
   - Include examples in documentation
   - Update docs when changing behavior

2. **Naming Conventions**:
   - Use snake_case for variables, functions, and modules
   - Use CamelCase for types
   - Use SCREAMING_SNAKE_CASE for constants
   - Prefer descriptive names over abbreviations

3. **Code Organization**:
   - Group related functionality in modules
   - Keep functions and methods small and focused
   - Use traits to define common behavior
   - Limit public exports to the necessary elements

## Contribution Workflow

1. **Starting Work**:
   - Choose an issue from the issue tracker
   - Assign yourself to the issue
   - Create a feature branch based on main

2. **Development**:
   - Follow the code style guidelines
   - Write tests for new functionality
   - Run `cargo clippy` and fix all warnings
   - Run `cargo fmt` to format your code

3. **Testing**:
   - Run the test suite with `cargo test`
   - Verify your changes in a local environment
   - Check for any performance regressions

4. **Pull Requests**:
   - Create a pull request with a descriptive title
   - Reference the issue number in the PR description
   - Provide a summary of changes
   - Request reviews from team members

5. **Code Review**:
   - Address review comments promptly
   - Re-request review after making changes
   - Ensure tests pass on CI
   - Merge after receiving approval

## RwLock Guidelines

When using RwLock, follow these guidelines:

1. **In synchronous contexts**:
   ```rust
   let guard = lock.read().unwrap(); // or write()
   let value = guard.get_value();
   // Use value
   drop(guard); // Explicitly drop if needed
   ```

2. **In asynchronous contexts**:
   ```rust
   let guard = lock.read().await; // or write()
   let value = guard.get_value();
   // Use value
   drop(guard); // Explicitly drop if needed
   ```

3. **Never mix** `std::sync::RwLock` and `tokio::sync::RwLock` in the same component.

4. **Convert between lock types** when passing objects between sync and async contexts.

## Troubleshooting

When encountering issues, follow these steps:

1. **Check for compiler errors**:
   - Run `cargo check` to see all errors
   - Address root causes, not just symptoms

2. **Debug with logs**:
   - Use the `tracing` crate for logging
   - Include context in log messages
   - Set appropriate log levels

3. **Analyze deadlocks**:
   - Check for lock ordering issues
   - Ensure locks are released properly
   - Minimize the duration locks are held

4. **Performance issues**:
   - Use profiling tools to identify bottlenecks
   - Consider lock-free alternatives where appropriate
   - Optimize critical paths

## Maintenance

Regular maintenance tasks include:

1. **Dependency updates**:
   - Review and update dependencies monthly
   - Test thoroughly after updates
   - Document any breaking changes

2. **Code cleanup**:
   - Remove deprecated functionality
   - Consolidate duplicate code
   - Improve documentation

3. **Performance optimization**:
   - Identify and address performance bottlenecks
   - Benchmark critical operations
   - Consider parallel execution for suitable tasks 