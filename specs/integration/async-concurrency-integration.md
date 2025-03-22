---
version: 1.0.0
last_updated: 2024-03-31
status: active
priority: high
phase: 1
---

# Async Concurrency Integration Pattern

## Overview

This document specifies the async concurrency integration patterns for the Squirrel codebase, focusing on thread-safe state management, proper async lock usage, and synchronization across components. These patterns were established during the context system implementation and refined through the async mutex refactoring process.

## Integration Status

- Current Progress: 100% (Context Management System)
- Current Progress: 100% (Plugin System)
- Target Components: All async components
- Priority: High

## Async Concurrency Architecture

### 1. Async-Aware Lock Management

```rust
use tokio::sync::{Mutex, RwLock};
use std::sync::Arc;
use std::collections::HashMap;

// Thread-safe component with async-aware locking
pub struct ThreadSafeComponent {
    // Use tokio's async-aware RwLock for read-heavy state
    shared_state: Arc<RwLock<HashMap<String, String>>>,
    
    // Use tokio's Mutex for write-heavy or exclusive access state
    internal_counters: Arc<Mutex<Vec<u64>>>,
}

impl ThreadSafeComponent {
    pub fn new() -> Self {
        Self {
            shared_state: Arc::new(RwLock::new(HashMap::new())),
            internal_counters: Arc::new(Mutex::new(Vec::new())),
        }
    }
}
```

### 2. Safe Async State Access

```rust
impl ThreadSafeComponent {
    // Read operation with minimal lock duration
    pub async fn get_value(&self, key: &str) -> Option<String> {
        // Scope the lock to minimize duration
        let value = {
            let state = self.shared_state.read().await;
            state.get(key).cloned()
        }; // Lock is dropped here
        
        value
    }
    
    // Write operation without holding lock across await points
    pub async fn set_value(&self, key: String, value: String) -> Result<(), Error> {
        // First phase: Update state with minimal lock duration
        {
            let mut state = self.shared_state.write().await;
            state.insert(key, value);
        } // Write lock is dropped here
        
        // Second phase: Perform async operations without holding lock
        self.notify_change().await?;
        
        Ok(())
    }
    
    // INCORRECT: Holding lock across await point
    pub async fn set_value_unsafe(&self, key: String, value: String) -> Result<(), Error> {
        let mut state = self.shared_state.write().await;
        state.insert(key, value);
        
        // INCORRECT: Lock is still held during this await operation!
        self.notify_change().await?;
        
        Ok(())
    }
}
```

### 3. Two-Phase Locking Pattern

```rust
impl ThreadSafeComponent {
    // Two-phase locking for conditional operations
    pub async fn conditional_update(&self, key: &str, new_value: String) -> Result<bool, Error> {
        // Phase 1: Read with read lock to check condition
        let should_update = {
            let state = self.shared_state.read().await;
            match state.get(key) {
                Some(current_value) => current_value != &new_value,
                None => true,
            }
        }; // Read lock is dropped here
        
        // Phase 2: Write with write lock if condition met
        if should_update {
            let mut state = self.shared_state.write().await;
            state.insert(key.to_string(), new_value);
            drop(state); // Explicit drop for clarity
            
            // Perform async operations after lock is released
            self.notify_change().await?;
            
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
```

### 4. Concurrent Operation Management

