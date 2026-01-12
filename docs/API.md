# API Documentation

Complete API reference for the Collateral Vault Management System.

## Base URL

- **Development**: `http://localhost:8080`
- **Production**: `https://api.yourdomain.com`

## Authentication

Most endpoints require a `user_pubkey` parameter. Some operations require MFA verification (see MFA section).

### Rate Limiting

The API implements a 3-tier rate limiting system:

- **Anonymous**: 100 requests/minute
- **Authenticated** (with wallet signature/API key): 500 requests/minute
- **Premium**: 2000 requests/minute

Rate limit headers are included in all responses:
```
X-RateLimit-Limit: 500
X-RateLimit-Remaining: 450
X-RateLimit-Reset: 1736715780
Retry-After: 52
```

When rate limit is exceeded, you'll receive a `429 Too Many Requests` response with details.

## Health & Configuration

### Health Check

```http
GET /health
```

**Response:**
```json
{
  "status": "healthy",
  "service": "Collateral Vault Management System",
  "version": "0.1.0"
}
```

### Public Configuration

```http
GET /config/public
```

**Response:**
```json
{
  "program_id": "pjYYA2y9UL5N4EDd8wKLySDCvb3N6zCoPtFU8WYsnDP",
  "usdt_mint": "4vKYTWtyt4BoVAC24yb1Bdsij2EagB4ep8krEkKoYVxA",
  "solana_rpc_url": "https://api.devnet.solana.com"
}
```

## Vault Operations

### Initialize Vault

Builds an unsigned transaction to initialize a new vault.

```http
POST /vault/initialize
Content-Type: application/json

{
  "user_pubkey": "5yWWZKjfqhhYJGW9wz9..."
}
```

**Response:**
```json
{
  "transaction_base64": "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAED...",
  "recent_blockhash": "GjJyeW1vZGVscG9pbnQ...",
  "fee_payer": "5yWWZKjfqhhYJGW9wz9..."
}
```

**Usage:**
1. Client receives unsigned transaction
2. User signs with wallet
3. Transaction submitted to Solana
4. Call `/vault/sync` to update database

### Build Deposit Transaction

```http
POST /vault/deposit
Content-Type: application/json

{
  "user_pubkey": "5yWWZKjfqhhYJGW9wz9...",
  "amount": 1000000000
}
```

**Response:** Same format as initialize

**Note:** `amount` is in token's smallest unit (e.g., 1000000000 = 1000 USDT for 6 decimals)

### Build Withdraw Transaction

```http
POST /vault/withdraw
Content-Type: application/json

{
  "user_pubkey": "5yWWZKjfqhhYJGW9wz9...",
  "amount": 500000000
}
```

**Response:** Same format as initialize

### Sync Transaction

Syncs a submitted transaction with the database. Call this after submitting a transaction to Solana.

```http
POST /vault/sync
Content-Type: application/json

{
  "user_pubkey": "5yWWZKjfqhhYJGW9wz9...",
  "signature": "3K1c7vx...",
  "transaction_type": "deposit",
  "amount": 1000000000
}
```

**Transaction Types:**
- `deposit`
- `withdrawal`
- `lock`
- `unlock`
- `transfer`

**Response:**
```json
{
  "vault": {
    "owner": "5yWWZKjfqhhYJGW9wz9...",
    "vault_address": "8xYYZKjfqhhYJGW9wz9...",
    "total_balance": 1000000000,
    "available_balance": 1000000000,
    "locked_balance": 0,
    "total_deposited": 1000000000,
    "total_withdrawn": 0,
    "created_at": "2024-01-12T10:00:00Z",
    "updated_at": "2024-01-12T10:00:00Z"
  },
  "recorded": true
}
```

### Force Sync Vault

Manually sync vault state from on-chain data without a transaction signature. Useful for bootstrapping existing vaults.

```http
POST /vault/force-sync
Content-Type: application/json

{
  "user_pubkey": "5yWWZKjfqhhYJGW9wz9..."
}
```

**Response:** Same format as sync transaction

### Get Balance

```http
GET /vault/balance/:user_pubkey
```

**Response:**
```json
{
  "vault": {
    "owner": "5yWWZKjfqhhYJGW9wz9...",
    "vault_address": "8xYYZKjfqhhYJGW9wz9...",
    "total_balance": 1000000000,
    "available_balance": 800000000,
    "locked_balance": 200000000,
    "total_deposited": 1500000000,
    "total_withdrawn": 500000000,
    "created_at": "2024-01-12T10:00:00Z",
    "updated_at": "2024-01-12T10:00:00Z"
  }
}
```

### Get Transaction History

```http
GET /vault/transactions/:user_pubkey
```

