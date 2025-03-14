use std::collections::HashMap;
use std::error::Error;
use std::sync::RwLock;
use std::time::Instant;
use super::Command;
use super::validation::ValidationError;
use crate::commands::CommandValidator;
use super::lifecycle::{LifecycleHook, LifecycleStage};

/// Error type for hook failures.
#[derive(Debug)]
pub struct HookError {
    /// Error message describing the hook failure
    pub message: String,
}

impl std::fmt::Display for HookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Hook error: {}", self.message)
    }
}

impl Error for HookError {}

/// A trait for command hooks that can be executed during command lifecycle stages
#[allow(dead_code)]
pub trait Hook: Send + Sync {
    /// Returns the name of the hook.
    fn name(&self) -> &'static str;
    
    /// Returns the description of the hook.
    fn description(&self) -> &'static str;
    
    /// Executes the hook
    /// 
    /// # Arguments
    /// 
    /// * `command` - The command being executed
    /// 
    /// # Errors
    /// 
    /// Returns an error if the hook fails to execute
    fn execute(&self, command: &dyn Command) -> Result<(), Box<dyn Error>>;
}

/// Context for hook execution.
/// 
/// This struct contains metadata about the command and stage
/// being processed by a hook.
#[allow(dead_code)]
pub struct HookContext {
    /// Name of the command being processed
    command_name: String,
    /// Additional hook data
    data: RwLock<HashMap<String, String>>,
}

/// Type alias for a hook function that returns a Result
type HookFunction = Box<dyn Fn() -> Result<(), Box<dyn Error>>>;

/// Type alias for a map of hook names to their implementations
type HookMap = HashMap<String, HookFunction>;

/// A registry for managing command hooks
#[allow(dead_code)]
pub struct HookRegistry {
    /// Map of hook names to their implementations
    hooks: HookMap,
    /// Shared context for hooks to store and retrieve data
    context: RwLock<HashMap<String, String>>,
}

impl Default for HookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl HookRegistry {
    /// Creates a new hook registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            hooks: HashMap::new(),
            context: RwLock::new(HashMap::new()),
        }
    }

    /// Registers a new hook with the given name
    ///
    /// # Arguments
    /// * `name` - The name of the hook
    /// * `hook` - The hook function to register
    ///
    /// # Errors
    /// Returns an error if a hook with the given name already exists
    #[allow(dead_code)]
    pub fn register<F>(&mut self, name: String, hook: F) -> Result<(), Box<dyn Error>>
    where
        F: Fn() -> Result<(), Box<dyn Error>> + 'static,
    {
        if self.hooks.contains_key(&name) {
            return Err(Box::new(ValidationError {
                rule_name: "HookRegistry".to_string(),
                message: format!("Hook '{name}' already exists"),
            }));
        }

        self.hooks.insert(name, Box::new(hook));
        Ok(())
    }

    /// Executes all registered hooks
    ///
    /// # Errors
    /// Returns an error if any hook fails to execute
    #[allow(dead_code)]
    pub fn execute_hooks(&self) -> Result<(), Box<dyn Error>> {
        for (name, hook) in &self.hooks {
            if let Err(e) = hook() {
                return Err(Box::new(ValidationError {
                    rule_name: "HookRegistry".to_string(),
                    message: format!("Hook '{name}' failed: {e}"),
                }));
            }
        }
        Ok(())
    }

    /// Sets a value in the shared context
    /// 
    /// # Arguments
    /// * `key` - The key to set
    /// * `value` - The value to set
    /// 
    /// # Errors
    /// Returns an error if unable to acquire write lock on context
    #[allow(dead_code)]
    pub fn set_context_data(&self, key: &str, value: &str) -> Result<(), Box<dyn Error>> {
        let mut context = self.context.write().map_err(|_| {
            Box::new(HookError {
                message: "Failed to acquire write lock on context".to_string(),
            }) as Box<dyn Error>
        })?;
        context.insert(key.to_string(), value.to_string());
        Ok(())
    }

    /// Gets a value from the shared context
    /// 
    /// # Arguments
    /// * `key` - The key to get
    /// 
    /// # Returns
    /// * `Ok(Some(String))` if the value exists
    /// * `Ok(None)` if the value does not exist
    /// 
    /// # Errors
    /// Returns an error if unable to acquire read lock on context
    #[allow(dead_code)]
    pub fn get_context_data(&self, key: &str) -> Result<Option<String>, Box<dyn Error>> {
        let context = self.context.read().map_err(|_| {
            Box::new(HookError {
                message: "Failed to acquire read lock on context".to_string(),
            }) as Box<dyn Error>
        })?;
        Ok(context.get(key).cloned())
    }
}

