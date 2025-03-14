use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use uuid::Uuid;
use super::{ContextState, ContextError, ContextSnapshot};
use super::persistence::ContextPersistence;

pub trait RecoveryStrategy {
    fn select_state<'a>(&self, snapshots: &'a [ContextSnapshot]) -> Option<&'a ContextSnapshot>;
}

pub struct LatestVersionStrategy;

impl LatestVersionStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LatestVersionStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl RecoveryStrategy for LatestVersionStrategy {
    fn select_state<'a>(&self, snapshots: &'a [ContextSnapshot]) -> Option<&'a ContextSnapshot> {
        snapshots.iter().max_by_key(move |s| s.state.version)
    }
}

pub struct SpecificVersionStrategy {
    version: u64,
}

impl SpecificVersionStrategy {
    pub fn new(version: u64) -> Self {
        Self { version }
    }
}

impl RecoveryStrategy for SpecificVersionStrategy {
    fn select_state<'a>(&self, snapshots: &'a [ContextSnapshot]) -> Option<&'a ContextSnapshot> {
        snapshots.iter().find(move |s| s.state.version == self.version)
    }
}

pub struct TimeBasedStrategy {
    timestamp: SystemTime,
}

impl TimeBasedStrategy {
    pub fn new(timestamp: SystemTime) -> Self {
        Self { timestamp }
    }
}

impl RecoveryStrategy for TimeBasedStrategy {
    fn select_state<'a>(&self, snapshots: &'a [ContextSnapshot]) -> Option<&'a ContextSnapshot> {
        snapshots
            .iter()
            .filter(move |s| s.timestamp <= self.timestamp)
            .max_by_key(move |s| s.timestamp)
    }
}

pub struct RecoveryManager {
    persistence: Arc<Mutex<ContextPersistence>>,
    snapshots: VecDeque<ContextSnapshot>,
    max_snapshots: usize,
}

impl RecoveryManager {
    pub fn new(
        persistence: Arc<Mutex<ContextPersistence>>,
        max_snapshots: usize,
    ) -> Self {
        Self {
            persistence,
            snapshots: VecDeque::new(),
            max_snapshots,
        }
    }

    pub fn create_snapshot(&mut self, state: &ContextState) -> Result<ContextSnapshot, ContextError> {
        let snapshot = ContextSnapshot {
            id: Uuid::new_v4().to_string(),
            timestamp: SystemTime::now(),
            state: state.clone(),
            metadata: None,
        };

        if let Ok(mut persistence) = self.persistence.lock() {
            persistence.save_snapshot(&snapshot)?;
        }

        self.snapshots.push_back(snapshot.clone());
        if self.snapshots.len() > self.max_snapshots {
            if let Some(old_snapshot) = self.snapshots.pop_front() {
                if let Ok(mut persistence) = self.persistence.lock() {
                    let _ = persistence.delete_snapshot(&old_snapshot.id);
                }
            }
        }

        Ok(snapshot)
    }

    pub fn get_snapshots(&self) -> &VecDeque<ContextSnapshot> {
        &self.snapshots
    }

    pub fn restore_snapshot(&self, id: &str) -> Result<ContextState, ContextError> {
        if let Some(snapshot) = self.snapshots.iter().find(|s| s.id == id) {
            Ok(snapshot.state.clone())
        } else {
            Err(ContextError::SnapshotNotFound)
        }
    }

    pub fn delete_snapshot(&mut self, id: &str) -> Result<(), ContextError> {
        if let Some(index) = self.snapshots.iter().position(|s| s.id == id) {
            self.snapshots.remove(index);
            if let Ok(mut persistence) = self.persistence.lock() {
                persistence.delete_snapshot(id)?;
            }
            Ok(())
        } else {
            Err(ContextError::SnapshotNotFound)
        }
    }

