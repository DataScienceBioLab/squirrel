---
version: 1.1.0
last_updated: 2024-03-31
status: active
priority: high
---

# Integration Patterns

## Overview
This document defines standard integration patterns for component interactions within the Squirrel platform. These patterns provide consistent approaches for implementing communication, state management, and error handling across different parts of the system.

## Pattern Categories

### 1. Component Communication Patterns

#### A. Service Interface Pattern
Components expose functionality through well-defined trait interfaces:

```rust
#[async_trait]
pub trait MyService: Send + Sync {
    async fn operation_one(&self, param: ParamType) -> Result<ResponseType>;
    async fn operation_two(&self, param: OtherParamType) -> Result<OtherResponseType>;
}

// Implementation
pub struct MyServiceImpl {
    // Dependencies
    dependency: Arc<dyn DependencyService>,
    // State
    state: Arc<RwLock<ServiceState>>,
}

#[async_trait]
impl MyService for MyServiceImpl {
    async fn operation_one(&self, param: ParamType) -> Result<ResponseType> {
        // Implementation
    }
    
    async fn operation_two(&self, param: OtherParamType) -> Result<OtherResponseType> {
        // Implementation
    }
}
```

**When to Use:**
- For core service capabilities that may have multiple implementations
- When the component's functionality needs to be accessed by multiple consumers
- For services that may be mocked in tests

**Benefits:**
- Clear API contract
- Implementation flexibility
- Testability through mocking
- Better separation of concerns

#### B. Event-Based Communication Pattern
Components communicate through an event bus without direct coupling:

```rust
// Event definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemEvent {
    StateChanged(StateChangeEvent),
    ResourceUpdated(ResourceUpdateEvent),
    UserAction(UserActionEvent),
    Error(ErrorEvent),
}

// Event bus interface
#[async_trait]
pub trait EventBus: Send + Sync {
    async fn publish(&self, event: SystemEvent) -> Result<()>;
    async fn subscribe(&self, topic: EventTopic) -> Result<mpsc::Receiver<SystemEvent>>;
}

// Component using event bus
pub struct ComponentWithEvents {
    event_bus: Arc<dyn EventBus>,
    // Other fields
}

impl ComponentWithEvents {
    pub async fn handle_event(&self, event: SystemEvent) -> Result<()> {
        match event {
            SystemEvent::StateChanged(e) => self.on_state_changed(e).await,
            SystemEvent::ResourceUpdated(e) => self.on_resource_updated(e).await,
            _ => Ok(()), // Ignore other events
        }
    }
    
    pub async fn trigger_action(&self) -> Result<()> {
        // Perform action
        let result = self.perform_action().await?;
        
        // Publish event
        self.event_bus.publish(SystemEvent::ResourceUpdated(
            ResourceUpdateEvent::new(result)
        )).await?;
        
        Ok(())
    }
}
```

**When to Use:**
- For loosely coupled components
- When multiple components need to react to the same events
- For asynchronous workflows where components operate independently

**Benefits:**
- Reduced coupling between components
- Easier to add new components without changing existing ones
- Simplified scaling and distribution
- Better handling of asynchronous operations

#### C. Async Concurrency Pattern
Components interact safely in an async environment using proper concurrency controls:

```rust
// Thread-safe component with async-aware locking
pub struct ThreadSafeComponent {
    // Use tokio's async-aware RwLock for read-heavy state
    shared_state: Arc<RwLock<HashMap<String, Value>>>,
    // Use tokio's Mutex for write-heavy or exclusive access state
    internal_counters: Arc<Mutex<Vec<u64>>>,
}

impl ThreadSafeComponent {
    // Safe async read operation with minimal lock duration
    pub async fn get_value(&self, key: &str) -> Option<Value> {
        // Scope the lock to minimize duration
        let value = {
            let state = self.shared_state.read().await;
            state.get(key).cloned()
        }; // Lock is dropped here
        
        value
    }
    
    // Safe async write operation without holding lock across await points
    pub async fn set_value(&self, key: String, value: Value) -> Result<(), Error> {
        // First phase: Update state with minimal lock duration
        {
            let mut state = self.shared_state.write().await;
            state.insert(key, value);
        } // Write lock is dropped here
        
        // Second phase: Perform async operations without holding lock
        self.notify_change().await?;
        
        Ok(())
    }
    
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

**When to Use:**
- For components that use async operations and shared state
- When implementing services that handle concurrent requests
- For any state that may be accessed by multiple async tasks

**Benefits:**
- Thread-safe state management in async code
- Proper handling of async locks to prevent deadlocks
- Controlled parallelism with backpressure
- Clear patterns for non-blocking operations

**Reference Implementation:** [Async Concurrency Integration Pattern](async-concurrency-integration.md)

### 2. State Management Patterns

#### A. Shared State Pattern
Multiple components access shared state with proper synchronization:

```rust
// State definition
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SharedState {
    pub data: HashMap<String, Value>,
    pub version: u64,
    pub last_updated: DateTime<Utc>,
}

