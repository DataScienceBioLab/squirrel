---
version: 1.0.0
last_updated: 2024-03-31
status: active
priority: high
phase: 1
---

# MCP-Context Integration Pattern

## Overview

This document specifies the integration patterns between the Machine Context Protocol (MCP) and the Context Management System. It outlines how MCP components interact with the context system to maintain consistent state across different execution environments, agents, and tools.

## Integration Status

- Current Progress: 100% (Context Management System)
- Current Progress: 90% (MCP Protocol Integration)
- Target Components: MCP Protocol Handler, Context Management System, Tool Management
- Priority: High

## MCP-Context Integration Architecture

### 1. Context ID and Session Management

```rust
use context::{ContextManager, ContextId, ContextKey};
use mcp_protocol::{Session, SessionId, ToolRequest};
use std::sync::Arc;

/// MCP Session with Context integration
pub struct ManagedSession {
    session_id: SessionId,
    context_id: ContextId,
    context_manager: Arc<ContextManager>,
}

impl ManagedSession {
    /// Create a new managed session with context tracking
    pub async fn new(
        session_id: SessionId, 
        initial_context: Option<ContextKey>
    ) -> Result<Self, Error> {
        let context_manager = ContextManager::global();
        
        // Create or retrieve a context for this session
        let context_id = match initial_context {
            Some(key) => context_manager.get_or_create_context(key).await?,
            None => context_manager.create_context().await?,
        };
        
        Ok(Self {
            session_id,
            context_id,
            context_manager: Arc::clone(&context_manager),
        })
    }
    
    /// Process a tool request with context awareness
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
```

### 2. Context Serialization for MCP Transport

```rust
use context::{Context, ContextState};
use mcp_protocol::{wire, Serializable};
use serde::{Serialize, Deserialize};

/// Context representation for MCP transport
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransportableContext {
    context_id: String,
    state: ContextState,
    metadata: HashMap<String, String>,
}

impl From<Context> for TransportableContext {
    fn from(context: Context) -> Self {
        Self {
            context_id: context.id().to_string(),
            state: context.state().clone(),
            metadata: context.metadata().clone(),
        }
    }
}

impl TryFrom<TransportableContext> for Context {
    type Error = Error;
    
    fn try_from(tc: TransportableContext) -> Result<Self, Self::Error> {
        Context::builder()
            .with_id(tc.context_id.parse()?)
            .with_state(tc.state)
            .with_metadata(tc.metadata)
            .build()
    }
}

impl Serializable for TransportableContext {
    fn serialize(&self) -> Result<Vec<u8>, wire::Error> {
        // Convert to wire format for transport
        wire::serialize(self)
    }
    
    fn deserialize(bytes: &[u8]) -> Result<Self, wire::Error> {
        // Reconstruct from wire format
        wire::deserialize(bytes)
    }
}
```

### 3. MCP Command Processing with Context

```rust
use context::{ContextManager, ContextId};
use mcp_protocol::{Command, CommandRegistry, CommandContext};

/// Context-aware command processing
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

/// Extension for the command registry to support contextualized commands
pub trait ContextCommandRegistryExt {
    fn register_with_context<T: Command + 'static>(
        &mut self,
        name: &str,
        command: T,
        context_id: ContextId
    ) -> Result<(), Error>;
}

impl ContextCommandRegistryExt for CommandRegistry {
    fn register_with_context<T: Command + 'static>(
        &mut self,
        name: &str,
        command: T,
        context_id: ContextId
    ) -> Result<(), Error> {
        let contextualized = ContextualizedCommand {
            inner: command,
            context_id,
        };
        
        self.register(name, contextualized)
    }
}
```

### 4. Context-Aware MCP Event System

