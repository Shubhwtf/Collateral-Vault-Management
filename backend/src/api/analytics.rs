use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use sqlx::Row;

use crate::AppState;
use crate::error::Result;

#[derive(Debug, Serialize)]
pub struct AnalyticsOverview {
    pub total_value_locked: i64,
    pub total_users: i64,
    pub total_deposits: i64,
    pub total_withdrawals: i64,
    pub active_vaults: i64,
    pub average_balance: f64,
    pub total_yield_earned: i64,
}

#[derive(Debug, Serialize)]
pub struct UserDistribution {
    pub balance_range: String,
    pub user_count: i64,
    pub percentage: f64,
}

#[derive(Debug, Serialize)]
pub struct CollateralUtilization {
    pub total_collateral: i64,
    pub locked_collateral: i64,
    pub available_collateral: i64,
    pub utilization_rate: f64,
}

#[derive(Debug, Serialize)]
pub struct FlowMetrics {
    pub period: String,
    pub deposits: i64,
    pub withdrawals: i64,
    pub net_flow: i64,
    pub deposit_count: i64,
    pub withdrawal_count: i64,
}

#[derive(Debug, Serialize)]
pub struct YieldMetrics {
    pub total_yield_earned: i64,
    pub average_apy: f64,
    pub active_yield_vaults: i64,
    pub total_yield_vaults: i64,
}

#[derive(Debug, Deserialize)]
pub struct TimeRangeQuery {
    pub days: Option<i32>,
}

/// Get comprehensive analytics overview
pub async fn get_overview(
    State(state): State<Arc<AppState>>,
) -> Result<Json<AnalyticsOverview>> {
    let db = &state.db_pool;

    // cast to BIGINT because SUM returns NUMERIC which causes type issues with sqlx
    let tvl_result = sqlx::query("SELECT COALESCE(SUM(total_balance), 0)::BIGINT as tvl FROM vaults")
        .fetch_one(db)
        .await?;
    let total_value_locked: i64 = tvl_result.get("tvl");

    // Count distinct users based on the 'owner' field
    let users_result = sqlx::query("SELECT COUNT(DISTINCT owner) as count FROM vaults")
        .fetch_one(db)
        .await?;
    let total_users: i64 = users_result.get("count");

    // Sum up total deposits and withdrawals across all vaults
    let totals_result = sqlx::query(
        "SELECT COALESCE(SUM(total_deposited), 0)::BIGINT as deposits, COALESCE(SUM(total_withdrawn), 0)::BIGINT as withdrawals FROM vaults"
    )
    .fetch_one(db)
    .await?;
    let total_deposits: i64 = totals_result.get("deposits");
    let total_withdrawals: i64 = totals_result.get("withdrawals");

    // Count active vaults with a balance greater than 0
    let active_result = sqlx::query("SELECT COUNT(*) as count FROM vaults WHERE total_balance > 0")
        .fetch_one(db)
        .await?;
    let active_vaults: i64 = active_result.get("count");

    // Calculate average balance per user, avoiding division by zero
    let average_balance = if total_users > 0 {
        total_value_locked as f64 / total_users as f64
    } else {
        0.0
    };

    // yield tracking wasn't in scope for this sprint
    let total_yield_earned = 0i64;

    Ok(Json(AnalyticsOverview {
        total_value_locked,
        total_users,
        total_deposits,
        total_withdrawals,
        active_vaults,
        average_balance,
        total_yield_earned,
    }))
}

