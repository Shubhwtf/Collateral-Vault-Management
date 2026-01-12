# Database Schema

Complete database schema documentation for the Collateral Vault Management System.

## Overview

The system uses PostgreSQL to store:
- Vault state and balances
- Transaction history
- TVL snapshots for analytics
- MFA configuration and audit logs
- Balance snapshots for historical tracking

## Tables

### `vaults`

Stores vault state for each user.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | SERIAL | PRIMARY KEY | Auto-incrementing ID |
| `owner` | VARCHAR(44) | NOT NULL, UNIQUE | Solana wallet address of vault owner |
| `vault_address` | VARCHAR(44) | NOT NULL, UNIQUE | PDA-derived vault address |
| `total_balance` | BIGINT | NOT NULL, DEFAULT 0 | Total USDT balance in lamports (6 decimals) |
| `locked_balance` | BIGINT | NOT NULL, DEFAULT 0 | Balance locked for open positions |
| `available_balance` | BIGINT | NOT NULL, DEFAULT 0 | Balance available for withdrawal |
| `total_deposited` | BIGINT | NOT NULL, DEFAULT 0 | Cumulative total of all deposits |
| `total_withdrawn` | BIGINT | NOT NULL, DEFAULT 0 | Cumulative total of all withdrawals |
| `mfa_enabled` | BOOLEAN | DEFAULT FALSE | Whether MFA is enabled for this vault |
| `mfa_secret` | VARCHAR(32) | NULL | Base32-encoded TOTP secret |
| `mfa_backup_codes` | TEXT[] | NULL | Array of backup codes (encrypted at app layer) |
| `created_at` | TIMESTAMP WITH TIME ZONE | NOT NULL, DEFAULT NOW() | Vault creation timestamp |
| `updated_at` | TIMESTAMP WITH TIME ZONE | NOT NULL, DEFAULT NOW() | Last update timestamp |

**Constraints:**
- `positive_balances`: All balance fields must be >= 0
- `balance_consistency`: `total_balance = locked_balance + available_balance`

**Indexes:**
- `idx_vaults_owner`: On `owner` column
- `idx_vaults_vault_address`: On `vault_address` column

**Triggers:**
- `update_vaults_updated_at`: Automatically updates `updated_at` on row update

**Example:**
```sql
SELECT * FROM vaults WHERE owner = '5yWWZKjfqhhYJGW9wz9...';
```

### `transactions`

Records all vault transactions for audit trail.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | SERIAL | PRIMARY KEY | Auto-incrementing ID |
| `vault_address` | VARCHAR(44) | NOT NULL | Foreign key to `vaults.vault_address` |
| `transaction_type` | transaction_type | NOT NULL | Type of transaction (enum) |
| `amount` | BIGINT | NOT NULL | Transaction amount in lamports |
| `signature` | VARCHAR(88) | NOT NULL, UNIQUE | Solana transaction signature |
| `created_at` | TIMESTAMP WITH TIME ZONE | NOT NULL, DEFAULT NOW() | Transaction timestamp |

**Transaction Types (Enum):**
- `deposit`: User deposits tokens to vault
- `withdrawal`: User withdraws tokens from vault
- `lock`: Collateral locked for trading
- `unlock`: Collateral unlocked after trading
- `transfer`: Cross-vault transfer

**Constraints:**
- `fk_vault`: Foreign key to `vaults.vault_address` with CASCADE delete
- `positive_amount`: Amount must be > 0
- `transactions_signature_unique`: Signature must be unique

**Indexes:**
- `idx_transactions_vault_address`: On `vault_address` column
- `idx_transactions_created_at`: On `created_at` DESC
- `idx_transactions_type`: On `transaction_type` column

**Example:**
```sql
SELECT * FROM transactions 
WHERE vault_address = '8xYYZKjfqhhYJGW9wz9...' 
ORDER BY created_at DESC 
LIMIT 10;
```

### `balance_snapshots`

Historical balance data for analytics and auditing.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | SERIAL | PRIMARY KEY | Auto-incrementing ID |
| `vault_address` | VARCHAR(44) | NOT NULL | Foreign key to `vaults.vault_address` |
| `total_balance` | BIGINT | NOT NULL | Snapshot of total balance |
| `locked_balance` | BIGINT | NOT NULL | Snapshot of locked balance |
| `available_balance` | BIGINT | NOT NULL | Snapshot of available balance |
| `snapshot_time` | TIMESTAMP WITH TIME ZONE | NOT NULL, DEFAULT NOW() | When snapshot was taken |