// State manager
pub struct StateManager {
    state: Arc<RwLock<SharedState>>,
    listeners: Arc<RwLock<Vec<StateChangeListener>>>,
}

impl StateManager {
    pub async fn update<F, T>(&self, updater: F) -> Result<T>
    where
        F: FnOnce(&mut SharedState) -> Result<T>,
    {
        let mut state = self.state.write().await;
        let result = updater(&mut state)?;
        state.version += 1;
        state.last_updated = Utc::now();
        
        // Notify listeners
        self.notify_state_change(&state).await?;
        
        Ok(result)
    }
    
    pub async fn get(&self) -> Result<SharedState> {
        Ok(self.state.read().await.clone())
    }
}

// Component using shared state
pub struct ComponentWithSharedState {
    state_manager: Arc<StateManager>,
}

impl ComponentWithSharedState {
    pub async fn perform_operation(&self) -> Result<()> {
        self.state_manager.update(|state| {
            state.data.insert("key".to_string(), Value::String("value".to_string()));
            Ok(())
        }).await?;
        
        Ok(())
    }
}
```

**When to Use:**
- When multiple components need access to the same state
- For centralized state management
- When state changes need to trigger reactions in multiple components

**Benefits:**
- Single source of truth
- Consistent state access
- Proper synchronization
- Change notification

#### B. State Synchronization Pattern
Components maintain local state that synchronizes with a central source:

```rust
// State synchronization interface
#[async_trait]
pub trait StateSynchronizer {
    async fn push_state(&self, state: LocalState) -> Result<()>;
    async fn pull_state(&self) -> Result<RemoteState>;
    async fn sync(&self) -> Result<SyncResult>;
}

// Component with local state
pub struct ComponentWithLocalState {
    local_state: RwLock<LocalState>,
    synchronizer: Arc<dyn StateSynchronizer>,
    last_sync: AtomicU64,
}

impl ComponentWithLocalState {
    pub async fn sync_state(&self) -> Result<()> {
        // Pull remote state
        let remote_state = self.synchronizer.pull_state().await?;
        
        // Merge with local state
        let mut local = self.local_state.write().await;
        *local = self.merge_states(&local, &remote_state)?;
        
        // Update last sync time
        self.last_sync.store(Utc::now().timestamp() as u64, Ordering::SeqCst);
        
        Ok(())
    }
    
    pub async fn update_local_state(&self) -> Result<()> {
        // Update local state
        let mut local = self.local_state.write().await;
        // Modify local state...
        
        // Push to remote if enough time has passed since last sync
        let now = Utc::now().timestamp() as u64;
        let last = self.last_sync.load(Ordering::SeqCst);
        
        if now - last > self.sync_interval {
            self.synchronizer.push_state(local.clone()).await?;
            self.last_sync.store(now, Ordering::SeqCst);
        }
        
        Ok(())
    }
}
```

**When to Use:**
- For distributed components that need their own state copy
- When network connectivity might be unreliable
- For performance-critical components that need local state access

**Benefits:**
- Better performance for local operations
- Resilience to network issues
- Reduced contention on central state
- Control over synchronization frequency

#### C. Context-Based State Pattern
Components access and update state through a managed context system:

```rust
// Context-aware component
pub struct ContextAwareComponent {
    context_manager: Arc<ContextManager>,
    component_id: ComponentId,
}