/// Get user balance distribution
pub async fn get_user_distribution(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<UserDistribution>>> {
    let db = &state.db_pool;

    // ranges in lamports (6 decimals for USDT)
    let ranges = vec![
        ("0-100", 0i64, 100_000_000i64),
        ("100-1000", 100_000_000i64, 1_000_000_000i64),
        ("1000-10000", 1_000_000_000i64, 10_000_000_000i64),
        ("10000+", 10_000_000_000i64, i64::MAX),
    ];

    // Get total users with a balance greater than 0
    let total_result = sqlx::query("SELECT COUNT(*) as count FROM vaults WHERE total_balance > 0")
        .fetch_one(db)
        .await?;
    let total_users: i64 = total_result.get("count");
    let total_users = total_users.max(1) as f64; // avoid division by zero
    
    let mut distribution = Vec::new();

    // Calculate user distribution across predefined balance ranges
    for (label, min, max) in ranges {
        let result = if max == i64::MAX {
            sqlx::query(
                "SELECT COUNT(*) AS count FROM vaults WHERE total_balance >= $1"
            )
            .bind(min)
            .fetch_one(db)
            .await?
        } else {
            sqlx::query(
                "SELECT COUNT(*) AS count FROM vaults
                 WHERE total_balance >= $1 AND total_balance < $2"
            )
            .bind(min)
            .bind(max)
            .fetch_one(db)
            .await?
        };

        let user_count: i64 = result.get("count");        
        let percentage = (user_count as f64 / total_users) * 100.0;

        distribution.push(UserDistribution {
            balance_range: label.to_string(),
            user_count,
            percentage,
        });
    }

    Ok(Json(distribution))
}

/// Get collateral utilization metrics
pub async fn get_utilization(
    State(state): State<Arc<AppState>>,
) -> Result<Json<CollateralUtilization>> {
    let db = &state.db_pool;

    // Query total, locked, and available balances
    let result = sqlx::query(
        "SELECT 
            COALESCE(SUM(total_balance), 0)::BIGINT as total,
            COALESCE(SUM(locked_balance), 0)::BIGINT as locked,
            COALESCE(SUM(available_balance), 0)::BIGINT as available
         FROM vaults"
    )
    .fetch_one(db)
    .await?;

    let total_collateral: i64 = result.get("total");
    let locked_collateral: i64 = result.get("locked");
    let available_collateral: i64 = result.get("available");

    // Calculate utilization rate as a percentage
    let utilization_rate = if total_collateral > 0 {
        (locked_collateral as f64 / total_collateral as f64) * 100.0
    } else {
        0.0
    };

    Ok(Json(CollateralUtilization {
        total_collateral,
        locked_collateral,
        available_collateral,
        utilization_rate,
    }))
}

/// Get deposit/withdrawal flow metrics
pub async fn get_flow_metrics(
    State(state): State<Arc<AppState>>,
    Query(params): Query<TimeRangeQuery>,
) -> Result<Json<Vec<FlowMetrics>>> {
    let db = &state.db_pool;
    let days = params.days.unwrap_or(30);

    // using query_as with explicit types because the macro had inference issues with CASE expressions
    let result = sqlx::query_as::<_, (Option<chrono::NaiveDate>, Option<i64>, Option<i64>, Option<i64>, Option<i64>)>(
        r#"
        SELECT 
            DATE(timestamp) as period,
            COALESCE(SUM(CASE WHEN transaction_type = 'deposit' THEN amount ELSE 0 END), 0) as deposits,
            COALESCE(SUM(CASE WHEN transaction_type = 'withdrawal' THEN amount ELSE 0 END), 0) as withdrawals,
            COALESCE(COUNT(CASE WHEN transaction_type = 'deposit' THEN 1 END), 0) as deposit_count,
            COALESCE(COUNT(CASE WHEN transaction_type = 'withdrawal' THEN 1 END), 0) as withdrawal_count
        FROM transactions
        WHERE timestamp >= NOW() - $1 * INTERVAL '1 day'
        GROUP BY DATE(timestamp)
        ORDER BY DATE(timestamp) DESC
        "#,
    )
    .bind(days)
    .fetch_all(db)
    .await?;

    // Map query results into FlowMetrics structs
    let metrics = result
        .into_iter()
        .map(|(period, deposits, withdrawals, deposit_count, withdrawal_count)| {
            let deposits = deposits.unwrap_or(0);
            let withdrawals = withdrawals.unwrap_or(0);
            let net_flow = deposits - withdrawals;

            FlowMetrics {
                period: period.map(|d| d.to_string()).unwrap_or_default(),
                deposits,
                withdrawals,
                net_flow,
                deposit_count: deposit_count.unwrap_or(0),
                withdrawal_count: withdrawal_count.unwrap_or(0),
            }
        })
        .collect();

    Ok(Json(metrics))
}

/// Get yield metrics
pub async fn get_yield_metrics(
    State(state): State<Arc<AppState>>,
) -> Result<Json<YieldMetrics>> {
    let _db = &state.db_pool;

    // placeholder - yield tracking would need additional schema columns and program changes
    Ok(Json(YieldMetrics {
        total_yield_earned: 0,
        average_apy: 5.0,
        active_yield_vaults: 0,
        total_yield_vaults: 0,
    }))
}

/// Get time series data for charts
#[derive(Debug, Serialize)]
pub struct TimeSeriesPoint {
    pub timestamp: String,
    pub tvl: i64,
    pub user_count: i64,
}

pub async fn get_tvl_chart(
    State(state): State<Arc<AppState>>,
    Query(params): Query<TimeRangeQuery>,
) -> Result<Json<Vec<TimeSeriesPoint>>> {
    let db = &state.db_pool;
    let days = params.days.unwrap_or(30);

    // returning individual snapshots instead of aggregating by day for demo mode
    // this lets us show minute-by-minute updates on the chart
    let snapshots = sqlx::query_as::<_, (chrono::DateTime<chrono::Utc>, i64, i32)>(
        r#"
        SELECT 
            snapshot_time,
            total_value_locked,
            total_users
        FROM public.tvl_snapshots
        WHERE snapshot_time >= NOW() - $1 * INTERVAL '1 day'
        ORDER BY snapshot_time ASC
        "#
    )
    .bind(days)
    .fetch_all(db)
    .await?;

    // fallback if no snapshots exist yet (fresh install)
    if snapshots.is_empty() {
        let tvl_result = sqlx::query("SELECT COALESCE(SUM(total_balance), 0)::BIGINT as tvl FROM vaults")
            .fetch_one(db)
            .await?;
        let current_tvl: i64 = tvl_result.get("tvl");
        
        let users_result = sqlx::query("SELECT COUNT(DISTINCT owner)::INTEGER as count FROM vaults")
            .fetch_one(db)
            .await?;
        let current_users: i32 = users_result.get("count");
        
        return Ok(Json(vec![TimeSeriesPoint {
            timestamp: chrono::Utc::now().to_rfc3339(),
            tvl: current_tvl,
            user_count: current_users as i64,
        }]));
    }

    // Map snapshots into TimeSeriesPoint structs
    let chart_data = snapshots
        .into_iter()
        .map(|(timestamp, tvl, users)| TimeSeriesPoint {
            timestamp: timestamp.to_rfc3339(),
            tvl,
            user_count: users as i64,
        })
        .collect();
        
    Ok(Json(chart_data))
}