```rust
use context::{ContextManager, ContextId, ContextEvent};
use mcp_protocol::{EventBus, Event, Subscriber};
use std::sync::Arc;

/// Context-aware event system integration
pub struct ContextEventBridge {
    context_manager: Arc<ContextManager>,
    event_bus: Arc<EventBus>,
}

impl ContextEventBridge {
    pub fn new(
        context_manager: Arc<ContextManager>,
        event_bus: Arc<EventBus>
    ) -> Self {
        Self {
            context_manager,
            event_bus,
        }
    }
    
    /// Start bridging context events to MCP events
    pub async fn start(&self) -> Result<(), Error> {
        // Subscribe to context events
        let mut context_events = self.context_manager.subscribe_events().await?;
        let event_bus = Arc::clone(&self.event_bus);
        
        // Process context events and convert to MCP events
        tokio::spawn(async move {
            while let Some(context_event) = context_events.recv().await {
                let mcp_event = match context_event {
                    ContextEvent::Created { id, initial_state } => {
                        Event::new("context.created", json!({
                            "context_id": id.to_string(),
                            "initial_state": initial_state,
                        }))
                    },
                    ContextEvent::Updated { id, changes } => {
                        Event::new("context.updated", json!({
                            "context_id": id.to_string(),
                            "changes": changes,
                        }))
                    },
                    ContextEvent::Deleted { id } => {
                        Event::new("context.deleted", json!({
                            "context_id": id.to_string(),
                        }))
                    },
                    // ... other context events
                };
                
                // Publish the converted event to MCP event bus
                if let Err(e) = event_bus.publish(mcp_event).await {
                    log::error!("Failed to publish MCP event: {}", e);
                }
            }
        });
        
        // Subscribe to relevant MCP events to update contexts
        let mcp_subscriber = ContextMcpSubscriber {
            context_manager: Arc::clone(&self.context_manager),
        };
        
        self.event_bus.subscribe("tool.executed", mcp_subscriber.clone()).await?;
        self.event_bus.subscribe("session.created", mcp_subscriber.clone()).await?;
        self.event_bus.subscribe("session.closed", mcp_subscriber).await?;
        
        Ok(())
    }
}

/// MCP event subscriber that updates contexts based on MCP events
#[derive(Clone)]
struct ContextMcpSubscriber {
    context_manager: Arc<ContextManager>,
}

impl Subscriber for ContextMcpSubscriber {
    async fn handle_event(&self, event: Event) -> Result<(), Error> {
        match event.event_type() {
            "tool.executed" => {
                // Extract context ID and updates from tool execution
                let context_id: ContextId = event.payload()
                    .get("context_id")
                    .ok_or_else(|| Error::MissingField("context_id"))?
                    .as_str()
                    .ok_or_else(|| Error::InvalidType("context_id"))?
                    .parse()?;
                
                let updates = event.payload()
                    .get("context_updates")
                    .ok_or_else(|| Error::MissingField("context_updates"))?;
                
                // Apply updates to the context
                self.context_manager
                    .update_context(context_id, |ctx| {
                        ctx.apply_updates_from_json(updates)?;
                        Ok(())
                    })
                    .await?;
            },
            "session.created" => {
                // Create new context for session if needed
                let session_id = event.payload()
                    .get("session_id")
                    .ok_or_else(|| Error::MissingField("session_id"))?
                    .as_str()
                    .ok_or_else(|| Error::InvalidType("session_id"))?;
                
                if let Some(context_key) = event.payload().get("context_key") {
                    let key = context_key.as_str()
                        .ok_or_else(|| Error::InvalidType("context_key"))?;
                    
                    self.context_manager
                        .get_or_create_context_with_key(key.into())
                        .await?;
                } else {
                    // No specific context requested, create new one
                    self.context_manager.create_context().await?;
                }
            },
            // ... handle other event types
            _ => {}
        }
        
        Ok(())
    }
}
```

## Integration Requirements

### 1. MCP Session-Context Binding

- **Session-to-Context Mapping**:
  - Each MCP session must be associated with exactly one Context
  - The Context ID must be included in session metadata
  - Sessions may share contexts with appropriate permissions

- **Context Persistence**:
  - Context must persist beyond session lifetime when required
  - Orphaned contexts should be garbage collected after a configurable period
  - Long-lived contexts must have a persistence key for retrieval

- **Context Recovery**:
  - MCP sessions must be able to reconnect to existing contexts
  - Context recovery mechanisms must handle corruption and version conflicts
  - Session authentication must validate context access permissions

### 2. MCP Tool Integration

- **Tool Execution Context**:
  - All tool executions must receive the current context
  - Tools must declare which context fields they read and modify
  - Context updates from tools must be atomic and validated

- **Context-Based Permissions**:
  - Tool access must respect permissions defined in context
  - Context changes must preserve security boundaries
  - Privilege escalation in context must require explicit authorization

- **Tool Result Integration**:
  - Tool results must include context updates when appropriate
  - Critical tool failures must not corrupt context state
  - Context recovery points should be created before risky operations

### 3. MCP Event Integration