/// Hook that logs command execution events with descriptive messages
#[allow(dead_code)]
pub struct LoggingHook {
    /// Name of the hook for identification
    name: String,
    /// Description of the hook's purpose
    description: String,
}

impl LoggingHook {
    /// Creates a new logging hook
    #[must_use]
    pub fn new() -> Self {
        Self {
            name: "logging".to_string(),
            description: "Logs command execution stages".to_string(),
        }
    }
}

impl Hook for LoggingHook {
    fn name(&self) -> &'static str {
        "logging"
    }

    fn description(&self) -> &'static str {
        "Logs command execution stages"
    }

    fn execute(&self, command: &dyn Command) -> Result<(), Box<dyn Error>> {
        println!(
            "Executing command '{}'",
            command.name()
        );
        Ok(())
    }
}

impl Default for LoggingHook {
    fn default() -> Self {
        Self::new()
    }
}

/// Hook that collects and records command execution metrics
#[allow(dead_code)]
pub struct MetricsHook {
    /// Name of the hook for identification
    name: String,
    /// Description of the hook's purpose
    description: String,
}

impl MetricsHook {
    /// Creates a new metrics hook
    #[must_use]
    pub fn new() -> Self {
        Self {
            name: "metrics".to_string(),
            description: "Collects command execution metrics".to_string(),
        }
    }
}

impl Hook for MetricsHook {
    fn name(&self) -> &'static str {
        "metrics"
    }

    fn description(&self) -> &'static str {
        "Collects command execution metrics"
    }

    fn execute(&self, command: &dyn Command) -> Result<(), Box<dyn Error>> {
        println!(
            "Metrics - Command: {}",
            command.name()
        );
        Ok(())
    }
}

impl Default for MetricsHook {
    fn default() -> Self {
        Self::new()
    }
}

/// Hook that measures and records command execution timing
#[allow(dead_code)]
pub struct TimingHook {
    /// Start time of the hook execution
    start_time: RwLock<Option<Instant>>,
}

impl TimingHook {
    /// Creates a new timing hook
    #[must_use]
    pub fn new() -> Self {
        Self {
            start_time: RwLock::new(None),
        }
    }
}

impl Hook for TimingHook {
    fn name(&self) -> &'static str {
        "timing"
    }

    fn description(&self) -> &'static str {
        "Measures command execution time"
    }

    fn execute(&self, command: &dyn Command) -> Result<(), Box<dyn Error>> {
        let mut start_time = self.start_time.write().map_err(|_| {
            Box::new(HookError {
                message: "Failed to acquire write lock on start time".to_string(),
            }) as Box<dyn Error>
        })?;

        match *start_time {
            None => {
                *start_time = Some(Instant::now());
            }
            Some(start) => {
                let duration = start.elapsed();
                println!("Command '{}' took {:?}", command.name(), duration);
                *start_time = None;
            }
        }

        Ok(())
    }
}

impl Default for TimingHook {
    fn default() -> Self {
        Self::new()
    }
}

/// Hook for validating command arguments
pub struct ArgumentValidationHook {
    /// Validator instance for checking command arguments
    validator: CommandValidator,
}

impl ArgumentValidationHook {
    /// Creates a new `ArgumentValidationHook`
    #[must_use]
    pub fn new() -> Self {
        Self {
            validator: CommandValidator::new(),
        }
    }
}

impl LifecycleHook for ArgumentValidationHook {
    fn pre_stage(&self, stage: &LifecycleStage, command: &dyn Command) -> Result<(), Box<dyn Error>> {
        if *stage == LifecycleStage::Validation {
            self.validator.validate(command)?;
        }
        Ok(())
    }