impl ContextAwareComponent {
    // Perform operation with context
    pub async fn perform_operation(
        &self,
        context_id: ContextId,
        params: OperationParams
    ) -> Result<OperationResult, Error> {
        // Get context for this operation
        let context = self.context_manager.get_context(context_id).await?;
        
        // Check permissions in context
        if !context.has_permission(self.component_id, Permission::Execute) {
            return Err(Error::PermissionDenied);
        }
        
        // Read state from context
        let config = context.state().get::<Config>("config")?;
        
        // Perform operation with context state
        let result = self.execute_with_config(&config, params).await?;
        
        // Update context with operation results
        self.context_manager
            .update_context(context_id, |ctx| {
                // Update state atomically
                ctx.set("last_result", &result)?;
                ctx.set("last_operation_time", Utc::now())?;
                Ok(())
            })
            .await?;
        
        Ok(result)
    }
    
    // Listen for context changes
    pub async fn start_context_monitoring(&self, context_id: ContextId) -> Result<(), Error> {
        // Subscribe to context events
        let mut events = self.context_manager
            .subscribe_context_events(context_id)
            .await?;
        
        // Handle context updates
        tokio::spawn(async move {
            while let Some(event) = events.recv().await {
                match event {
                    ContextEvent::Updated { id, changes } => {
                        if changes.contains_key("config") {
                            // React to config changes
                            self.on_config_updated(id).await
                                .unwrap_or_else(|e| log::error!("Failed to update: {}", e));
                        }
                    },
                    // Handle other events
                    _ => {},
                }
            }
        });
        
        Ok(())
    }
}
```

**When to Use:**
- When components need to share state with clear access boundaries
- For systems with complex permission requirements
- When integrating with protocol handlers like MCP

**Benefits:**
- Centralized state management with clear ownership
- Permission-based access control
- Event-driven updates
- Transactional state changes

**Reference Implementation:** [MCP-Context Integration Pattern](mcp-context-integration.md)

### 3. Error Handling Patterns

#### A. Error Propagation Pattern
Components propagate errors with appropriate context:

```rust
// Error types
#[derive(Debug, Error)]
pub enum ComponentError {
    #[error("Operation failed: {0}")]
    OperationFailed(String),
    
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Dependency error: {0}")]
    DependencyError(#[from] DependencyError),
    
    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

// Component with proper error handling
impl MyComponent {
    pub async fn operation(&self) -> Result<(), ComponentError> {
        // Try dependency operation
        let result = self.dependency
            .perform_operation()
            .await
            .map_err(|e| {
                // Add context to error
                ComponentError::DependencyError(e)
            })?;
        
        // Check result
        if !self.validate_result(&result) {
            return Err(ComponentError::OperationFailed(
                format!("Validation failed for result: {:?}", result)
            ));
        }
        
        Ok(())
    }
}
```

**When to Use:**
- For all component boundaries
- When detailed error information is needed for debugging
- When errors need to be handled at different levels of the system

**Benefits:**
- Clear error context
- Proper error categorization
- Easy to track error source
- Better debugging experience

#### B. Circuit Breaker Pattern
Prevents cascading failures when a dependency fails repeatedly:

```rust
pub struct CircuitBreaker {
    state: AtomicU8,
    failure_count: AtomicUsize,
    last_failure: AtomicU64,
    settings: CircuitBreakerSettings,
}

impl CircuitBreaker {
    pub async fn execute<F, T, E>(&self, operation: F) -> Result<T, E>
    where
        F: Future<Output = Result<T, E>>,
        E: std::error::Error,
    {
        match self.get_state() {
            CircuitState::Closed => {
                // Normal operation
                match operation.await {
                    Ok(result) => {
                        // Reset failure count on success
                        self.failure_count.store(0, Ordering::SeqCst);
                        Ok(result)
                    }
                    Err(e) => {
                        // Increment failure count
                        let failures = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
                        self.last_failure.store(Utc::now().timestamp() as u64, Ordering::SeqCst);
                        
                        // Trip circuit if threshold reached
                        if failures >= self.settings.failure_threshold {
                            self.trip_circuit();
                        }
                        
                        Err(e)
                    }
                }
            }
            CircuitState::Open => {
                // Circuit is open, fast fail
                Err(E::from_error(CircuitBreakerError::CircuitOpen))
            }
            CircuitState::HalfOpen => {
                // Test if the circuit can be closed again
                match operation.await {
                    Ok(result) => {
                        // Success, close the circuit
                        self.close_circuit();
                        Ok(result)
                    }
                    Err(e) => {
                        // Failed again, keep circuit open
                        self.trip_circuit();
                        Err(e)
                    }
                }
            }
        }
    }
}
```

**When to Use:**
- For operations that depend on external systems
- When failures in one component should not cascade to others
- For resilient system design

**Benefits:**
- Prevents cascading failures
- Allows systems to recover
- Reduces load on failing systems
- Improves overall system stability

### 4. Resource Management Patterns

#### A. Connection Pool Pattern
Manages shared connections to external resources:

```rust
pub struct ConnectionPool<T> {
    available: Mutex<Vec<T>>,
    max_size: usize,
    factory: Box<dyn Fn() -> Result<T> + Send + Sync>,
}

impl<T: Send + Sync + 'static> ConnectionPool<T> {
    pub async fn get(&self) -> Result<PooledConnection<T>> {
        // Try to get an available connection
        let mut available = self.available.lock().await;
        
        let conn = if let Some(conn) = available.pop() {
            conn
        } else if available.len() < self.max_size {
            // Create new connection if under max size
            (self.factory)()?
        } else {
            // Wait for a connection to become available
            return Err(PoolError::NoAvailableConnections);
        };
        
        // Return pooled connection
        Ok(PooledConnection {
            conn: Some(conn),
            pool: self,
        })
    }
    
    fn return_connection(&self, conn: T) {
        let mut available = self.available.lock().await;
        available.push(conn);
    }
}

// Pooled connection that returns to pool on drop
pub struct PooledConnection<'a, T> {
    conn: Option<T>,
    pool: &'a ConnectionPool<T>,
}