```rust
use tokio::sync::Semaphore;

impl ThreadSafeComponent {
    // Process concurrent operations with controlled parallelism
    pub async fn process_concurrent_operations(
        &self, 
        operations: Vec<Operation>,
        max_concurrent: usize
    ) -> Result<Vec<OperationResult>, Error> {
        // Create semaphore to limit concurrent operations
        let semaphore = Arc::new(Semaphore::new(max_concurrent));
        let mut handles = Vec::with_capacity(operations.len());
        
        // Launch tasks with controlled concurrency
        for operation in operations {
            let semaphore_clone = Arc::clone(&semaphore);
            let component = self.clone();
            
            let handle = tokio::spawn(async move {
                // Acquire permit, limiting concurrency
                let _permit = semaphore_clone.acquire().await.unwrap();
                
                // Process operation
                component.process_single_operation(operation).await
            });
            
            handles.push(handle);
        }
        
        // Collect results maintaining order
        let mut results = Vec::with_capacity(handles.len());
        for handle in handles {
            match handle.await {
                Ok(Ok(result)) => results.push(result),
                Ok(Err(e)) => return Err(e),
                Err(e) => return Err(Error::JoinError(e.to_string())),
            }
        }
        
        Ok(results)
    }
}
```

## Integration Requirements

### 1. Async Lock Usage Requirements

- **Always use tokio's async-aware locks**:
  - Use `tokio::sync::RwLock` instead of `std::sync::RwLock`
  - Use `tokio::sync::Mutex` instead of `std::sync::Mutex`

- **Never hold locks across await points**:
  - Release locks before any `.await` operation
  - Clone or copy data if needed for processing outside locks

- **Minimize lock duration**:
  - Keep lock scopes as small as possible
  - Use explicit scope blocks with `{}` to make lock lifetimes clear
  - Consider using explicit `drop(guard)` for clarity

- **Use appropriate lock types**:
  - Use `RwLock` for read-heavy data structures
  - Use `Mutex` for exclusive access requirements
  - Consider using `tokio::sync::RwLock::read_owned()` for long-lived read locks

- **Implement proper error handling**:
  - Handle lock acquisition failures
  - Provide clear error messages for lock-related errors
  - Consider using timeouts for lock acquisition in critical code paths

### 2. Data Structure Integration

- **Shared state containers**:
  - Wrap shared state in `Arc<RwLock<T>>` or `Arc<Mutex<T>>`
  - Implement methods for safe concurrent access
  - Document thread safety guarantees

- **Read-write separation**:
  - Separate read and write operations for optimal concurrency
  - Use two-phase locking for complex operations (read first, then write if needed)
  - Consider implementing Copy or Clone for data structures that need processing outside locks

- **Lock granularity**:
  - Use fine-grained locks for high-contention data structures
  - Consider sharding data for better concurrency
  - Document lock ordering to prevent deadlocks

### 3. Cross-Component Integration

- **Component boundaries**:
  - Release locks before crossing component boundaries
  - Use message passing for cross-component communication when possible
  - Document locking responsibilities in component interfaces

- **Event propagation**:
  - Release locks before emitting events
  - Process events without holding locks
  - Use asynchronous event channels (e.g., `tokio::sync::mpsc`)

- **Resource management**:
  - Implement proper resource cleanup with drop guards
  - Use `async_trait` for async trait implementations
  - Document resource acquisition and release patterns

## Implementation Patterns

### 1. Shared State Manager Pattern

```rust
pub struct SharedStateManager<T: Clone + Send + Sync + 'static> {
    state: Arc<RwLock<T>>,
    listeners: Arc<RwLock<Vec<Sender<StateUpdate<T>>>>>,
}

impl<T: Clone + Send + Sync + 'static> SharedStateManager<T> {
    // Create with initial state
    pub fn new(initial_state: T) -> Self {
        Self {
            state: Arc::new(RwLock::new(initial_state)),
            listeners: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    // Safe state update with notification
    pub async fn update<F, R>(&self, updater: F) -> Result<R, Error>
    where
        F: FnOnce(&mut T) -> Result<R, Error>,
    {
        // Phase 1: Update state with minimal lock duration
        let (result, state_clone) = {
            let mut state = self.state.write().await;
            let result = updater(&mut *state)?;
            (result, state.clone())
        }; // Lock released here
        
        // Phase 2: Notify listeners without holding lock
        self.notify_listeners(StateUpdate::new(state_clone)).await?;
        
        Ok(result)
    }
    
    // Read state with minimal lock duration
    pub async fn get(&self) -> Result<T, Error> {
        let state = {
            let guard = self.state.read().await;
            guard.clone()
        }; // Lock released here
        
        Ok(state)
    }
    
    // Register for state updates
    pub async fn register_listener(&self) -> Result<Receiver<StateUpdate<T>>, Error> {
        let (tx, rx) = mpsc::channel(32);
        
        {
            let mut listeners = self.listeners.write().await;
            listeners.push(tx);
        } // Lock released here
        
        Ok(rx)
    }
    
    // Notify all listeners of state change
    async fn notify_listeners(&self, update: StateUpdate<T>) -> Result<(), Error> {
        let listeners = {
            let guard = self.listeners.read().await;
            guard.clone()
        }; // Lock released here
        
        for listener in listeners {
            if listener.send(update.clone()).await.is_err() {
                // Handle closed channel, possibly remove listener later
            }
        }
        
        Ok(())
    }
}
```

