use crate::identifiers::DuUuid;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub struct ActuatorRegistry {
    _db_connection_string: String,
    champions: HashMap<String, Uuid>,
    /// Directory that `.ure` manifests are resolved from and scanned in.
    /// Defaults to the synthesis output directory rather than the process
    /// working directory, so a manifest written by the transducer is found by
    /// the registry that is meant to serve it. The two used to agree only
    /// because both happened to be the working directory, which meant any
    /// process launched from elsewhere saw an empty mesh.
    base_dir: PathBuf,
}

impl ActuatorRegistry {
    /// `conn_str` is a connection string and is recorded, not interpreted: this
    /// crate resolves manifests from the filesystem, so there is no database to
    /// connect to. The argument is **not** a directory, and a caller that wants
    /// a specific scan root must say so with [`ActuatorRegistry::with_base_dir`].
    pub fn new(conn_str: &str) -> Self {
        Self {
            _db_connection_string: conn_str.to_string(),
            champions: HashMap::new(),
            base_dir: crate::outdir::out_dir(),
        }
    }

    /// Points the registry at an explicit `.ure` root instead of the process
    /// working directory, so resolution does not depend on where the process
    /// happens to be launched from.
    pub fn with_base_dir<P: AsRef<Path>>(conn_str: &str, base_dir: P) -> Self {
        Self {
            _db_connection_string: conn_str.to_string(),
            champions: HashMap::new(),
            base_dir: base_dir.as_ref().to_path_buf(),
        }
    }

    pub fn set_champion(&mut self, capability: &str, id: Uuid) {
        println!("🏆 Champion set for {}: {}", capability, id);
        self.champions.insert(capability.to_string(), id);
    }

    pub fn get_champion(&self, capability: &str) -> Option<Uuid> {
        self.champions.get(capability).cloned()
    }

    /// Removes a champion.
    ///
    /// **This is what makes  safe.** A registry that can install a
    /// winner and never remove one is a write-once cache with extra steps, and a
    /// system that runs forever will eventually install a rule that a later run
    /// refutes. It existed with no caller for as long as promotion was manual,
    /// which is exactly the state in which nothing needs it.
    pub fn clear_champion(&mut self, capability: &str) -> Option<Uuid> {
        self.champions.remove(capability)
    }

    /// How many champions are installed. A summary number, so a caller can check
    /// the loop did something without knowing which rules it touched.
    pub fn champion_count(&self) -> usize {
        self.champions.len()
    }

    pub fn register_ure_file<P: AsRef<Path>>(
        &self,
        path: P,
        encryption_key: Option<&[u8; 32]>,
    ) -> Result<Uuid, Box<dyn std::error::Error>> {
        let content_str = fs::read_to_string(path)?;
        let manifest: Value = serde_json::from_str(&content_str)?;
        let resource_id = DuUuid::generate(&manifest, encryption_key)?;

        println!(
            "Registering Actuator: {} | ID: {}",
            manifest["resource_type"], resource_id
        );
        Ok(resource_id)
    }

    pub fn resolve_actuator(&self, id: Uuid) -> Result<Value, Box<dyn std::error::Error>> {
        let path = self.base_dir.join(format!("{}.ure", id));
        let content = fs::read_to_string(path)?;
        let manifest: Value = serde_json::from_str(&content)?;
        Ok(manifest)
    }

    /// Scans the registry for actuators that match a prompt's intent.
    pub fn find_matching_actuators(&self, prompt: &str) -> Vec<(Uuid, Value)> {
        println!("Scanning mesh for patterns matching: '{}'...", prompt);
        let mut matches = Vec::new();

        if let Ok(entries) = fs::read_dir(&self.base_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("ure") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(manifest) = serde_json::from_str::<Value>(&content) {
                            let guidance = manifest["guidance"].as_str().unwrap_or("");
                            if guidance.to_lowercase().contains(&prompt.to_lowercase())
                                || prompt.to_lowercase().contains(&guidance.to_lowercase())
                            {
                                if let Some(id_str) = manifest["resource_id"].as_str() {
                                    if let Ok(id) = Uuid::parse_str(id_str) {
                                        matches.push((id, manifest));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// A manifest with the structure a real one has, minus everything the
    /// address deliberately ignores.
    fn manifest() -> Value {
        json!({
            "resource_id": "should-not-affect-the-address",
            "category": "actuator",
            "complexity_score": 0.3,
            "guidance": "Standard income generation path."
        })
    }

    #[test]
    fn a_manifest_registers_under_a_content_address() {
        // Registration is the step that turns a file on disk into something the
        // rest of the system can name, and the name has to come from the
        // manifest's content rather than from its filename.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test_actuator.ure");
        fs::write(&path, serde_json::to_string(&manifest()).unwrap()).unwrap();

        let registry = ActuatorRegistry::with_base_dir("mock", dir.path());
        let id = registry.register_ure_file(&path, None).unwrap();

        assert!(!id.is_nil(), "a nil address is not an address");
    }

    #[test]
    fn the_same_manifest_registers_under_the_same_address() {
        // Two files, one act. If this fails, two people describing the same
        // thing differently get two artifacts, which is the defect the
        // skeleton hash exists to prevent.
        let dir = tempfile::tempdir().unwrap();
        let registry = ActuatorRegistry::with_base_dir("mock", dir.path());

        let a = dir.path().join("a.ure");
        let b = dir.path().join("b.ure");
        fs::write(&a, serde_json::to_string(&manifest()).unwrap()).unwrap();
        let mut other = manifest();
        other["resource_id"] = json!("a-different-spelling-of-the-same-thing");
        fs::write(&b, serde_json::to_string(&other).unwrap()).unwrap();

        assert_eq!(
            registry.register_ure_file(&a, None).unwrap(),
            registry.register_ure_file(&b, None).unwrap()
        );
    }

    #[test]
    fn a_registered_manifest_resolves_from_its_base_dir() {
        // Resolution and registration have to agree on where manifests live.
        // This test used to pass only because both fell back to the process
        // working directory, which meant it asserted nothing about the registry
        // and would have failed for any process launched from anywhere else.
        let dir = tempfile::tempdir().unwrap();
        let registry = ActuatorRegistry::with_base_dir("mock", dir.path());

        // `register_ure_file` reports an address but does not rename the file,
        // because a manifest is content-addressed rather than filed under its
        // address. Resolution expects the latter, so the file is written under
        // the address the manifest actually has.
        let id = DuUuid::generate(&manifest(), None).unwrap();
        fs::write(
            dir.path().join(format!("{id}.ure")),
            serde_json::to_string(&manifest()).unwrap(),
        )
        .unwrap();

        let resolved = registry.resolve_actuator(id).unwrap();
        assert_eq!(resolved["complexity_score"], json!(0.3));
    }
}