**Constraints:**
- `fk_vault_snapshot`: Foreign key to `vaults.vault_address` with CASCADE delete

**Indexes:**
- `idx_balance_snapshots_vault_address`: On `vault_address` column
- `idx_balance_snapshots_snapshot_time`: On `snapshot_time` DESC

**Example:**
```sql
SELECT * FROM balance_snapshots 
WHERE vault_address = '8xYYZKjfqhhYJGW9wz9...' 
ORDER BY snapshot_time DESC;
```

### `tvl_snapshots`

Periodic snapshots of Total Value Locked for charting and analytics.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | SERIAL | PRIMARY KEY | Auto-incrementing ID |
| `snapshot_date` | DATE | NOT NULL, DEFAULT CURRENT_DATE | Date of snapshot |
| `snapshot_time` | TIMESTAMP WITH TIME ZONE | NOT NULL, DEFAULT NOW() | Exact timestamp of snapshot |
| `total_value_locked` | BIGINT | NOT NULL | Total TVL across all vaults |
| `total_users` | INTEGER | NOT NULL | Number of distinct vault owners |
| `active_vaults` | INTEGER | NOT NULL | Number of vaults with balance > 0 |
| `total_deposited` | BIGINT | NOT NULL | Cumulative deposits across all vaults |
| `total_withdrawn` | BIGINT | NOT NULL | Cumulative withdrawals across all vaults |
| `average_balance` | BIGINT | NOT NULL | Average balance per user |
| `created_at` | TIMESTAMP WITH TIME ZONE | NOT NULL, DEFAULT NOW() | Record creation timestamp |

**Indexes:**
- `idx_tvl_snapshots_time`: On `snapshot_time` DESC
- `idx_tvl_snapshots_time_asc`: On `snapshot_time` ASC

**Note:** Multiple snapshots per day are allowed (removed unique constraint in migration 004 for demo mode).

**Example:**
```sql
SELECT * FROM tvl_snapshots 
WHERE snapshot_time >= NOW() - INTERVAL '30 days' 
ORDER BY snapshot_time ASC;
```

### `mfa_audit_log`

Audit log for MFA authentication attempts and configuration changes.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | SERIAL | PRIMARY KEY | Auto-incrementing ID |
| `vault_address` | VARCHAR(44) | NOT NULL | Vault address (no FK to allow logging before vault exists) |
| `action` | VARCHAR(50) | NOT NULL | Action type (see below) |
| `ip_address` | VARCHAR(45) | NULL | IP address of requester (supports IPv6) |
| `user_agent` | TEXT | NULL | User agent string |
| `success` | BOOLEAN | NOT NULL | Whether action succeeded |
| `created_at` | TIMESTAMP WITH TIME ZONE | NOT NULL, DEFAULT NOW() | When action occurred |

**Action Types:**
- `enable`: MFA was enabled
- `disable`: MFA was disabled
- `verify_success`: Successful MFA verification
- `verify_failed`: Failed MFA verification attempt

**Indexes:**
- `idx_mfa_audit_vault`: Composite on `(vault_address, created_at DESC)`
- `idx_mfa_audit_action`: On `(action, created_at DESC)`

**Example:**
```sql
SELECT * FROM mfa_audit_log 
WHERE vault_address = '8xYYZKjfqhhYJGW9wz9...' 
ORDER BY created_at DESC;
```

## Materialized Views

### `tvl_daily_summary`

Aggregated daily TVL statistics (for potential future use).

| Column | Type | Description |
|--------|------|-------------|
| `date` | DATE | Date of summary |
| `avg_tvl` | BIGINT | Average TVL for the day |
| `max_tvl` | BIGINT | Maximum TVL for the day |
| `min_tvl` | BIGINT | Minimum TVL for the day |
| `avg_users` | INTEGER | Average number of users |
| `max_users` | INTEGER | Maximum number of users |

**Index:**
- `idx_tvl_daily_summary_date`: On `date` DESC

**Note:** This view is created but not actively used in the current implementation.

## Functions

### `update_updated_at_column()`

Trigger function that automatically updates the `updated_at` timestamp when a row is modified.

**Usage:** Automatically called by trigger on `vaults` table.

### `take_balance_snapshot()`

Function to create balance snapshots for all vaults.

**Usage:**
```sql
SELECT take_balance_snapshot();
```

This function is typically called periodically by the backend service.

## Relationships

