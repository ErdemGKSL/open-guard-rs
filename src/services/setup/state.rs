use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use crate::db::entities::module_configs::ModuleType;
use std::collections::HashMap;
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};

fn generate_random_id() -> String {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap_or_default();
    let mut seed = since_the_epoch.as_nanos() as u64;
    
    let mut id = String::with_capacity(10);
    let chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let chars_len = chars.len();
    
    for _ in 0..10 {
        // Simple LCG
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let idx = ((seed >> 32) as usize) % chars_len;
        id.push(chars.as_bytes()[idx] as char);
    }
    id
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SetupStep {
    Systems,
    Logging,
    Whitelist,
    ModuleConfig(ModuleType),
    Summary,
}

#[derive(Debug, Clone)]
pub struct SetupState {
    pub id: String,
    pub guild_id: u64,
    pub last_interaction: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub enabled_modules: Vec<ModuleType>,
    pub fallback_log_channel: Option<u64>,
    pub whitelist_users: Vec<u64>,
    pub whitelist_roles: Vec<u64>,
    pub module_configs: HashMap<ModuleType, Value>,
    pub current_step: SetupStep,
    pub pending_modules: Vec<ModuleType>,
}

pub struct SetupStateService {
    states: DashMap<u64, SetupState>,
}

impl SetupStateService {
    pub fn new() -> Self {
        Self {
            states: DashMap::new(),
        }
    }

    pub fn start_setup(&self, guild_id: u64) -> Result<String, String> {
        let now = Utc::now();
        if let Some(state) = self.states.get(&guild_id) {
            let age = now - state.created_at;
            let idle = now - state.last_interaction;

            // TTL 15 mins, Idle 3 mins
            if age < Duration::minutes(15) && idle < Duration::minutes(3) {
                return Err("Setup is already in progress for this guild.".to_string());
            }
        }

        let id = generate_random_id();

        let state = SetupState {
            id: id.clone(),
            guild_id,
            last_interaction: now,
            created_at: now,
            enabled_modules: Vec::new(),
            fallback_log_channel: None,
            whitelist_users: Vec::new(),
            whitelist_roles: Vec::new(),
            module_configs: HashMap::new(),
            current_step: SetupStep::Systems,
            pending_modules: Vec::new(),
        };

        self.states.insert(guild_id, state);
        Ok(id)
    }

    pub fn get_state(&self, guild_id: u64) -> Option<SetupState> {
        self.states.get(&guild_id).map(|s| s.clone())
    }

    pub fn update_state<F>(&self, guild_id: u64, f: F) -> bool
    where
        F: FnOnce(&mut SetupState),
    {
        if let Some(mut state) = self.states.get_mut(&guild_id) {
            f(&mut state);
            state.last_interaction = Utc::now();
            true
        } else {
            false
        }
    }

    pub fn cancel_setup(&self, guild_id: u64) {
        self.states.remove(&guild_id);
    }
}
