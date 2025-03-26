use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct PasswordHashConfig {
    pub mem_cost: u32,
    pub time_cost: u32,
    pub lanes: u32,
    pub hash_length: usize,
}