### 2. Concurrent Task Processor Pattern

```rust
pub struct TaskProcessor<T: Send + 'static, R: Send + 'static> {
    max_concurrent: usize,
    processing_fn: Arc<dyn Fn(T) -> BoxFuture<'static, Result<R, Error>> + Send + Sync>,
}

impl<T: Send + 'static, R: Send + 'static> TaskProcessor<T, R> {
    // Create with processing function and concurrency limit
    pub fn new<F>(processing_fn: F, max_concurrent: usize) -> Self
    where
        F: Fn(T) -> BoxFuture<'static, Result<R, Error>> + Send + Sync + 'static,
    {
        Self {
            max_concurrent,
            processing_fn: Arc::new(processing_fn),
        }
    }
    
    // Process multiple tasks with controlled concurrency
    pub async fn process_all(&self, tasks: Vec<T>) -> Result<Vec<R>, Error> {
        // Create semaphore to limit concurrency
        let semaphore = Arc::new(Semaphore::new(self.max_concurrent));
        let processing_fn = self.processing_fn.clone();
        
        // Process with controlled parallelism
        let results = futures::future::try_join_all(
            tasks.into_iter().map(|task| {
                let sem = semaphore.clone();
                let proc_fn = processing_fn.clone();
                
                async move {
                    let _permit = sem.acquire().await.map_err(|e| 
                        Error::SemaphoreError(e.to_string()))?;
                    
                    (proc_fn)(task).await
                }
            })
        ).await?;
        
        Ok(results)
    }
}
```

### 3. Lock-Free State Pattern

For some use cases, consider lock-free approaches:

```rust
use std::sync::atomic::{AtomicU64, Ordering};

pub struct Metrics {
    // Use atomics for simple counters
    request_count: AtomicU64,
    error_count: AtomicU64,
    last_request_time: AtomicU64,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            request_count: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
            last_request_time: AtomicU64::new(0),
        }
    }
    
    // Lock-free counter increment
    pub fn record_request(&self) {
        self.request_count.fetch_add(1, Ordering::Relaxed);
        self.last_request_time.store(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            Ordering::Relaxed
        );
    }
    
    // Lock-free error recording
    pub fn record_error(&self) {
        self.error_count.fetch_add(1, Ordering::Relaxed);
    }
    
    // Read metrics without locks
    pub fn get_metrics(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            request_count: self.request_count.load(Ordering::Relaxed),
            error_count: self.error_count.load(Ordering::Relaxed),
            last_request_time: self.last_request_time.load(Ordering::Relaxed),
        }
    }
}
```

## Integration Tests

Tests for async concurrency patterns should focus on correctness, performance, and scalability:

### 1. Concurrent Access Testing

