/// Configuration for Argon2 password hashing parameters.
#[derive(Debug, Clone)]
pub struct PasswordHashConfig {
    /// Memory cost in kibibytes (1 KiB = 1024 bytes). Higher values increase resistance to brute-force attacks.
    /// Recommended: at least 65536 (64 MiB) in production environments.
    pub mem_cost: u32,

    /// Number of iterations (aka time cost). Higher values increase CPU effort.
    /// Recommended: at least 3 for most applications.
    pub time_cost: u32,

    /// Degree of parallelism (number of threads/lanes to use).
    /// Usually matches the number of CPU cores available.
    pub lanes: u32,

    /// Desired length of the resulting password hash in bytes.
    /// Common values are 32 (256 bits) or 64 (512 bits).
    pub hash_length: usize,
}