    pub fn recover_using_strategy(&self, strategy: &dyn RecoveryStrategy) -> Result<ContextState, ContextError> {
        let snapshots: Vec<_> = self.snapshots.iter().cloned().collect();
        if let Some(snapshot) = strategy.select_state(&snapshots) {
            Ok(snapshot.state.clone())
        } else {
            Err(ContextError::NoValidSnapshot)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tempfile::tempdir;
    use crate::core::{ContextPersistence, FileStorage, JsonSerializer};

    #[test]
    fn test_recovery_manager() {
        let temp_dir = tempdir().unwrap();
        let storage = Box::new(FileStorage::new(temp_dir.path().to_path_buf()).unwrap());
        let serializer = Box::new(JsonSerializer::new());
        let persistence = Arc::new(Mutex::new(ContextPersistence::new(
            storage,
            serializer,
            10,
            Duration::from_secs(60),
        )));

        let mut recovery = RecoveryManager::new(persistence.clone(), 10);

        // Create test state
        let state = ContextState {
            version: 1,
            data: serde_json::json!({"test": "value"}),
            last_modified: SystemTime::now(),
        };

        // Test snapshot creation
        let snapshot = recovery.create_snapshot(&state).unwrap();
        assert!(!snapshot.id.is_empty());

        // Test snapshot listing
        let snapshots = recovery.get_snapshots();
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].state.version, state.version);

        // Test snapshot restoration
        let restored_state = recovery.restore_snapshot(&snapshot.id).unwrap();
        assert_eq!(restored_state.version, state.version);
        assert_eq!(restored_state.data, state.data);

        // Test snapshot deletion
        assert!(recovery.delete_snapshot(&snapshot.id).is_ok());
        let snapshots = recovery.get_snapshots();
        assert!(snapshots.is_empty());
    }

    #[test]
    fn test_recovery_strategy() {
        let temp_dir = tempdir().unwrap();
        let storage = Box::new(FileStorage::new(temp_dir.path().to_path_buf()).unwrap());
        let serializer = Box::new(JsonSerializer::new());
        let persistence = Arc::new(Mutex::new(ContextPersistence::new(
            storage,
            serializer,
            10,
            Duration::from_secs(60),
        )));

        let mut recovery = RecoveryManager::new(persistence.clone(), 10);

        // Create test states
        let states: Vec<ContextState> = (1..=3)
            .map(|i| ContextState {
                version: i,
                data: serde_json::json!({ "value": i }),
                last_modified: SystemTime::now(),
            })
            .collect();

        // Create snapshots
        let mut snapshot_ids = Vec::new();
        for state in &states {
            let snapshot = recovery.create_snapshot(state).unwrap();
            snapshot_ids.push(snapshot.id);
        }

        // Test latest strategy
        let strategy = LatestVersionStrategy::new();
        let recovered = recovery.recover_using_strategy(&strategy).unwrap();
        assert_eq!(recovered.version, 3);

        // Test specific version strategy
        let strategy = SpecificVersionStrategy::new(2);
        let recovered = recovery.recover_using_strategy(&strategy).unwrap();
        assert_eq!(recovered.version, 2);

        // Test time-based strategy
        std::thread::sleep(Duration::from_millis(100));
        let strategy = TimeBasedStrategy::new(SystemTime::now());
        let recovered = recovery.recover_using_strategy(&strategy).unwrap();
        assert!(recovered.version > 0);

        // Test cleanup
        for id in snapshot_ids {
            assert!(recovery.delete_snapshot(&id).is_ok());
        }
    }

    #[test]
    fn test_recovery_error_handling() {
        let temp_dir = tempdir().unwrap();
        let storage = Box::new(FileStorage::new(temp_dir.path().to_path_buf()).unwrap());
        let serializer = Box::new(JsonSerializer::new());
        let persistence = Arc::new(Mutex::new(ContextPersistence::new(
            storage,
            serializer,
            10,
            Duration::from_secs(60),
        )));

        let mut recovery = RecoveryManager::new(persistence.clone(), 10);

        // Test restoring non-existent snapshot
        assert!(matches!(
            recovery.restore_snapshot("non_existent"),
            Err(ContextError::SnapshotNotFound)
        ));

        // Test deleting non-existent snapshot
        assert!(matches!(
            recovery.delete_snapshot("non_existent"),
            Err(ContextError::SnapshotNotFound)
        ));

        // Test recovery with no snapshots
        let strategy = LatestVersionStrategy::new();
        assert!(matches!(
            recovery.recover_using_strategy(&strategy),
            Err(ContextError::NoValidSnapshot)
        ));
    }
} 