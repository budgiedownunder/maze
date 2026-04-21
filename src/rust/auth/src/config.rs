use serde::{Deserialize, Serialize};
/// Configuration for Argon2 password hashing parameters.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PasswordHashConfig {
    /// Memory cost in kibibytes (1 KiB = 1024 bytes). Higher values increase resistance to brute-force attacks.
    /// Recommended: at least 65536 (64 MiB) in production environments.
    #[serde(default = "default_mem_cost")]
    pub mem_cost: u32,

    /// Number of iterations (aka time cost). Higher values increase CPU effort.
    /// Recommended: at least 3 for most applications.
    #[serde(default = "default_time_cost")]
    pub time_cost: u32,

    /// Degree of parallelism (number of threads/lanes to use).
    /// Usually matches the number of CPU cores available.
    #[serde(default = "default_lanes")]
    pub lanes: u32,

    /// Desired length of the resulting password hash in bytes.
    /// Common values are 32 (256 bits) or 64 (512 bits).
    #[serde(default = "default_hash_length")]
    pub hash_length: usize,
}

impl Default for PasswordHashConfig {
    fn default() -> Self {
        Self {
            mem_cost: default_mem_cost(),
            time_cost: default_time_cost(),
            lanes: default_lanes(),
            hash_length: default_hash_length(),
        }
    }
}

impl PasswordHashConfig {
    /// Returns a minimal-cost configuration suitable for use in tests only.
    /// Never use this in production — the parameters provide no real security.
    pub fn for_testing() -> Self {
        Self {
            mem_cost: 8,
            time_cost: 1,
            lanes: 1,
            hash_length: default_hash_length(),
        }
    }
}

const DEFAULT_MEM_COST:u32 = 65536;
const DEFAULT_TIME_COST:u32 = 3;
const DEFAULT_LANES:u32 = 4;
const DEFAULT_HASH_LENGTH:usize = 32;

fn default_mem_cost() -> u32 {
    DEFAULT_MEM_COST
}

fn default_time_cost() -> u32 {
    DEFAULT_TIME_COST
}

fn default_lanes() -> u32 {
    DEFAULT_LANES
}

fn default_hash_length() -> usize {
    DEFAULT_HASH_LENGTH
}
