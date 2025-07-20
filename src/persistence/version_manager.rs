use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::persistence::serialization::{SerializationResult, SerializationError, SaveData};

/// Save file version information
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SaveVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub pre_release: Option<String>,
}

impl SaveVersion {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        SaveVersion {
            major,
            minor,
            patch,
            pre_release: None,
        }
    }

    pub fn with_pre_release(mut self, pre_release: String) -> Self {
        self.pre_release = Some(pre_release);
        self
    }

    pub fn from_string(version_str: &str) -> Result<Self, String> {
        let parts: Vec<&str> = version_str.split('.').collect();
        
        if parts.len() < 3 {
            return Err(format!("Invalid version format: {}", version_str));
        }

        let major = parts[0].parse::<u32>()
            .map_err(|_| format!("Invalid major version: {}", parts[0]))?;
        
        let minor = parts[1].parse::<u32>()
            .map_err(|_| format!("Invalid minor version: {}", parts[1]))?;
        
        // Handle patch version with potential pre-release suffix
        let patch_part = parts[2];
        let (patch_str, pre_release) = if let Some(dash_pos) = patch_part.find('-') {
            let (patch, pre) = patch_part.split_at(dash_pos);
            (patch, Some(pre[1..].to_string())) // Skip the dash
        } else {
            (patch_part, None)
        };

        let patch = patch_str.parse::<u32>()
            .map_err(|_| format!("Invalid patch version: {}", patch_str))?;

        Ok(SaveVersion {
            major,
            minor,
            patch,
            pre_release,
        })
    }

    pub fn to_string(&self) -> String {
        if let Some(ref pre) = self.pre_release {
            format!("{}.{}.{}-{}", self.major, self.minor, self.patch, pre)
        } else {
            format!("{}.{}.{}", self.major, self.minor, self.patch)
        }
    }

    pub fn is_compatible_with(&self, other: &SaveVersion) -> VersionCompatibility {
        // Major version differences are incompatible
        if self.major != other.major {
            return VersionCompatibility::Incompatible;
        }

        // Minor version differences might need migration
        if self.minor != other.minor {
            if self.minor > other.minor {
                return VersionCompatibility::NeedsMigration;
            } else {
                return VersionCompatibility::TooNew;
            }
        }

        // Patch differences are usually compatible
        if self.patch != other.patch {
            return VersionCompatibility::Compatible;
        }

        // Same version
        VersionCompatibility::Exact
    }
}

/// Version compatibility status
#[derive(Debug, Clone, PartialEq)]
pub enum VersionCompatibility {
    Exact,           // Same version
    Compatible,      // Compatible, no migration needed
    NeedsMigration,  // Compatible but needs migration
    TooNew,          // Save is from newer version
    Incompatible,    // Incompatible versions
}

/// Migration result
#[derive(Debug, Clone)]
pub enum MigrationResult {
    Success(SaveData),
    Failed(String),
    NotNeeded,
}

/// Version manager for handling save compatibility and migrations
pub struct VersionManager {
    current_version: SaveVersion,
    migrations: HashMap<String, Box<dyn SaveMigration>>,
    compatibility_rules: HashMap<String, VersionCompatibility>,
}

/// Trait for save file migrations
pub trait SaveMigration: Send + Sync {
    fn migrate(&self, save_data: SaveData) -> SerializationResult<SaveData>;
    fn from_version(&self) -> SaveVersion;
    fn to_version(&self) -> SaveVersion;
    fn description(&self) -> &str;
}

impl VersionManager {
    pub fn new(current_version: SaveVersion) -> Self {
        VersionManager {
            current_version,
            migrations: HashMap::new(),
            compatibility_rules: HashMap::new(),
        }
    }

    /// Register a migration
    pub fn register_migration(&mut self, migration: Box<dyn SaveMigration>) {
        let key = format!("{}->{}", 
            migration.from_version().to_string(), 
            migration.to_version().to_string()
        );
        self.migrations.insert(key, migration);
    }