```rust
#[tokio::test]
async fn test_concurrent_operations() {
    // Setup test component
    let component = ThreadSafeComponent::new();
    
    // Launch multiple concurrent operations
    let mut handles = Vec::new();
    for i in 0..100 {
        let component_clone = component.clone();
        let key = format!("key-{}", i % 10); // Create some key contention
        let value = format!("value-{}", i);
        
        let handle = tokio::spawn(async move {
            component_clone.set_value(key, value).await
        });
        
        handles.push(handle);
    }
    
    // Wait for all operations to complete
    for handle in handles {
        handle.await.unwrap().unwrap();
    }
    
    // Verify final state
    for i in 0..10 {
        let key = format!("key-{}", i);
        let value = component.get_value(&key).await;
        assert!(value.is_some());
    }
}
```

### 2. Lock Contention Testing

```rust
#[tokio::test]
async fn test_lock_contention() {
    use std::time::Instant;
    
    // Setup component with shared state
    let component = ThreadSafeComponent::new();
    
    // Measure performance under different concurrency levels
    for concurrency in [1, 2, 4, 8, 16, 32] {
        let start = Instant::now();
        
        // Create controlled concurrency
        let semaphore = Arc::new(Semaphore::new(concurrency));
        let mut handles = Vec::new();
        
        for i in 0..1000 {
            let component_clone = component.clone();
            let semaphore_clone = semaphore.clone();
            
            let handle = tokio::spawn(async move {
                let _permit = semaphore_clone.acquire().await.unwrap();
                
                if i % 2 == 0 {
                    // Write operation
                    component_clone.set_value(
                        format!("key-{}", i % 100),
                        format!("value-{}", i)
                    ).await
                } else {
                    // Read operation
                    component_clone.get_value(&format!("key-{}", i % 100)).await;
                    Ok(())
                }
            });
            
            handles.push(handle);
        }
        
        // Wait for completion
        for handle in handles {
            handle.await.unwrap().unwrap();
        }
        
        let duration = start.elapsed();
        println!("Concurrency {}: {:?}", concurrency, duration);
    }
}
```

### 3. Deadlock Detection Testing

```rust
#[tokio::test]
async fn test_deadlock_prevention() {
    // Setup component
    let component = ThreadSafeComponent::new();
    
    // Create two operations that acquire locks in different orders
    let op1 = async {
        // Operation 1 order: A -> B
        component.operation_a().await?;
        component.operation_b().await?;
        Ok::<_, Error>(())
    };
    
    let op2 = async {
        // Operation 2 order: B -> A
        component.operation_b().await?;
        component.operation_a().await?;
        Ok::<_, Error>(())
    };
    
    // Run with timeout to detect potential deadlocks
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        futures::future::join(op1, op2)
    ).await;
    
    // Should complete without timeout
    assert!(result.is_ok());
}
```

## Best Practices

### 1. Lock Management

- Keep lock durations as short as possible
- Never hold locks across await points
- Use scoped blocks to make lock lifetimes explicit
- Document lock acquisition patterns

### 2. Concurrent Access

- Design APIs to minimize lock contention
- Use read locks for read-only operations
- Implement proper batching for bulk operations
- Consider sharding for highly contended data

### 3. Testing

- Test concurrent access patterns
- Measure performance under varying loads
- Test edge cases with high contention
- Implement tests for deadlock prevention

### 4. Documentation

- Document thread safety guarantees
- Specify lock ordering requirements
- Provide clear examples of proper usage
- Document performance characteristics

## Implementation Across Components

The async concurrency pattern has been successfully implemented in the following components:

1. **Context Management System**: Completely refactored to use async-aware locks and proper concurrency patterns.
2. **Plugin System**: Implemented proper async locking patterns for plugin lifecycle management.

The pattern should be applied to all other async components in the codebase:

1. **MCP Protocol Handler**: Handles concurrent client connections and requests.
2. **Command System**: Processes commands with proper concurrency control.
3. **Integration Components**: Manages integration between different subsystems.
4. **Monitoring Systems**: Collects metrics across components without blocking.

## Version History

- 1.0.0 (2024-03-31): Initial version based on context and plugin system implementations

<version>1.0.0</version> 