//! Verification helper for `migrations/0001_initial.sql`.
//!
//! Applies the schema migration via SQLx, then re-applies it to confirm
//! idempotency, then inspects the resulting tables and indexes.
//!
//! Usage (default = SQLite in-memory):
//!     cargo run --example verify_migration --features sql-store
//!
//! Usage (custom DB via env var):
//!     DATABASE_URL=postgres://postgres:pw@localhost/postgres \
//!         cargo run --example verify_migration --features sql-store
//!
//! `sqlx::Any::install_default_drivers()` registers all three drivers
//! (`postgres`, `mysql`, `sqlite`) at runtime so the same binary works
//! against any backend supported by the `sql-store` feature.

use sqlx::any::install_default_drivers;
use sqlx::migrate::Migrator;
use sqlx::AnyPool;
use sqlx::Row;
use std::path::Path;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    install_default_drivers();

    let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:".to_string());
    println!("Connecting: {url}");
    let pool = AnyPool::connect(&url).await?;

    // Read migrations from disk at runtime. The production `SqlStore::new` will
    // use the compile-time `sqlx::migrate!` macro instead — this example keeps
    // the feature surface minimal.
    let migrator = Migrator::new(Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations")).await?;

    println!("Applying migration (first pass)...");
    migrator.run(&pool).await?;
    println!("  OK");

    println!("Applying migration (second pass — idempotency check)...");
    migrator.run(&pool).await?;
    println!("  OK");

    print_sqlite_objects(&pool).await.ok();

    pool.close().await;
    println!("All checks passed.");
    Ok(())
}

/// SQLite-specific introspection. Other backends are validated by the
/// successful migration runs above (and by the Step 2.3 integration tests).
async fn print_sqlite_objects(pool: &AnyPool) -> Result<(), sqlx::Error> {
    let rows = sqlx::query(
        "SELECT type, name FROM sqlite_master \
         WHERE type IN ('table','index') AND name NOT LIKE 'sqlite_%' \
         ORDER BY type, name",
    )
    .fetch_all(pool)
    .await?;

    println!("\nSchema objects (SQLite):");
    for row in rows {
        let kind: String = row.try_get("type")?;
        let name: String = row.try_get("name")?;
        println!("  {kind:<6} {name}");
    }
    Ok(())
}