    /// Set compatibility rule for a specific version
    pub fn set_compatibility_rule(&mut self, version: SaveVersion, compatibility: VersionCompatibility) {
        self.compatibility_rules.insert(version.to_string(), compatibility);
    }

    /// Check if a save version is compatible
    pub fn check_compatibility(&self, save_version: &SaveVersion) -> VersionCompatibility {
        // Check explicit compatibility rules first
        if let Some(compatibility) = self.compatibility_rules.get(&save_version.to_string()) {
            return compatibility.clone();
        }

        // Use default compatibility logic
        self.current_version.is_compatible_with(save_version)
    }

    /// Migrate save data to current version
    pub fn migrate_save(&self, mut save_data: SaveData) -> SerializationResult<MigrationResult> {
        let save_version = SaveVersion::from_string(&save_data.version)
            .map_err(|e| SerializationError::InvalidFormat(e))?;

        let compatibility = self.check_compatibility(&save_version);

        match compatibility {
            VersionCompatibility::Exact | VersionCompatibility::Compatible => {
                return Ok(MigrationResult::NotNeeded);
            }
            VersionCompatibility::TooNew => {
                return Err(SerializationError::VersionMismatch {
                    expected: self.current_version.to_string(),
                    found: save_version.to_string(),
                });
            }
            VersionCompatibility::Incompatible => {
                return Err(SerializationError::VersionMismatch {
                    expected: self.current_version.to_string(),
                    found: save_version.to_string(),
                });
            }
            VersionCompatibility::NeedsMigration => {
                // Continue with migration
            }
        }

        // Find migration path
        let migration_path = self.find_migration_path(&save_version, &self.current_version)?;

        // Apply migrations in sequence
        for migration_key in migration_path {
            if let Some(migration) = self.migrations.get(&migration_key) {
                save_data = migration.migrate(save_data)
                    .map_err(|e| SerializationError::DeserializationFailed(
                        format!("Migration failed: {}", e)
                    ))?;
            } else {
                return Err(SerializationError::DeserializationFailed(
                    format!("Migration not found: {}", migration_key)
                ));
            }
        }

        // Update version in save data
        save_data.version = self.current_version.to_string();

        Ok(MigrationResult::Success(save_data))
    }

    /// Find migration path from one version to another
    fn find_migration_path(&self, from: &SaveVersion, to: &SaveVersion) -> SerializationResult<Vec<String>> {
        // This is a simplified implementation
        // In practice, you might need a more sophisticated pathfinding algorithm
        
        let mut path = Vec::new();
        let mut current = from.clone();

        // For now, we'll only handle direct migrations
        while current != *to {
            let next_version = self.find_next_migration_step(&current, to)?;
            let migration_key = format!("{}->{}", current.to_string(), next_version.to_string());
            
            if self.migrations.contains_key(&migration_key) {
                path.push(migration_key);
                current = next_version;
            } else {
                return Err(SerializationError::DeserializationFailed(
                    format!("No migration path found from {} to {}", from.to_string(), to.to_string())
                ));
            }
        }

        Ok(path)
    }

    fn find_next_migration_step(&self, from: &SaveVersion, to: &SaveVersion) -> SerializationResult<SaveVersion> {
        // Simple strategy: increment minor version if different, otherwise patch
        if from.minor < to.minor {
            Ok(SaveVersion::new(from.major, from.minor + 1, 0))
        } else if from.patch < to.patch {
            Ok(SaveVersion::new(from.major, from.minor, from.patch + 1))
        } else {
            Err(SerializationError::DeserializationFailed(
                "Cannot determine next migration step".to_string()
            ))
        }
    }

    /// Get current version
    pub fn current_version(&self) -> &SaveVersion {
        &self.current_version
    }

    /// Get available migrations
    pub fn get_available_migrations(&self) -> Vec<String> {
        self.migrations.keys().cloned().collect()
    }