impl<'a, T> Drop for PooledConnection<'a, T> {
    fn drop(&mut self) {
        if let Some(conn) = self.conn.take() {
            self.pool.return_connection(conn);
        }
    }
}

// Usage
let pool = ConnectionPool::new(
    10, // max_size
    Box::new(|| {
        // Connection factory
        DatabaseConnection::connect(&config)
    })
);

let conn = pool.get().await?;
// Use connection
conn.execute_query("SELECT * FROM table").await?;
// Connection automatically returns to pool when dropped
```

**When to Use:**
- For managing expensive resources like database connections
- When resources are limited and need to be shared
- For improving performance by reusing resources

**Benefits:**
- Efficient resource utilization
- Controlled access to limited resources
- Automatic resource cleanup
- Improved performance

#### B. Resource Lifecycle Pattern
Manages resource creation, initialization, and cleanup:

```rust
#[async_trait]
pub trait ManagedResource: Send + Sync {
    async fn initialize(&self) -> Result<()>;
    async fn is_healthy(&self) -> bool;
    async fn cleanup(&self) -> Result<()>;
}

pub struct ResourceManager {
    resources: RwLock<HashMap<ResourceId, Arc<dyn ManagedResource>>>,
}

impl ResourceManager {
    pub async fn register<R>(&self, id: ResourceId, resource: R) -> Result<()>
    where
        R: ManagedResource + 'static,
    {
        let resource = Arc::new(resource);
        
        // Initialize the resource
        resource.initialize().await?;
        
        // Store in resources map
        let mut resources = self.resources.write().await;
        resources.insert(id, resource);
        
        Ok(())
    }
    
    pub async fn get(&self, id: &ResourceId) -> Result<Arc<dyn ManagedResource>> {
        let resources = self.resources.read().await;
        
        resources.get(id)
            .cloned()
            .ok_or_else(|| ResourceError::NotFound(id.clone()))
    }
    
    pub async fn cleanup_all(&self) -> Result<()> {
        let resources = self.resources.read().await;
        
        for (id, resource) in resources.iter() {
            if let Err(e) = resource.cleanup().await {
                log::warn!("Failed to clean up resource {}: {}", id, e);
            }
        }
        
        Ok(())
    }
}
```

**When to Use:**
- For managing resources with complex lifecycles
- When resources need proper initialization and cleanup
- For centralized resource management

**Benefits:**
- Proper resource initialization
- Controlled resource lifecycle
- Centralized management
- Guaranteed cleanup

## Implementation Examples

### 1. Service Interface Example

```rust
// MCP protocol service interface
#[async_trait]
pub trait McpService: Send + Sync {
    async fn send_message(&self, message: McpMessage) -> Result<McpResponse>;
    async fn receive_messages(&self) -> Result<mpsc::Receiver<McpMessage>>;
    async fn register_handler(&self, handler: Box<dyn McpMessageHandler>) -> Result<HandlerId>;
}

