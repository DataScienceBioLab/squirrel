# Dependency Injection Patterns

## Overview

This document provides a guide to the dependency injection (DI) patterns used throughout the Squirrel codebase after the migration from singleton-based architecture to explicit dependency injection. This approach makes code more testable, maintainable, and helps avoid issues with global state.

## Core Patterns

### 1. Adapter Pattern

The adapter pattern is the central DI pattern used in our codebase. It provides explicit initialization, proper error handling, and component management.

```rust
// Example adapter pattern
pub struct ComponentAdapter {
    inner: Option<Arc<RwLock<Component>>>,
    initialized: bool,
}

impl ComponentAdapter {
    pub fn new() -> Self {
        Self {
            inner: None,
            initialized: false,
        }
    }

    pub async fn initialize(&mut self, config: Config) -> Result<()> {
        if self.initialized {
            return Err(SquirrelError::AlreadyInitialized("Already initialized".to_string()));
        }
        
        // Initialize the component
        let component = Component::new(config);
        self.inner = Some(Arc::new(RwLock::new(component)));
        self.initialized = true;
        Ok(())
    }
    
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    
    async fn ensure_initialized(&self) -> Result<()> {
        if !self.initialized {
            return Err(SquirrelError::NotInitialized("Not initialized".to_string()));
        }
        Ok(())
    }
    
    // Component methods with initialization checks
    pub async fn some_operation(&self, data: &str) -> Result<String> {
        self.ensure_initialized().await?;
        
        let component = self.inner.as_ref()
            .ok_or_else(|| SquirrelError::NotInitialized("Inner is None".to_string()))?;
            
        component.write().await.some_operation(data).await
    }
}
```

### 2. Factory Functions

Factory functions provide a convenient way to create and initialize components in a single operation:

```rust
// Factory function example
pub async fn create_initialized_component(config: Config) -> Result<ComponentAdapter> {
    let mut adapter = ComponentAdapter::new();
    adapter.initialize(config).await?;
    Ok(adapter)
}

pub async fn create_default_component() -> Result<ComponentAdapter> {
    create_initialized_component(Config::default()).await
}
```

### 3. Standardized Error Handling

All components use standardized error types for initialization-related errors:

```rust
// Error types for initialization
pub enum SquirrelError {
    // Other variants...
    
    #[error("Not initialized: {0}")]
    NotInitialized(String),
    
    #[error("Already initialized: {0}")]
    AlreadyInitialized(String),
}
```

## Usage Examples

### Example 1: Creating and Using an AppAdapter

```rust
// Create and initialize
let mut app_adapter = AppAdapter::new();
let config = AppConfig::default();
app_adapter.initialize(config).await?;

// Use the adapter
app_adapter.start().await?;

// Access component methods
let context = app_adapter.context().await?;
```

### Example 2: Using Factory Functions

```rust
// Create with factory function
let app_adapter = create_initialized_app_adapter(AppConfig::default()).await?;

// Use the adapter directly
app_adapter.start().await?;
```

### Example 3: Integration Testing

```rust
// Set up test context with multiple components
let context = IntegrationTestContext::new().await?;

// Work with components
context.app.start().await?;
context.sync.synchronize().await?;

// Test interaction between components
let app_context = context.app.context().await?;
// ... test interactions
```

## Migrating from Singletons

When migrating from singleton patterns to DI:

1. Identify components using global state or singletons
2. Create an adapter class for the component
3. Add explicit initialization with error handling
4. Update all code that uses the component to properly initialize
5. Create factory functions for common initialization patterns
6. Update tests to use the new pattern

### Before (singleton pattern):

```rust
// Before: Global instance or on-demand creation
pub fn get_instance() -> &'static mut Component {
    static mut INSTANCE: Option<Component> = None;
    
    unsafe {
        if INSTANCE.is_none() {
            INSTANCE = Some(Component::new());
        }
        INSTANCE.as_mut().unwrap()
    }
}

// Usage
let component = Component::get_instance();
component.do_something();
```

### After (DI pattern):

```rust
// After: Explicit initialization
let mut adapter = ComponentAdapter::new();
adapter.initialize(config).await?;

// Or with factory
let adapter = create_initialized_component(config).await?;

// Usage
adapter.do_something().await?;
```

## Testing with DI

The DI pattern makes testing much simpler:

```rust
#[tokio::test]
async fn test_component() {
    // Arrange: Create with test configuration
    let config = TestConfig::default();
    let adapter = create_initialized_component(config).await.unwrap();
    
    // Act: Perform operation
    let result = adapter.some_operation("test").await;
    
    // Assert: Check results
    assert!(result.is_ok());
}
```

## Best Practices

1. **Always check initialization**: All adapter methods should check if the component is initialized
2. **Use factory functions**: Provide factory functions for common initialization patterns
3. **Standardize error handling**: Use consistent error types for initialization errors
4. **Document initialization requirements**: Make initialization requirements clear in documentation
5. **Test initialization failure**: Add tests for both successful and failed initialization
6. **Make dependencies explicit**: Constructor parameters should clearly show dependencies
7. **Use Arc/RwLock properly**: Wrap components that need concurrent access in appropriate synchronization primitives
8. **Initialize in one place**: Avoid distributed initialization of components

## Components Using DI

The following components have been migrated to use DI patterns:

1. **AppAdapter**: The main application adapter
2. **MCPSync**: Synchronization module
3. **MCPProtocolAdapter**: Protocol handling adapter 
4. **MCPContextAdapter**: Context management adapter

More components will be migrated in future phases of the project. 