- **Context Events**:
  - Context changes must trigger appropriate MCP events
  - MCP events must include minimal context information for privacy
  - Context events must be delivered in causal order

- **Event Authorization**:
  - Context event subscriptions must be authorized
  - Sensitive context data must be filtered from events based on subscriber permissions
  - Context change events must include the source of the change

- **Event Performance**:
  - High-frequency context changes must be batched
  - Context event subscribers must handle backpressure
  - Critical context operations must not be blocked by event delivery

## Implementation Patterns

### 1. Tool Execution with Context Pattern

```rust
pub struct ContextAwareToolExecutor {
    context_manager: Arc<ContextManager>,
    tool_registry: Arc<ToolRegistry>,
}

impl ContextAwareToolExecutor {
    pub async fn execute_tool(
        &self,
        tool_id: ToolId,
        args: ToolArgs,
        context_id: ContextId
    ) -> Result<ToolResult, Error> {
        // Acquire context for this tool execution
        let context = self.context_manager.get_context(context_id).await?;
        
        // Verify tool has permission to access this context
        self.verify_tool_permissions(tool_id, &context).await?;
        
        // Get the tool implementation
        let tool = self.tool_registry.get_tool(tool_id)?;
        
        // Create execution environment with context
        let env = ToolExecutionEnvironment::new()
            .with_context(context)
            .with_args(args);
        
        // Execute tool with transaction for context updates
        let result = self.context_manager
            .with_transaction(context_id, |ctx| {
                // Provide context to tool
                tool.execute_with_context(ctx, &env)
            })
            .await?;
        
        // Emit event about tool execution
        self.emit_tool_executed_event(tool_id, context_id, &result).await?;
        
        Ok(result)
    }
}
```

### 2. Context Sharing Pattern

```rust
/// Context sharing between MCP sessions
pub struct SharedContextManager {
    context_manager: Arc<ContextManager>,
    // Track which sessions are using which contexts
    session_contexts: Arc<RwLock<HashMap<SessionId, ContextId>>>,
    // Track which contexts are shared (reference counted)
    shared_contexts: Arc<RwLock<HashMap<ContextId, usize>>>,
}

impl SharedContextManager {
    /// Share a context with another session
    pub async fn share_context(
        &self,
        source_session: SessionId,
        target_session: SessionId,
        permissions: ContextPermissions
    ) -> Result<(), Error> {
        // Get the context ID for the source session
        let context_id = {
            let sessions = self.session_contexts.read().await;
            sessions.get(&source_session)
                .ok_or_else(|| Error::SessionNotFound(source_session))?
                .clone()
        };
        
        // Create a sharing link with permissions
        self.context_manager
            .create_sharing_link(context_id, permissions)
            .await?;
        
        // Associate the target session with this context
        {
            let mut sessions = self.session_contexts.write().await;
            sessions.insert(target_session, context_id);
            
            // Increment reference count for this context
            let mut shared = self.shared_contexts.write().await;
            *shared.entry(context_id).or_insert(0) += 1;
        }
        
        Ok(())
    }
    
    /// Release a context when a session ends
    pub async fn release_context(
        &self,
        session_id: SessionId
    ) -> Result<(), Error> {
        let context_id = {
            // Get and remove the context ID for this session
            let mut sessions = self.session_contexts.write().await;
            sessions.remove(&session_id)
                .ok_or_else(|| Error::SessionNotFound(session_id))?
        };
        
        // Decrement reference count for this context
        let should_cleanup = {
            let mut shared = self.shared_contexts.write().await;
            if let Some(count) = shared.get_mut(&context_id) {
                *count -= 1;
                *count == 0
            } else {
                true
            }
        };
        
        // If no more references, clean up the context
        if should_cleanup {
            self.context_manager.mark_for_cleanup(context_id).await?;
        }
        
        Ok(())
    }
}
```

### 3. Context Synchronization Pattern