**Response:**
```json
{
  "transactions": [
    {
      "id": 1,
      "vault_address": "8xYYZKjfqhhYJGW9wz9...",
      "transaction_type": "deposit",
      "amount": 1000000000,
      "signature": "3K1c7vx...",
      "created_at": "2024-01-12T10:00:00Z"
    }
  ]
}
```

### Get Total Value Locked (TVL)

```http
GET /vault/tvl
```

**Response:**
```json
{
  "total_value_locked": 50000000000
}
```

## Analytics Endpoints

### Overview

```http
GET /analytics/overview
```

**Response:**
```json
{
  "total_value_locked": 50000000000,
  "total_users": 1250,
  "total_deposits": 75000000000,
  "total_withdrawals": 25000000000,
  "active_vaults": 1000,
  "average_balance": 40000000.0,
  "total_yield_earned": 0
}
```

### User Distribution

```http
GET /analytics/distribution
```

**Response:**
```json
{
  "distribution": [
    {"balance_range": "0-100", "user_count": 500, "percentage": 40.0},
    {"balance_range": "100-1000", "user_count": 400, "percentage": 32.0},
    {"balance_range": "1000-10000", "user_count": 300, "percentage": 24.0},
    {"balance_range": "10000+", "user_count": 50, "percentage": 4.0}
  ]
}
```

**Note:** Balance ranges are in USDT (6 decimals), so "0-100" means 0-100 USDT.

### Utilization

```http
GET /analytics/utilization
```

**Response:**
```json
{
  "total_collateral": 50000000000,
  "locked_collateral": 20000000000,
  "available_collateral": 30000000000,
  "utilization_rate": 40.0
}
```

### Flow Metrics

```http
GET /analytics/flow?days=30
```

**Query Parameters:**
- `days` (optional): Number of days to look back (default: 30)

**Response:**
```json
[
  {
    "period": "2024-01-12",
    "deposits": 5000000000,
    "withdrawals": 2000000000,
    "net_flow": 3000000000,
    "deposit_count": 25,
    "withdrawal_count": 10
  }
]
```

### TVL Chart Data

```http
GET /analytics/chart/tvl?days=30
```

**Query Parameters:**
- `days` (optional): Number of days to look back (default: 30)

**Response:**
```json
[
  {
    "timestamp": "2024-01-12T10:00:00Z",
    "tvl": 45000000000,
    "user_count": 1200
  },
  {
    "timestamp": "2024-01-12T11:00:00Z",
    "tvl": 48000000000,
    "user_count": 1250
  }
]
```

### Yield Metrics

```http
GET /analytics/yield
```

**Response:**
```json
{
  "total_yield_earned": 0,
  "average_apy": 5.0,
  "active_yield_vaults": 0,
  "total_yield_vaults": 0
}
```

## Yield Operations

### Compound Yield

Builds an unsigned transaction to compound accumulated yield.

```http
POST /yield/compound
Content-Type: application/json

{
  "user_pubkey": "5yWWZKjfqhhYJGW9wz9..."
}
```

**Response:** Transaction response (same format as initialize)

### Auto-Compound Yield

Allows anyone to trigger yield compounding (throttled to once per hour on-chain).

```http
POST /yield/auto-compound
Content-Type: application/json

{
  "vault_owner_pubkey": "5yWWZKjfqhhYJGW9wz9...",
  "caller_pubkey": "7xYYZKjfqhhYJGW9wz9..."
}
```

**Response:** Transaction response

### Configure Yield

```http
POST /yield/configure
Content-Type: application/json

{
  "user_pubkey": "5yWWZKjfqhhYJGW9wz9...",
  "enabled": true
}
```

**Response:** Transaction response

### Get Yield Info

```http
GET /yield/info/:user_pubkey
```

**Response:**
```json
{
  "vault_address": "8xYYZKjfqhhYJGW9wz9...",
  "yield_enabled": true,
  "total_yield_earned": 150000000,
  "last_yield_compound": 1705075200,
  "estimated_next_yield": 5000000,
  "time_until_next_compound": 3200
}
```

### Sync Yield Transaction

```http
POST /yield/sync
Content-Type: application/json

{
  "user_pubkey": "5yWWZKjfqhhYJGW9wz9...",
  "signature": "3K1c7vx..."
}
```

**Response:**
```json
{
  "vault": { ... },
  "success": true
}
```

## MFA Operations

### Setup MFA

Generates a QR code and secret for TOTP setup. This is step 1 of a two-step process.

```http
POST /mfa/setup
Content-Type: application/json

{
  "vault_address": "8xYYZKjfqhhYJGW9wz9..."
}
```

**Response:**
```json
{
  "qr_code_svg": "<svg>...</svg>",
  "secret": "JBSWY3DPEHPK3PXP",
  "backup_codes": ["1234-5678", "2345-6789", "3456-7890", "4567-8901", "5678-9012"]
}
```