    /// Validate save data version
    pub fn validate_save_version(&self, save_data: &SaveData) -> SerializationResult<()> {
        let save_version = SaveVersion::from_string(&save_data.version)
            .map_err(|e| SerializationError::InvalidFormat(e))?;

        let compatibility = self.check_compatibility(&save_version);

        match compatibility {
            VersionCompatibility::Incompatible => {
                Err(SerializationError::VersionMismatch {
                    expected: self.current_version.to_string(),
                    found: save_version.to_string(),
                })
            }
            VersionCompatibility::TooNew => {
                Err(SerializationError::VersionMismatch {
                    expected: format!("<= {}", self.current_version.to_string()),
                    found: save_version.to_string(),
                })
            }
            _ => Ok(()),
        }
    }
}

/// Example migration implementations
pub struct Migration_0_1_0_to_0_2_0;

impl SaveMigration for Migration_0_1_0_to_0_2_0 {
    fn migrate(&self, mut save_data: SaveData) -> SerializationResult<SaveData> {
        // Example migration: add new metadata field
        save_data.metadata.insert("migration_applied".to_string(), "0.1.0->0.2.0".to_string());
        
        // Example: modify component data structure
        for component in &mut save_data.components {
            if component.component_name == "CombatStats" {
                // Migrate old combat stats format to new format
                // This would involve deserializing, modifying, and re-serializing
            }
        }

        Ok(save_data)
    }

    fn from_version(&self) -> SaveVersion {
        SaveVersion::new(0, 1, 0)
    }

    fn to_version(&self) -> SaveVersion {
        SaveVersion::new(0, 2, 0)
    }

    fn description(&self) -> &str {
        "Migrate from 0.1.0 to 0.2.0: Add new combat stats fields"
    }
}

pub struct Migration_0_2_0_to_0_3_0;

impl SaveMigration for Migration_0_2_0_to_0_3_0 {
    fn migrate(&self, mut save_data: SaveData) -> SerializationResult<SaveData> {
        // Example migration: update inventory system
        save_data.metadata.insert("inventory_system_updated".to_string(), "true".to_string());
        
        // Remove deprecated components
        save_data.components.retain(|c| c.component_name != "OldInventory");
        
        Ok(save_data)
    }

    fn from_version(&self) -> SaveVersion {
        SaveVersion::new(0, 2, 0)
    }

    fn to_version(&self) -> SaveVersion {
        SaveVersion::new(0, 3, 0)
    }

    fn description(&self) -> &str {
        "Migrate from 0.2.0 to 0.3.0: Update inventory system"
    }
}