```rust
/// Context synchronization for distributed MCP environments
pub struct DistributedContextSynchronizer {
    context_manager: Arc<ContextManager>,
    sync_channel: SyncChannel,
}

impl DistributedContextSynchronizer {
    /// Start synchronization for a context
    pub async fn start_sync(&self, context_id: ContextId) -> Result<(), Error> {
        // Subscribe to context changes
        let mut events = self.context_manager
            .subscribe_context_events(context_id)
            .await?;
        
        // Get current context state for initial sync
        let context = self.context_manager.get_context(context_id).await?;
        let initial_state = context.state().clone();
        
        // Send initial state to sync channel
        self.sync_channel
            .send_sync_message(SyncMessage::InitialState {
                context_id,
                state: initial_state,
            })
            .await?;
        
        // Process ongoing changes
        tokio::spawn(async move {
            while let Some(event) = events.recv().await {
                match event {
                    ContextEvent::Updated { id, changes } => {
                        // Send incremental update
                        self.sync_channel
                            .send_sync_message(SyncMessage::IncrementalUpdate {
                                context_id: id,
                                changes,
                            })
                            .await
                            .unwrap_or_else(|e| {
                                log::error!("Failed to send sync message: {}", e);
                            });
                    },
                    // Handle other events...
                    _ => {},
                }
            }
        });
        
        // Start receiving remote changes
        self.start_receiving_remote_changes(context_id).await?;
        
        Ok(())
    }
    
    /// Apply remote changes to local context
    async fn start_receiving_remote_changes(
        &self,
        context_id: ContextId
    ) -> Result<(), Error> {
        let context_manager = Arc::clone(&self.context_manager);
        let mut receiver = self.sync_channel.subscribe_to_context(context_id).await?;
        
        tokio::spawn(async move {
            while let Some(message) = receiver.recv().await {
                match message {
                    SyncMessage::IncrementalUpdate { context_id, changes } => {
                        // Apply remote changes to local context
                        if let Err(e) = context_manager
                            .update_context(context_id, |ctx| {
                                ctx.apply_changes(changes)?;
                                Ok(())
                            })
                            .await
                        {
                            log::error!("Failed to apply remote changes: {}", e);
                        }
                    },
                    // Handle other message types...
                    _ => {},
                }
            }
        });
        
        Ok(())
    }
}
```

## Integration Tests

Tests for MCP-Context integration should focus on correctness, consistency, and performance:

### 1. Basic Integration Testing

```rust
#[tokio::test]
async fn test_mcp_session_context_integration() {
    // Setup test components
    let context_manager = ContextManager::for_testing();
    let mcp_server = McpServer::for_testing().with_context_manager(&context_manager);
    
    // Create MCP session with new context
    let session = mcp_server.create_session(None).await.unwrap();
    let session_id = session.id();
    
    // Verify context was created and associated
    let context_id = session.context_id();
    let context = context_manager.get_context(context_id).await.unwrap();
    
    // Context should exist and be empty
    assert!(context.state().is_empty());
    
    // Execute a tool that modifies context
    let tool_request = ToolRequest::new("test.tool", json!({
        "action": "set_value",
        "key": "test_key",
        "value": "test_value"
    }));
    
    let response = session.process_tool_request(tool_request).await.unwrap();
    assert!(response.is_success());
    
    // Verify context was updated
    let updated_context = context_manager.get_context(context_id).await.unwrap();
    assert_eq!(
        updated_context.state().get::<String>("test_key").unwrap(),
        "test_value"
    );
    
    // Close session and verify context is preserved
    mcp_server.close_session(session_id).await.unwrap();
    
    // Context should still exist
    let context_after_close = context_manager.get_context(context_id).await.unwrap();
    assert_eq!(
        context_after_close.state().get::<String>("test_key").unwrap(),
        "test_value"
    );
}
```

### 2. Context Sharing Testing

```rust
#[tokio::test]
async fn test_context_sharing_between_sessions() {
    // Setup test environment
    let shared_context_mgr = SharedContextManager::for_testing();
    let mcp_server = McpServer::for_testing()
        .with_shared_context_manager(&shared_context_mgr);
    
    // Create first session with a context
    let session1 = mcp_server.create_session(None).await.unwrap();
    let session1_id = session1.id();
    
    // Add some data to the context
    let tool_request = ToolRequest::new("test.tool", json!({
        "action": "set_value",
        "key": "shared_key",
        "value": "shared_value"
    }));
    
    session1.process_tool_request(tool_request).await.unwrap();
    
    // Create second session
    let session2 = mcp_server.create_session(None).await.unwrap();
    let session2_id = session2.id();
    
    // Share context from session1 to session2
    shared_context_mgr.share_context(
        session1_id,
        session2_id,
        ContextPermissions::read_only()
    ).await.unwrap();
    
    // Verify session2 can read the shared value
    let read_request = ToolRequest::new("test.tool", json!({
        "action": "get_value",
        "key": "shared_key"
    }));
    
    let response = session2.process_tool_request(read_request).await.unwrap();
    let value = response.result()
        .get("value")
        .unwrap()
        .as_str()
        .unwrap();
    
    assert_eq!(value, "shared_value");
    
    // Verify session2 cannot modify the context (read-only permission)
    let write_request = ToolRequest::new("test.tool", json!({
        "action": "set_value",
        "key": "shared_key",
        "value": "modified_value"
    }));
    
    let error_response = session2.process_tool_request(write_request).await;
    assert!(error_response.is_err());
    
    // Close both sessions and verify context cleanup
    mcp_server.close_session(session1_id).await.unwrap();
    mcp_server.close_session(session2_id).await.unwrap();
}
```