```
vaults (1) ──< (many) transactions
vaults (1) ──< (many) balance_snapshots
```

- Each vault can have many transactions
- Each vault can have many balance snapshots
- `mfa_audit_log` is not strictly related (no FK) to allow logging before vault creation

## Data Types

### `transaction_type` Enum

```sql
CREATE TYPE transaction_type AS ENUM (
  'deposit',
  'withdrawal', 
  'lock',
  'unlock',
  'transfer'
);
```

## Constraints

### Vault Constraints

1. **Positive Balances**: All balance fields must be non-negative
2. **Balance Consistency**: `total_balance = locked_balance + available_balance`
3. **Unique Owner**: Each owner can only have one vault
4. **Unique Vault Address**: Each vault address is unique

### Transaction Constraints

1. **Positive Amount**: Transaction amount must be > 0
2. **Unique Signature**: Each transaction signature is unique
3. **Valid Vault**: Transaction must reference an existing vault

## Indexes Summary

| Table | Index | Columns | Purpose |
|-------|-------|---------|---------|
| `vaults` | `idx_vaults_owner` | `owner` | Fast lookup by owner |
| `vaults` | `idx_vaults_vault_address` | `vault_address` | Fast lookup by vault address |
| `transactions` | `idx_transactions_vault_address` | `vault_address` | Fast transaction queries by vault |
| `transactions` | `idx_transactions_created_at` | `created_at DESC` | Fast chronological queries |
| `transactions` | `idx_transactions_type` | `transaction_type` | Filter by transaction type |
| `balance_snapshots` | `idx_balance_snapshots_vault_address` | `vault_address` | Fast snapshot queries |
| `balance_snapshots` | `idx_balance_snapshots_snapshot_time` | `snapshot_time DESC` | Chronological snapshots |
| `tvl_snapshots` | `idx_tvl_snapshots_time` | `snapshot_time DESC` | Fast TVL chart queries |
| `tvl_snapshots` | `idx_tvl_snapshots_time_asc` | `snapshot_time ASC` | Ascending TVL queries |
| `mfa_audit_log` | `idx_mfa_audit_vault` | `vault_address, created_at DESC` | Fast vault audit queries |
| `mfa_audit_log` | `idx_mfa_audit_action` | `action, created_at DESC` | Security monitoring |

## Common Queries

### Get Vault with Latest Balance

```sql
SELECT * FROM vaults WHERE owner = $1;
```

### Get Recent Transactions

```sql
SELECT * FROM transactions 
WHERE vault_address = $1 
ORDER BY created_at DESC 
LIMIT $2;
```

### Calculate Current TVL

```sql
SELECT COALESCE(SUM(total_balance), 0)::BIGINT as tvl 
FROM vaults;
```

### Get User Distribution

```sql
SELECT 
  CASE 
    WHEN total_balance < 100000000 THEN '0-100'
    WHEN total_balance < 1000000000 THEN '100-1000'
    WHEN total_balance < 10000000000 THEN '1000-10000'
    ELSE '10000+'
  END as range,
  COUNT(*) as count
FROM vaults
WHERE total_balance > 0
GROUP BY range;
```

### Get TVL Chart Data

```sql
SELECT 
  snapshot_time,
  total_value_locked,
  total_users
FROM tvl_snapshots
WHERE snapshot_time >= NOW() - INTERVAL '30 days'
ORDER BY snapshot_time ASC;
```

### Get MFA Audit Trail

```sql
SELECT * FROM mfa_audit_log
WHERE vault_address = $1
ORDER BY created_at DESC
LIMIT 100;
```

## Migration History

1. **001_initial_schema.sql**: Creates core tables (`vaults`, `transactions`, `balance_snapshots`)
2. **002_tvl_snapshots.sql**: Adds `tvl_snapshots` table and materialized view
3. **003_mfa_support.sql**: Adds MFA columns to `vaults` and creates `mfa_audit_log` table
4. **004_allow_multiple_snapshots_per_day.sql**: Removes unique constraint on `tvl_snapshots.snapshot_date` for demo mode

## Notes

- All timestamps use `TIMESTAMP WITH TIME ZONE` for proper timezone handling
- Balance amounts are stored in lamports (smallest unit, 6 decimals for USDT)
- The `mfa_backup_codes` array should be encrypted at the application layer
- `mfa_secret` is stored as base32-encoded string (32 characters)
- Transaction signatures are Solana base58-encoded (88 characters max)
- All addresses (owner, vault_address) are Solana base58-encoded (44 characters)