/// Helper function to create a version manager with common migrations
pub fn create_version_manager() -> VersionManager {
    let current_version = SaveVersion::from_string(env!("CARGO_PKG_VERSION"))
        .unwrap_or_else(|_| SaveVersion::new(0, 1, 0));

    let mut manager = VersionManager::new(current_version);

    // Register migrations
    manager.register_migration(Box::new(Migration_0_1_0_to_0_2_0));
    manager.register_migration(Box::new(Migration_0_2_0_to_0_3_0));

    // Set compatibility rules
    manager.set_compatibility_rule(
        SaveVersion::new(0, 1, 0),
        VersionCompatibility::NeedsMigration
    );
    manager.set_compatibility_rule(
        SaveVersion::new(0, 2, 0),
        VersionCompatibility::NeedsMigration
    );

    manager
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_version_creation() {
        let version = SaveVersion::new(1, 2, 3);
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
        assert_eq!(version.pre_release, None);
    }

    #[test]
    fn test_save_version_with_pre_release() {
        let version = SaveVersion::new(1, 0, 0).with_pre_release("alpha".to_string());
        assert_eq!(version.pre_release, Some("alpha".to_string()));
        assert_eq!(version.to_string(), "1.0.0-alpha");
    }

    #[test]
    fn test_save_version_from_string() {
        let version = SaveVersion::from_string("1.2.3").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);

        let version_pre = SaveVersion::from_string("2.0.0-beta").unwrap();
        assert_eq!(version_pre.major, 2);
        assert_eq!(version_pre.minor, 0);
        assert_eq!(version_pre.patch, 0);
        assert_eq!(version_pre.pre_release, Some("beta".to_string()));
    }

    #[test]
    fn test_save_version_invalid_format() {
        assert!(SaveVersion::from_string("1.2").is_err());
        assert!(SaveVersion::from_string("invalid").is_err());
        assert!(SaveVersion::from_string("1.2.x").is_err());
    }

    #[test]
    fn test_version_compatibility() {
        let v1_0_0 = SaveVersion::new(1, 0, 0);
        let v1_0_1 = SaveVersion::new(1, 0, 1);
        let v1_1_0 = SaveVersion::new(1, 1, 0);
        let v2_0_0 = SaveVersion::new(2, 0, 0);

        // Same version
        assert_eq!(v1_0_0.is_compatible_with(&v1_0_0), VersionCompatibility::Exact);

        // Patch difference
        assert_eq!(v1_0_1.is_compatible_with(&v1_0_0), VersionCompatibility::Compatible);

        // Minor version difference
        assert_eq!(v1_1_0.is_compatible_with(&v1_0_0), VersionCompatibility::NeedsMigration);
        assert_eq!(v1_0_0.is_compatible_with(&v1_1_0), VersionCompatibility::TooNew);

        // Major version difference
        assert_eq!(v2_0_0.is_compatible_with(&v1_0_0), VersionCompatibility::Incompatible);
    }

    #[test]
    fn test_version_manager_creation() {
        let version = SaveVersion::new(1, 0, 0);
        let manager = VersionManager::new(version.clone());
        
        assert_eq!(manager.current_version(), &version);
        assert!(manager.get_available_migrations().is_empty());
    }

    #[test]
    fn test_migration_registration() {
        let mut manager = VersionManager::new(SaveVersion::new(0, 2, 0));
        manager.register_migration(Box::new(Migration_0_1_0_to_0_2_0));
        
        let migrations = manager.get_available_migrations();
        assert_eq!(migrations.len(), 1);
        assert!(migrations.contains(&"0.1.0->0.2.0".to_string()));
    }

    #[test]
    fn test_compatibility_check() {
        let mut manager = VersionManager::new(SaveVersion::new(1, 0, 0));
        
        // Default compatibility
        let old_version = SaveVersion::new(0, 9, 0);
        assert_eq!(manager.check_compatibility(&old_version), VersionCompatibility::Incompatible);
        
        // Custom compatibility rule
        manager.set_compatibility_rule(old_version.clone(), VersionCompatibility::NeedsMigration);
        assert_eq!(manager.check_compatibility(&old_version), VersionCompatibility::NeedsMigration);
    }

    #[test]
    fn test_save_validation() {
        let manager = VersionManager::new(SaveVersion::new(1, 0, 0));
        
        // Valid save
        let valid_save = SaveData::new("Test".to_string(), "Player".to_string());
        assert!(manager.validate_save_version(&valid_save).is_ok());
        
        // Invalid save (too new)
        let mut invalid_save = SaveData::new("Test".to_string(), "Player".to_string());
        invalid_save.version = "2.0.0".to_string();
        assert!(manager.validate_save_version(&invalid_save).is_err());
    }

    #[test]
    fn test_migration_execution() {
        let migration = Migration_0_1_0_to_0_2_0;
        let save_data = SaveData::new("Test".to_string(), "Player".to_string());
        
        let migrated = migration.migrate(save_data).unwrap();
        assert!(migrated.metadata.contains_key("migration_applied"));
        assert_eq!(migrated.metadata.get("migration_applied"), Some(&"0.1.0->0.2.0".to_string()));
    }

    #[test]
    fn test_create_version_manager() {
        let manager = create_version_manager();
        
        assert!(!manager.get_available_migrations().is_empty());
        
        // Should have registered migrations
        let migrations = manager.get_available_migrations();
        assert!(migrations.iter().any(|m| m.contains("0.1.0->0.2.0")));
        assert!(migrations.iter().any(|m| m.contains("0.2.0->0.3.0")));
    }

    #[test]
    fn test_version_ordering() {
        let v1 = SaveVersion::new(1, 0, 0);
        let v2 = SaveVersion::new(1, 0, 1);
        let v3 = SaveVersion::new(1, 1, 0);
        let v4 = SaveVersion::new(2, 0, 0);

        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v3 < v4);
    }
}