    fn post_stage(&self, _stage: &LifecycleStage, _command: &dyn Command) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

impl Default for ArgumentValidationHook {
    fn default() -> Self {
        Self::new()
    }
}

/// Hook for validating environment requirements
pub struct EnvironmentValidationHook {
    /// Validator instance for checking environment requirements
    validator: CommandValidator,
}

impl EnvironmentValidationHook {
    /// Creates a new `EnvironmentValidationHook`
    #[must_use]
    pub fn new() -> Self {
        Self {
            validator: CommandValidator::new(),
        }
    }
}

impl LifecycleHook for EnvironmentValidationHook {
    fn pre_stage(&self, stage: &LifecycleStage, command: &dyn Command) -> Result<(), Box<dyn Error>> {
        if *stage == LifecycleStage::Validation {
            self.validator.validate(command)?;
        }
        Ok(())
    }

    fn post_stage(&self, _stage: &LifecycleStage, _command: &dyn Command) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

impl Default for EnvironmentValidationHook {
    fn default() -> Self {
        Self::new()
    }
}

/// Hook for validating resource requirements
pub struct ResourceValidationHook {
    /// Validator instance for checking resource requirements
    validator: CommandValidator,
}

impl ResourceValidationHook {
    /// Creates a new `ResourceValidationHook`
    #[must_use]
    pub fn new() -> Self {
        Self {
            validator: CommandValidator::new(),
        }
    }
}

impl LifecycleHook for ResourceValidationHook {
    fn pre_stage(&self, stage: &LifecycleStage, command: &dyn Command) -> Result<(), Box<dyn Error>> {
        if *stage == LifecycleStage::Validation {
            self.validator.validate(command)?;
        }
        Ok(())
    }

    fn post_stage(&self, _stage: &LifecycleStage, _command: &dyn Command) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

impl Default for ResourceValidationHook {
    fn default() -> Self {
        Self::new()
    }
}

/// A manager for command hooks
pub struct HookManager {
    /// Map of hook names to their implementations
    #[allow(dead_code)]
    hooks: HookMap,
}

impl HookManager {
    /// Creates a new hook manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            hooks: HashMap::new(),
        }
    }

    /// Adds a hook to the manager
    /// 
    /// # Arguments
    /// * `name` - Name of the hook
    /// * `hook` - The hook implementation
    /// 
    /// # Errors
    /// Returns an error if a hook with the same name already exists
    #[allow(dead_code)]
    pub fn add_hook(
        &mut self,
        name: &str,
        hook: Box<dyn Fn() -> Result<(), Box<dyn Error>>>,
    ) -> Result<(), Box<dyn Error>> {
        if self.hooks.contains_key(name) {
            return Err(Box::new(HookError {
                message: format!("Hook '{name}' already exists"),
            }));
        }
        self.hooks.insert(name.to_string(), hook);
        Ok(())
    }

    /// Executes all registered hooks
    /// 
    /// # Errors
    /// Returns an error if any hook fails to execute
    #[allow(dead_code)]
    pub fn execute_hooks(&self) -> Result<(), Box<dyn Error>> {
        for (name, hook) in &self.hooks {
            hook().map_err(|e| {
                Box::new(HookError {
                    message: format!("Hook '{name}' failed: {e}"),
                }) as Box<dyn Error>
            })?;
        }
        Ok(())
    }
}

impl Default for HookManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{Parser, CommandFactory};

    #[derive(Parser)]
    #[command(name = "test")]
    #[allow(dead_code)]
    struct TestArgs {
        #[arg(short, long)]
        value: String,
    }

    #[derive(Clone)]
    #[allow(dead_code)]
    struct TestCommand;

    impl Command for TestCommand {
        fn name(&self) -> &str {
            "test"
        }

        fn description(&self) -> &str {
            "A test command"
        }

        fn execute(&self) -> Result<(), Box<dyn Error>> {
            Ok(())
        }

        fn parser(&self) -> clap::Command {
            TestArgs::command()
        }

        fn clone_box(&self) -> Box<dyn Command> {
            Box::new(self.clone())
        }
    }

    #[test]
    fn test_hook_execution() {
        let mut manager = HookManager::new();
        let hook = || -> Result<(), Box<dyn Error>> { Ok(()) };
        manager.add_hook("test_hook", Box::new(hook)).unwrap();
        manager.execute_hooks().unwrap();
    }

    #[test]
    fn test_hook_error_handling() {
        let mut manager = HookManager::new();
        let hook = || -> Result<(), Box<dyn Error>> {
            Err("Hook error".into())
        };
        manager.add_hook("error_hook", Box::new(hook)).unwrap();
        assert!(manager.execute_hooks().is_err());
    }
} 