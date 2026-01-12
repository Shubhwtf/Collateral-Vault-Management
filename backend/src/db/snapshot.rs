use sqlx::PgPool;
use tracing;

use crate::error::{Result, VaultError};
use crate::db::models::TvlSnapshot;

pub struct SnapshotService {
    db_pool: PgPool,
}

impl SnapshotService {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    pub async fn take_snapshot(&self) -> Result<TvlSnapshot> {
        tracing::info!("Taking TVL snapshot...");

        // doing all the aggregation in one query to avoid race conditions
        // between reading different metrics at slightly different times
        let snapshot = sqlx::query_as::<_, TvlSnapshot>(
            r#"
            INSERT INTO public.tvl_snapshots (
                snapshot_date,
                snapshot_time,
                total_value_locked,
                total_users,
                active_vaults,
                total_deposited,
                total_withdrawn,
                average_balance
            )
            SELECT 
                CURRENT_DATE,
                NOW(),
                COALESCE(SUM(total_balance), 0)::BIGINT,
                COUNT(DISTINCT owner)::INTEGER,
                COUNT(*) FILTER (WHERE total_balance > 0)::INTEGER,
                COALESCE(SUM(total_deposited), 0)::BIGINT,
                COALESCE(SUM(total_withdrawn), 0)::BIGINT,
                CASE 
                    WHEN COUNT(DISTINCT owner) > 0 
                    THEN (COALESCE(SUM(total_balance), 0) / COUNT(DISTINCT owner))::BIGINT
                    ELSE 0
                END
            FROM public.vaults
            RETURNING *
            "#
        )
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| VaultError::Database(e.to_string()))?;

        tracing::info!(
            "Snapshot taken: TVL={}, users={}, active_vaults={}",
            snapshot.total_value_locked,
            snapshot.total_users,
            snapshot.active_vaults
        );

        Ok(snapshot)
    }

    // 1 minute interval for demo purposes - would be hourly or daily in prod
    pub async fn run_periodic_snapshot(&self) -> Result<()> {
        loop {
            match self.take_snapshot().await {
                Ok(_) => tracing::info!("Periodic snapshot completed successfully"),
                Err(e) => tracing::error!("Failed to take periodic snapshot: {}", e),
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
    }
}