### Verify MFA Setup

Verifies the TOTP code and enables MFA. This is step 2 of the setup process.

```http
POST /mfa/verify-setup
Content-Type: application/json

{
  "vault_address": "8xYYZKjfqhhYJGW9wz9...",
  "secret": "JBSWY3DPEHPK3PXP",
  "code": "123456"
}
```

**Response:**
```json
{
  "success": true,
  "backup_codes": ["1234-5678", "2345-6789", ...]
}
```

If verification fails:
```json
{
  "success": false,
  "backup_codes": null
}
```

### Disable MFA

Requires a valid MFA code or backup code to disable.

```http
POST /mfa/disable
Content-Type: application/json

{
  "vault_address": "8xYYZKjfqhhYJGW9wz9...",
  "code": "123456"
}
```

**Response:**
```json
{
  "success": true,
  "message": "MFA disabled successfully"
}
```

### Check MFA

Verify an MFA code without enabling/disabling.

```http
POST /mfa/check
Content-Type: application/json

{
  "vault_address": "8xYYZKjfqhhYJGW9wz9...",
  "code": "123456"
}
```

**Response:**
```json
{
  "valid": true
}
```

### Get MFA Status

```http
GET /mfa/status/:vault_address
```

**Response:**
```json
{
  "mfa_enabled": true,
  "vault_address": "8xYYZKjfqhhYJGW9wz9..."
}
```

## WebSocket API

Connect to `ws://localhost:8080/ws` for real-time updates.

### Connection

```javascript
const ws = new WebSocket('ws://localhost:8080/ws');

ws.onopen = () => {
  console.log('Connected to WebSocket');
};

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  console.log('Received:', message);
};
```

### Message Types

#### Subscribe to Vault Updates

```json
{
  "type": "Subscribe",
  "vault_address": "8xYYZKjfqhhYJGW9wz9..."
}
```

#### Unsubscribe from Vault Updates

```json
{
  "type": "Unsubscribe",
  "vault_address": "8xYYZKjfqhhYJGW9wz9..."
}
```

### Server Messages

#### Connection Confirmation

```json
{
  "type": "connected",
  "message": "Connected to Collateral Vault WebSocket"
}
```

#### Balance Update

```json
{
  "type": "BalanceUpdate",
  "vault_address": "8xYYZKjfqhhYJGW9wz9...",
  "balance": 1000000000
}
```

#### Transaction Notification

```json
{
  "type": "TransactionNotification",
  "vault_address": "8xYYZKjfqhhYJGW9wz9...",
  "tx_type": "deposit",
  "amount": 1000000000
}
```

**Note:** WebSocket support is currently stubbed out. Full implementation requires a pubsub system (Redis) for multi-instance deployments.

## Error Responses

All errors follow this format:

```json
{
  "error": "ErrorType",
  "message": "Human-readable error message"
}
```

### Common Error Types

- `InvalidAmount`: Invalid amount or pubkey format
- `SolanaClient`: Solana RPC error
- `Database`: Database operation failed
- `Internal`: Internal server error
- `VaultNotFound`: Vault does not exist
- `InsufficientBalance`: Not enough balance for operation

### HTTP Status Codes

- `200 OK`: Success
- `400 Bad Request`: Invalid request parameters
- `404 Not Found`: Resource not found
- `429 Too Many Requests`: Rate limit exceeded
- `500 Internal Server Error`: Server error

## Transaction Flow

### Typical Deposit Flow

1. **Build Transaction**
   ```http
   POST /vault/deposit
   { "user_pubkey": "...", "amount": 1000000000 }
   ```

2. **Sign Transaction**
   - Client receives unsigned transaction
   - User signs with wallet (Phantom, Solflare, etc.)

3. **Submit to Solana**
   - Client submits signed transaction to Solana network
   - Wait for confirmation

4. **Sync with Database**
   ```http
   POST /vault/sync
   {
     "user_pubkey": "...",
     "signature": "...",
     "transaction_type": "deposit",
     "amount": 1000000000
   }
   ```

### Typical Withdrawal Flow

Same as deposit, but use `/vault/withdraw` instead of `/vault/deposit`.

## Best Practices

1. **Always sync after transactions**: Call `/vault/sync` after submitting transactions to keep database in sync
2. **Handle rate limits**: Check `X-RateLimit-Remaining` header and implement exponential backoff
3. **Validate amounts**: Ensure amounts are in smallest token unit (6 decimals for USDT)
4. **Error handling**: Always check for error responses and handle appropriately
5. **WebSocket reconnection**: Implement automatic reconnection logic for WebSocket connections
6. **Transaction timeouts**: Set appropriate timeouts for Solana transaction submission