// Core component using MCP service
pub struct CoreComponent {
    mcp_service: Arc<dyn McpService>,
    state: Arc<RwLock<CoreState>>,
}

impl CoreComponent {
    pub fn new(mcp_service: Arc<dyn McpService>) -> Self {
        let component = Self {
            mcp_service,
            state: Arc::new(RwLock::new(CoreState::default())),
        };
        
        // Register message handler
        let handler = Box::new(CoreMessageHandler::new(component.state.clone()));
        let _ = component.mcp_service.register_handler(handler);
        
        component
    }
    
    pub async fn send_command(&self, command: Command) -> Result<CommandResponse> {
        // Convert to MCP message
        let message = McpMessage::new_command(command);
        
        // Send via MCP service
        let response = self.mcp_service.send_message(message).await?;
        
        // Convert response
        Ok(CommandResponse::from_mcp(response))
    }
}
```

### 2. Event-Based Communication Example

```rust
// Event definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemEvent {
    ContextChanged(ContextChangedEvent),
    ToolExecuted(ToolExecutedEvent),
    UserAction(UserActionEvent),
}

// UI component using events
pub struct UiComponent {
    event_bus: Arc<dyn EventBus>,
    state: Arc<RwLock<UiState>>,
}

impl UiComponent {
    pub fn new(event_bus: Arc<dyn EventBus>) -> Self {
        let component = Self {
            event_bus: event_bus.clone(),
            state: Arc::new(RwLock::new(UiState::default())),
        };
        
        // Subscribe to relevant events
        let event_bus_clone = event_bus.clone();
        let state_clone = component.state.clone();
        
        tokio::spawn(async move {
            let mut receiver = event_bus_clone.subscribe(
                EventTopic::Context | EventTopic::Tool
            ).await.unwrap();
            
            while let Some(event) = receiver.recv().await {
                match event {
                    SystemEvent::ContextChanged(e) => {
                        let mut state = state_clone.write().await;
                        state.update_from_context(e.context);
                    },
                    SystemEvent::ToolExecuted(e) => {
                        let mut state = state_clone.write().await;
                        state.update_tool_status(e.tool_id, e.status);
                    },
                    _ => {}
                }
            }
        });
        
        component
    }
    
    pub async fn user_action(&self, action: UserAction) -> Result<()> {
        // Perform local updates
        {
            let mut state = self.state.write().await;
            state.record_user_action(&action);
        }
        
        // Publish event
        self.event_bus.publish(SystemEvent::UserAction(
            UserActionEvent::new(action)
        )).await?;
        
        Ok(())
    }
}
```

## Best Practices

### 1. Interface Design
- Define clear trait interfaces for component boundaries
- Use async functions for potentially blocking operations
- Return Result types for operations that can fail
- Keep interfaces focused on a single responsibility

### 2. State Management
- Use appropriate synchronization primitives (RwLock, Mutex)
- Consider state ownership and access patterns
- Document state invariants and access requirements
- Implement proper change notification mechanisms

### 3. Error Handling
- Define component-specific error types
- Include context in error messages
- Implement proper error conversion between components
- Use circuit breakers for external dependencies

### 4. Resource Management
- Implement proper lifecycle management for resources
- Use connection pooling for expensive resources
- Ensure cleanup happens even in error cases
- Monitor resource usage and health

### 5. Testing
- Mock dependencies for unit testing
- Test integration points explicitly
- Simulate error conditions
- Verify state consistency across components

## Migration Strategies

When migrating components to use these patterns:

1. **Interface Abstraction**:
   - Extract interfaces from concrete implementations
   - Update consumers to use interface types
   - Implement adapters for legacy components

2. **Event Conversion**:
   - Introduce event types alongside direct calls
   - Gradually migrate to event-based communication
   - Use adapters to bridge direct and event-based approaches

3. **State Management**:
   - Encapsulate state behind proper abstractions
   - Introduce synchronization gradually
   - Use feature flags to toggle between approaches

4. **Error Handling**:
   - Define new error types
   - Add context to existing errors
   - Implement conversion between error types

## Conclusion

These integration patterns provide a foundation for building robust, maintainable component interactions within the Squirrel platform. By consistently applying these patterns, teams can ensure that components interact properly, handle errors gracefully, and manage state effectively.

## Protocol Integration Patterns

### A. MCP-Context Integration Pattern
Components interact with the Machine Context Protocol through a context-aware interface:

```rust
// MCP Session with Context integration
pub struct ManagedSession {
    session_id: SessionId,
    context_id: ContextId,
    context_manager: Arc<ContextManager>,
}

