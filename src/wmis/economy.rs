use uuid::Uuid;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use crate::wmis::WmisResource;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WmisOperation {
    Read,
    Write,
    Execute,
    Transform,
    Delegate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectivePermission {
    pub principal: String,
    pub resource_id: Uuid,
    pub operations: Vec<WmisOperation>,
    pub percentage: f64, // 0.0 to 100.0
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WmisEconomicLayer {
    pub token_wallets: HashMap<String, f64>,
    #[serde(skip)]
    persistence_path: Option<String>,
}

impl WmisEconomicLayer {
    pub fn new() -> Self {
        Self {
            token_wallets: HashMap::new(),
            persistence_path: None,
        }
    }

    pub fn with_persistence<P: AsRef<Path>>(path: P) -> Self {
        let path_str = path.as_ref().to_string_lossy().into_owned();
        let mut economy = Self::new();
        economy.persistence_path = Some(path_str.clone());
        
        if let Ok(data) = fs::read_to_string(&path_str) {
            if let Ok(wallets) = serde_json::from_str(&data) {
                economy.token_wallets = wallets;
            }
        }
        economy
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref path) = self.persistence_path {
            let data = serde_json::to_string_pretty(&self.token_wallets)?;
            fs::write(path, data)?;
            Ok(())
        } else {
            Err("Persistence path not configured".into())
        }
    }

    /// Calculates cost based on Resource Quality (QoR) and Operation type.
    /// Formula: BaseCost * (1 + QoR) * OpMultiplier
    pub fn calculate_cost(&self, resource: &WmisResource, op: &WmisOperation) -> f64 {
        let base_cost = 1.0;
        let qor_multiplier = 1.0 + resource.quality.qor;
        
        let op_multiplier = match op {
            WmisOperation::Read => 0.5,
            WmisOperation::Execute => 1.0,
            WmisOperation::Transform => 2.0,
            WmisOperation::Delegate => 1.5,
            WmisOperation::Write => 2.0,
        };

        base_cost * qor_multiplier * op_multiplier
    }

    pub fn charge_actuation(&mut self, user: &str, resource: &WmisResource, op: &WmisOperation) -> Result<(), String> {
        let cost = self.calculate_cost(resource, op);
        let balance = self.token_wallets.entry(user.to_string()).or_insert(100.0);

        if *balance >= cost {
            *balance -= cost;
            println!("💰 Charged {:.2} tokens from {}. Resource: {} (QoR: {:.2}). New balance: {:.2}", 
                cost, user, resource.id, resource.quality.qor, balance);
            
            if let Some(_) = &self.persistence_path {
                let _ = self.save();
            }
            Ok(())
        } else {
            Err(format!("Insufficient tokens. Need {:.2}, have {:.2}", cost, balance))
        }
    }

    pub fn charge_fixed(&mut self, user: &str, cost: f64) -> Result<(), String> {
        let balance = self.token_wallets.entry(user.to_string()).or_insert(100.0);
        if *balance >= cost {
            *balance -= cost;
            println!("💰 Charged fixed {:.2} tokens from {}. New balance: {:.2}", cost, user, balance);
            if let Some(_) = &self.persistence_path {
                let _ = self.save();
            }
            Ok(())
        } else {
            Err("Insufficient tokens".to_string())
        }
    }

    pub fn reward_contributor(&mut self, user: &str, reward: f64) {
        let balance = self.token_wallets.entry(user.to_string()).or_insert(0.0);
        *balance += reward;
        println!("🎁 Rewarded {:.2} tokens to contributor {}. New balance: {:.2}", reward, user, balance);
        if let Some(_) = &self.persistence_path {
            let _ = self.save();
        }
    }
}

pub struct WmisPermissionEngine {
    permissions: HashMap<Uuid, Vec<ObjectivePermission>>,
}

impl WmisPermissionEngine {
    pub fn new() -> Self {
        Self {
            permissions: HashMap::new(),
        }
    }

    pub fn grant_permission(&mut self, perm: ObjectivePermission) {
        self.permissions.entry(perm.resource_id).or_default().push(perm);
    }

    pub fn check_permission(&self, principal: &str, resource_id: Uuid, op: WmisOperation) -> bool {
        if let Some(perms) = self.permissions.get(&resource_id) {
            return perms.iter().any(|p| p.principal == principal && p.operations.contains(&op));
        }
        false
    }
}