### 3. Distributed Context Synchronization Testing

```rust
#[tokio::test]
async fn test_distributed_context_synchronization() {
    // Create two context managers representing different nodes
    let (node1_cm, node2_cm) = setup_distributed_test_environment().await;
    
    // Create a synchronizer for each node
    let sync_channel = MemorySyncChannel::new();
    
    let node1_sync = DistributedContextSynchronizer::new(
        Arc::clone(&node1_cm),
        sync_channel.clone()
    );
    
    let node2_sync = DistributedContextSynchronizer::new(
        Arc::clone(&node2_cm),
        sync_channel
    );
    
    // Create a context on node1
    let context_id = node1_cm.create_context().await.unwrap();
    
    // Update the context on node1
    node1_cm.update_context(context_id, |ctx| {
        ctx.set("key1", "value1")?;
        ctx.set("key2", 42)?;
        Ok(())
    }).await.unwrap();
    
    // Start synchronization on both nodes
    node1_sync.start_sync(context_id).await.unwrap();
    node2_sync.start_sync(context_id).await.unwrap();
    
    // Allow time for initial sync
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Verify node2 received the initial state
    let node2_context = node2_cm.get_context(context_id).await.unwrap();
    assert_eq!(node2_context.state().get::<String>("key1").unwrap(), "value1");
    assert_eq!(node2_context.state().get::<i32>("key2").unwrap(), 42);
    
    // Update context on node2
    node2_cm.update_context(context_id, |ctx| {
        ctx.set("key3", "value3")?;
        ctx.set("key2", 100)?; // Modify existing value
        Ok(())
    }).await.unwrap();
    
    // Allow time for sync to propagate
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Verify node1 received the updates from node2
    let node1_context = node1_cm.get_context(context_id).await.unwrap();
    assert_eq!(node1_context.state().get::<String>("key1").unwrap(), "value1");
    assert_eq!(node1_context.state().get::<i32>("key2").unwrap(), 100); // Updated value
    assert_eq!(node1_context.state().get::<String>("key3").unwrap(), "value3");
}
```

## Best Practices

### 1. MCP Session Management

- Associate each MCP session with a unique context or a shared context
- Implement proper context cleanup when sessions terminate
- Apply appropriate permission boundaries for context access
- Document context requirements for tools and commands

### 2. Context State Management

- Define clear ownership boundaries for context fields
- Implement conflict resolution strategies for concurrent updates
- Use proper serialization formats for context transport
- Validate context state after updates for consistency

### 3. Performance Considerations

- Minimize context size to reduce serialization overhead
- Implement context segmentation for large states
- Use copy-on-write for large shared contexts
- Apply batching for high-frequency context updates
- Implement appropriate caching strategies for context access

### 4. Security Best Practices

- Apply principle of least privilege for context access
- Audit all context modifications for security compliance
- Sanitize context data before storage or transport
- Encrypt sensitive context data during transmission
- Implement proper authentication for context access
- Apply rate limiting for context-modifying operations

## Implementation Across Components

The MCP-Context integration pattern has been successfully implemented in the following components:

1. **Context Management System**: Provides the core context functionality with async-aware locking and state management.
2. **MCP Protocol Handler**: Integrates with contexts for session management and tool execution.
3. **Tool Management System**: Enables context-aware tool execution and updates.

The pattern should be applied to additional components:

1. **Agent System**: Integrate context awareness for agent execution environments.
2. **UI Components**: Provide context-aware UI updates and state management.
3. **Distributed Execution**: Enable context synchronization across distributed nodes.
4. **Security System**: Implement context-based permission management.

## Version History

- 1.0.0 (2024-03-31): Initial version based on the completed context system implementation

<version>1.0.0</version> 