impl ManagedSession {
    // Process a tool request with context awareness
    pub async fn process_tool_request(
        &self,
        request: ToolRequest
    ) -> Result<ToolResponse, Error> {
        // Use the session's context to process the request
        let context = self.context_manager.get_context(self.context_id).await?;
        
        // Apply context to tool request
        let contextualized_request = request.with_context(context.clone());
        
        // Process the request with the appropriate tool
        let tool_manager = ToolManager::global();
        let result = tool_manager.execute_tool(contextualized_request).await?;
        
        // Update context with tool execution results if needed
        if let Some(context_updates) = result.context_updates() {
            self.context_manager
                .update_context(self.context_id, |ctx| {
                    ctx.apply_updates(context_updates)?;
                    Ok(())
                })
                .await?;
        }
        
        Ok(result.into_response())
    }
}

// Context-aware command processing
pub struct ContextualizedCommand<T> {
    inner: T,
    context_id: ContextId,
}

impl<T: Command> Command for ContextualizedCommand<T> {
    type Input = T::Input;
    type Output = T::Output;
    
    async fn execute(
        &self,
        input: Self::Input,
        cmd_context: &CommandContext
    ) -> Result<Self::Output, Error> {
        // Retrieve context for this command execution
        let context_manager = ContextManager::global();
        let context = context_manager.get_context(self.context_id).await?;
        
        // Enhance command context with our context information
        let enhanced_context = cmd_context.with_application_context(context);
        
        // Execute the inner command with the enhanced context
        let result = self.inner.execute(input, &enhanced_context).await?;
        
        // Update our context if command modified state
        if let Some(ctx_changes) = enhanced_context.context_changes() {
            context_manager
                .update_context(self.context_id, |ctx| {
                    ctx.apply_changes(ctx_changes)?;
                    Ok(())
                })
                .await?;
        }
        
        Ok(result)
    }
}
```

**When to Use:**
- When integrating components with the MCP protocol
- For tools and commands that need context awareness
- When implementing session management with state persistence

**Benefits:**
- Clean integration between MCP and Context systems
- Proper state handling for sessions and tools
- Consistent context management across protocol boundaries
- Clear permission boundaries for context access

**Reference Implementation:** [MCP-Context Integration Pattern](mcp-context-integration.md)

## Implementation Guidelines

### 1. Pattern Selection

When integrating components, select the appropriate patterns based on:

- The type of communication needed (direct calls, events, shared state)
- Performance requirements (latency, throughput)
- Reliability needs (error handling, retries)
- Distribution requirements (local vs. distributed)

Use multiple patterns when appropriate, but be consistent within each integration point.

### 2. Error Handling

All integration points should have clear error handling:

- Define specific error types for each integration boundary
- Implement proper error propagation
- Document error handling expectations
- Consider retries for transient failures
- Provide helpful error messages

### 3. Testing

Each integration pattern implementation should include:

- Unit tests for individual components
- Integration tests for component pairs
- System tests for end-to-end flows
- Performance tests for critical paths
- Chaos tests for reliability

### 4. Documentation

Document each integration point with:

- Pattern used
- Responsibility boundaries
- State ownership
- Error handling expectations
- Performance characteristics

## Version History

- 1.0.0 (2024-03-23): Initial version defining core integration patterns
- 1.1.0 (2024-03-31): Added Async Concurrency and MCP-Context integration patterns

<version>1.1.0</version> 