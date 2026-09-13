use crate::identifiers::DuUuid;
use serde_json::Value;
use uuid::Uuid;
use std::path::Path;
use std::fs;
use crate::identifiers::UreIdError;

pub struct ActuatorRegistry {
    // In a real implementation, this would hold a database connection pool (e.g., sqlx::PgPool)
    db_connection_string: String,
}

impl ActuatorRegistry {
    pub fn new(conn_str: &str) -> Self {
        Self {
            db_connection_string: conn_str.to_string(),
        }
    }

    /// Registers a .ure file into the Actuator Mesh.
    /// This is the core pipeline: .ure -> DU-UUID -> Database
    pub fn register_ure_file<P: AsRef<Path>>(&self, path: P, encryption_key: Option<&[u8; 32]>) -> Result<Uuid, Box<dyn std::error::Error>> {
        // 1. Read the .ure file (which is JSON)
        let content_str = fs::read_to_string(path)?;
        let manifest: Value = serde_json::from_str(&content_str)?;

        // 2. Generate the Deterministic UUID (DU-UUID)
        let resource_id = DuUuid::generate(&manifest, encryption_key)?;

        // 3. Extract metadata for database insertion
        let complexity_score = manifest["complexity_score"].as_f64().unwrap_or(0.5);
        let resource_type = manifest["resource_type"].as_str().unwrap_or("identity");
        let category = manifest["category"].as_str().unwrap_or("L");
        let location_flag = manifest["location_flag"].as_str().unwrap_or("L");
        
        // 4. Database Insertion (Mocked for this implementation phase)
        println!("Registering Actuator to DB: {} | ID: {}", resource_type, resource_id);
        println!("Complexity: {} | Category: {}", complexity_score, category);
        
        // In real code: 
        // sqlx::query!("INSERT INTO actuators ...").execute(&self.pool).await?;

        Ok(resource_id)
    }

    /// Resolves a DU-UUID back to its manifest (if available locally)
    pub fn resolve_actuator(&self, id: Uuid) -> Result<Value, Box<dyn std::error::Error>> {
        // In a real system, this would search the filesystem for <id>.ure
        // or fetch the compressed blob from the DB.
        let path = format!("{}.ure", id);
        let content = fs::read_to_string(path)?;
        let manifest: Value = serde_json::from_str(&content)?;
        Ok(manifest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_registration_pipeline() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_actuator.ure");
        
        let manifest = json!({
            "resource_type": "skill",
            "complexity_score": 0.3,
            "category": "L",
            "location_flag": "L"
        });
        
        let mut file = File::create(&file_path).unwrap();
        file.write_all(serde_json::to_string(&manifest).unwrap().as_bytes()).unwrap();

        let registry = ActuatorRegistry::new("postgres://localhost/askillify");
        let id = registry.register_ure_file(&file_path, None).unwrap();

        assert!(!id.is_nil());
        
        // Create a file named after the UUID to test resolution
        let uuid_file_path = dir.path().join(format!("{}.ure", id));
        let mut uuid_file = File::create(&uuid_file_path).unwrap();
        uuid_file.write_all(serde_json::to_string(&manifest).unwrap().as_bytes()).unwrap();

        let resolved = registry.resolve_actuator(id).unwrap();
        assert_eq!(resolved["resource_type"], "skill");
    }
}
