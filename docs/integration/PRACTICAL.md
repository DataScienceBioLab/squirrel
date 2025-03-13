# Practical Integration Guide

## Quick Start

### 1. Setting Up Your Environment
```bash
# Clone and setup
git clone https://github.com/yourusername/groundhog-mcp.git
cd groundhog-mcp
cargo build

# Run integration tests
cargo test --test '*_integration'
```

### 2. Common Integration Tasks

#### Adding a New MCP Command
```rust
// 1. Define command in protocol
pub struct NewCommand {
    pub parameter: String,
}

// 2. Implement handler
impl CommandHandler for NewCommand {
    async fn handle(&self, ctx: &Context) -> Result<Response> {
        // Implementation
    }
}

// 3. Register in MCP
mcp.register_command("new_command", NewCommand::handle);
```

#### Connecting UI to MCP
```rust
// 1. Create UI event
#[derive(Debug, Serialize)]
pub struct UIEvent {
    pub event_type: String,
    pub payload: Value,
}

// 2. Connect to MCP
ui.on_event(|event| {
    mcp.dispatch(event).await?;
});
```

## Common Patterns

### 1. State Management
```rust
// Shared state pattern
pub struct SharedState {
    inner: Arc<RwLock<State>>,
}

impl SharedState {
    pub async fn update(&self, f: impl FnOnce(&mut State)) -> Result<()> {
        let mut state = self.inner.write().await;
        f(&mut state);
        Ok(())
    }
}
```

### 2. Error Handling
```rust
// Common error types
#[derive(Debug, Error)]
pub enum IntegrationError {
    #[error("Component communication failed: {0}")]
    CommunicationError(String),
    
    #[error("State synchronization failed: {0}")]
    SyncError(String),
}

// Error handling pattern
async fn handle_with_retry<F, T>(f: F) -> Result<T>
where
    F: Fn() -> Future<Output = Result<T>>,
{
    retry(ExponentialBackoff::default(), || async {
        f().await
    }).await
}
```

## Testing Patterns

### 1. Component Integration Tests
```rust
#[tokio::test]
async fn test_component_integration() {
    // 1. Setup test environment
    let ctx = TestContext::new().await;
    
    // 2. Run integration scenario
    let result = ctx.run_scenario(TestScenario::new()
        .with_command(Command::new())
        .with_state_change(StateChange::new())
        .with_ui_event(UIEvent::new()))
        .await?;
    
    // 3. Verify results
    assert_integration_result(result);
}
```

### 2. Performance Testing
```rust
#[bench]
fn bench_integration_flow(b: &mut Bencher) {
    b.iter(|| {
        // Setup minimal test environment
        let ctx = MinimalContext::new();
        
        // Run core integration flow
        ctx.run_core_flow()
    });
}
```

## Debugging Tips

### 1. Logging Integration Points
```rust
// Add these log points in your integration code
tracing::info!("MCP command received: {:?}", cmd);
tracing::debug!("State update: {:?}", state_diff);
tracing::warn!("Retrying operation: attempt {}", attempt);
```

### 2. Common Issues and Solutions

#### State Desync
```rust
// Check state consistency
async fn verify_state_sync() -> Result<()> {
    let ui_state = ui.get_state().await?;
    let mcp_state = mcp.get_state().await?;
    
    assert_eq!(ui_state.version, mcp_state.version);
    Ok(())
}
```

#### Performance Issues
```rust
// Add performance tracing
#[tracing::instrument(skip(input))]
async fn measure_operation(input: &Input) -> Result<Output> {
    let timer = Instant::now();
    let result = perform_operation(input).await?;
    
    tracing::info!(
        "Operation completed in {:?}ms",
        timer.elapsed().as_millis()
    );
    
    Ok(result)
}
```

## Daily Workflow

### 1. Before Starting
- Pull latest changes
- Run integration tests
- Check performance metrics
- Review error logs

### 2. During Development
- Use integration test fixtures
- Monitor resource usage
- Keep error handling consistent
- Document API changes

### 3. Before Committing
- Run full test suite
- Check performance impact
- Update documentation
- Review security implications

## Quick Reference

### Common Commands
```bash
# Run specific integration tests
cargo test --test ui_integration
cargo test --test mcp_integration

# Check performance
cargo bench --bench integration_bench

# Generate documentation
cargo doc --document-private-items
```

### Useful Tools
- Integration Test Runner: `cargo test --test '*_integration'`
- Performance Monitor: `cargo bench`
- State Inspector: `cargo run --bin state-inspector`
- Log Analyzer: `cargo run --bin log-analyzer`

## Need Help?

### Common Resources
- Integration Test Examples: `tests/integration/`
- Performance Benchmarks: `benches/`
- Documentation: `docs/integration/`
- Error Handling Guide: `docs/errors.md`

### Contact Points
- Integration Issues: Create issue with `[Integration]` prefix
- Performance Problems: Use `[Perf]` tag in issues
- Security Concerns: Direct message security team

Remember: Integration is an iterative process. Start small, test thoroughly, and scale gradually. 