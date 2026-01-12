# Collateral Vault Management System

A non-custodial collateral management system for decentralized perpetual futures exchanges on Solana. Built with Rust, Anchor, and PostgreSQL.

![Status](https://img.shields.io/badge/status-MVP%20complete-blue)
[![Test Status](https://img.shields.io/badge/tests-52%2F52%20passing-brightgreen)](test-results/001-test-coverage-report.md)
[![Security](https://img.shields.io/badge/security-0%20critical-green)](test-results/002-security-test-results.md)
[![Anchor](https://img.shields.io/badge/anchor-0.32.1-blue)](https://www.anchor-lang.com/)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)

## Executive Summary

The Collateral Vault Management System is a non-custodial solution for managing user collateral on Solana-based perpetual futures exchanges. Users maintain full control of their funds through PDA-based vaults, while trading programs can securely lock and unlock collateral via Cross-Program Invocations (CPI). The system provides comprehensive analytics, real-time balance tracking, and production-ready features including MFA, yield generation, and rate limiting.

**Why**: Traditional custodial solutions require users to trust a central entity with their funds. This system eliminates that trust requirement by using Solana's program-derived addresses (PDAs) and smart contracts, ensuring users always control their collateral while enabling seamless integration with trading platforms.

## 🚀 Features

- **Secure Vault Management**: PDA-based vaults for user collateral (USDT)
- **Non-Custodial**: Users maintain full control of their funds
- **Deposit/Withdrawal**: Safe transfer of funds with comprehensive validation
- **Collateral Locking**: Real-time tracking of locked vs available balances
- **Cross-Program Invocations (CPI)**: Seamless integration with trading programs
- **Advanced Analytics**: TVL tracking, user distribution, flow metrics
- **Transaction History**: Complete audit trail with PostgreSQL
- **REST & WebSocket APIs**: Real-time balance updates and notifications
- **Multi-Factor Authentication (MFA)**: TOTP-based security for vault operations
- **Yield Generation**: Automatic yield compounding for vault balances
- **Production-Ready**: Error handling, logging, and monitoring

## 🏗️ Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        Web Frontend                              │
│                  (React + TypeScript + Vite)                     │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  Wallet Integration (Phantom/Solflare/Sollet)            │  │
│  │  Pages: Dashboard, Vault, Transactions, Analytics, Yield │  │
│  └──────────────────────────────────────────────────────────┘  │
└────────────────────────────┬────────────────────────────────────┘
                             │ HTTPS/WebSocket
                    ┌────────▼────────┐
                    │   REST API      │
                    │   WebSocket API │
                    └────────┬────────┘
                             │
        ┌────────────────────▼────────────────────┐
        │         Rust Backend Service            │
        │  - Vault Manager                        │
        │  - Analytics Engine                     │
        │  - MFA Service                         │
        │  - Snapshot Service                     │
        └────────┬──────────────────┬─────────────┘
                 │                  │
        ┌────────▼────────┐  ┌──────▼──────────┐
        │  Solana Network │  │   PostgreSQL    │
        │  ┌───────────┐  │  │   (Neon.tech)   │
        │  │  Anchor   │  │  │  - Vaults       │
        │  │  Program  │  │  │  - Transactions │
        │  │  - Vault  │  │  │  - Analytics    │
        │  │  - Deposit│  │  │  - MFA Data     │
        │  │  - Withdraw│ │  │  - Snapshots    │
        │  │  - Lock   │  │  └─────────────────┘
        │  │  - Unlock │  │
        │  │  - Yield  │  │
        │  └───────────┘  │
        │  ┌───────────┐  │
        │  │ SPL Token │  │
        │  │  Program  │  │
        │  │  (CPI)    │  │
        │  └───────────┘  │
        └─────────────────┘
```

## 🔑 PDA Derivation Scheme

Program-Derived Addresses (PDAs) are used to create deterministic, non-custodial vault addresses. Each user has exactly one vault, derived from their wallet address.

### Vault PDA

**Seeds:** `["vault", owner_pubkey]`

```rust
let (vault_pda, bump) = Pubkey::find_program_address(
    &[b"vault", owner_pubkey.as_ref()],
    &program_id
);
```

**Properties:**
- One vault per user (deterministic)
- Vault PDA owns the associated token account
- No private key needed (PDA can sign via seeds)
- Bump seed stored in vault account for signing

**Example:**
- Owner: `5yWWZKjfqhhYJGW9wz9...`
- Seeds: `["vault", <owner_bytes>]`
- Result: Deterministic vault address for this owner

### Authority PDA

**Seeds:** `["vault_authority"]`

```rust
let (authority_pda, bump) = Pubkey::find_program_address(
    &[b"vault_authority"],
    &program_id
);
```

**Purpose:**
- Stores list of authorized programs that can lock/unlock collateral
- Managed by admin
- Prevents unauthorized programs from accessing user funds

### Vault Token Account (ATA)

**Derivation:** Associated Token Account for vault PDA

```rust
let vault_token_account = get_associated_token_address(
    &vault_pda,
    &usdt_mint
);
```

**Properties:**
- Owned by vault PDA (not user)
- Holds USDT tokens for the vault
- Authority is the vault PDA (enables PDA signing for withdrawals)

### Complete Derivation Flow

```
User Wallet (Owner)
    │
    ├─→ Vault PDA
    │   Seeds: ["vault", owner_pubkey]
    │   │
    │   └─→ Vault Token Account (ATA)
    │       Derived from: vault_pda + usdt_mint
    │       Authority: vault_pda
    │
    └─→ Authority PDA (Global)
        Seeds: ["vault_authority"]
        Stores: authorized_programs list
```

## 🔄 CPI Flow Diagram

Cross-Program Invocations (CPIs) enable the vault program to interact with the SPL Token Program for token transfers. The flow differs based on whether the user or the vault PDA is the authority.

### Deposit Flow (User Authority)

```
User Wallet
    │
    ├─→ Calls: deposit(amount)
    │   │
    │   └─→ Collateral Vault Program
    │       │
    │       ├─→ Validates: amount > 0
    │       │
    │       └─→ CPI: SPL Token Program
    │           │
    │           ├─→ Instruction: transfer
    │           ├─→ From: user_token_account (User ATA)
    │           ├─→ To: vault_token_account (Vault ATA)
    │           └─→ Authority: user (signs transaction)
    │
    └─→ Updates: vault.total_balance += amount
        vault.available_balance += amount
        vault.total_deposited += amount
```

**Key Points:**
- User signs the transaction (authority for their token account)
- Tokens move from user's ATA to vault's ATA
- Vault state updated after successful CPI

### Withdraw Flow (PDA Authority)

```
User Wallet
    │
    ├─→ Calls: withdraw(amount)
    │   │
    │   └─→ Collateral Vault Program
    │       │
    │       ├─→ Validates: available_balance >= amount
    │       │
    │       ├─→ Derives PDA Signer:
    │       │   Seeds: ["vault", owner_pubkey, bump]
    │       │
    │       └─→ CPI: SPL Token Program
    │           │
    │           ├─→ Instruction: transfer
    │           ├─→ From: vault_token_account (Vault ATA)
    │           ├─→ To: user_token_account (User ATA)
    │           └─→ Authority: vault_pda (signs via seeds)
    │
    └─→ Updates: vault.total_balance -= amount
        vault.available_balance -= amount
        vault.total_withdrawn += amount
```

**Key Points:**
- Vault PDA signs the CPI (no private key needed)
- Seeds: `["vault", owner_pubkey, bump]`
- Tokens move from vault's ATA to user's ATA
- Only available balance can be withdrawn (locked funds protected)

### Lock/Unlock Flow (State-Only, No CPI)

```
Authorized Trading Program
    │
    ├─→ CPI: lock_collateral(amount)
    │   │
    │   └─→ Collateral Vault Program
    │       │
    │       ├─→ Validates: caller_program in authorized_programs
    │       ├─→ Validates: available_balance >= amount
    │       │
    │       └─→ Updates State (No Token Transfer):
    │           vault.available_balance -= amount
    │           vault.locked_balance += amount
    │           vault.total_balance unchanged
    │
    └─→ Later: unlock_collateral(amount)
        └─→ Reverses the lock operation
```

**Key Points:**
- No token transfer (tokens stay in vault ATA)
- Only authorized programs can lock/unlock
- Moves balance between available and locked states
- Used for trading positions (collateral backing open positions)

### Transfer Flow (Cross-Vault)

```
Authorized Program
    │
    ├─→ CPI: transfer_collateral(amount, from_vault, to_vault)
    │   │
    │   └─→ Collateral Vault Program
    │       │
    │       ├─→ Validates: caller_program authorized
    │       ├─→ Validates: from_vault.total_balance >= amount
    │       │
    │       ├─→ Derives From Vault PDA Signer:
    │       │   Seeds: ["vault", from_owner, bump]
    │       │
    │       └─→ CPI: SPL Token Program
    │           │
    │           ├─→ Instruction: transfer
    │           ├─→ From: from_vault_token_account
    │           ├─→ To: to_vault_token_account
    │           └─→ Authority: from_vault_pda (signs via seeds)
    │
    └─→ Updates Both Vaults:
        from_vault: subtract amount
        to_vault: add amount
```

**Key Points:**
- Used for liquidations, cross-vault operations
- From vault PDA signs the transfer
- Both vault states updated atomically

### Complete CPI Flow Summary

```
┌─────────────────────────────────────────────────────────────┐
│                    CPI Flow Types                            │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  DEPOSIT:                                                    │
│  User → Vault Program → SPL Token (User Authority)         │
│  [User Token Account] → [Vault Token Account]              │
│                                                              │
│  WITHDRAW:                                                   │
│  User → Vault Program → SPL Token (PDA Authority)            │
│  [Vault Token Account] → [User Token Account]              │
│  Seeds: ["vault", owner, bump]                              │
│                                                              │
│  LOCK/UNLOCK:                                                │
│  Trading Program → Vault Program (State Only)               │
│  available_balance ↔ locked_balance                         │
│  (No token transfer, tokens stay in vault)                  │
│                                                              │
│  TRANSFER:                                                   │
│  Authorized Program → Vault Program → SPL Token (PDA)       │
│  [Vault A] → [Vault B]                                       │
│  Seeds: ["vault", from_owner, bump]                          │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## 🚀 Quick Start (Docker Compose)

The fastest way to get started is using Docker Compose:

```bash
# Clone the repository
git clone https://github.com/Shubhwtf/Collateral-Vault-Management?
cd collateral-vault-management-system

# Start all services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop all services
docker-compose down
```

Services will be available at:
- **Frontend**: `http://localhost:3000`
- **Backend**: `http://localhost:8080`
- **PostgreSQL**: `localhost:5432`

## 🛠️ Installation & Basic Setup

### Prerequisites

- **Rust** 1.75 or later
- **Solana CLI** 1.16 or later
- **Anchor** 0.32.1
- **Node.js** 18+ and npm/yarn
- **PostgreSQL** 14+ (or Neon.tech account)
- **Docker** and **Docker Compose** (optional, but recommended)

### Installation Steps

1. **Install Rust**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup component add rustfmt clippy
```

2. **Install Solana CLI**
```bash
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
```

3. **Install Anchor**
```bash
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install 0.32.1
avm use 0.32.1
```

4. **Set Up Solana Wallet**
```bash
solana-keygen new --outfile ~/.config/solana/id.json
solana config set --url devnet
solana airdrop 2
```

5. **Build & Deploy Smart Contract**
```bash
cd program
anchor build
anchor test  # Runs on localnet automatically
```

6. **Set Up Database**

**Option A: Docker Compose (Recommended)**
```bash
docker-compose up -d postgres
# Migrations run automatically on startup
```

**Option B: Neon.tech**
- Sign up at https://neon.tech
- Create a new project
- Copy the connection string

7. **Configure Backend**
```bash
cd backend
cp .env.example .env
# Edit .env with your configuration:
# - PROGRAM_ID: Your deployed program ID
# - PAYER_KEYPAIR_PATH: Path to Solana keypair
# - DATABASE_URL: PostgreSQL connection string
# - USDT_MINT: Devnet USDT address
# - SOLANA_RPC_URL: Solana RPC endpoint
```

8. **Install Frontend Dependencies**
```bash
cd frontend
npm install
```

## 💡 Basic Usage Examples

### Initialize a Vault

```bash
# Via API
curl -X POST http://localhost:8080/vault/initialize \
  -H "Content-Type: application/json" \
  -d '{"user_pubkey": "YOUR_PUBKEY"}'

# Returns unsigned transaction - sign with wallet and submit
```

### Deposit Tokens

```bash
# Build deposit transaction
curl -X POST http://localhost:8080/vault/deposit \
  -H "Content-Type: application/json" \
  -d '{
    "user_pubkey": "YOUR_PUBKEY",
    "amount": 1000000000
  }'

# Sign transaction with wallet, submit to Solana, then sync:
curl -X POST http://localhost:8080/vault/sync \
  -H "Content-Type: application/json" \
  -d '{
    "user_pubkey": "YOUR_PUBKEY",
    "signature": "TRANSACTION_SIGNATURE",
    "transaction_type": "deposit",
    "amount": 1000000000
  }'
```

### Check Balance

```bash
curl http://localhost:8080/vault/balance/YOUR_PUBKEY
```

### Get Analytics

```bash
# TVL Overview
curl http://localhost:8080/analytics/overview

# TVL Chart Data
curl http://localhost:8080/analytics/chart/tvl
```

### Frontend Usage

1. Connect your Solana wallet (Phantom, Solflare, etc.)
2. Navigate to the Vault page
3. Click "Initialize Vault" if you haven't already
4. Deposit tokens by entering amount and confirming transaction
5. View your balance, transaction history, and analytics on the Dashboard

## 📚 API Documentation

For complete API documentation including all endpoints, request/response formats, authentication, rate limiting, and WebSocket API details, see:

**[Full API Documentation](docs/API.md)**

Key endpoints:
- `/vault/initialize` - Initialize a new vault
- `/vault/deposit` - Build deposit transaction
- `/vault/withdraw` - Build withdrawal transaction
- `/vault/balance/:user_pubkey` - Get vault balance
- `/analytics/overview` - Get analytics overview
- `/yield/compound` - Compound yield
- `/mfa/setup` - Setup MFA

## 🧪 Testing

### How to Run Tests

**Test Smart Contract:**
```bash
cd program
anchor test
```

**Test Backend:**
```bash
cd backend
cargo test
```

**Test Frontend:**
```bash
cd frontend
npm test
```

### Test Coverage

**Current Status:** ✅ **52/52 tests passing (100%)**

- **Backend**: 19/19 tests passing
- **Frontend**: 18/18 tests passing
- **Solana Program**: 15/15 tests passing

See [test-results/001-test-coverage-report.md](test-results/001-test-coverage-report.md) for detailed coverage.

## 🔒 Security Analysis

For comprehensive security analysis including security features, test results, best practices, and known considerations, see:

**[Security Analysis](test-results/002-security-test-results.md)**

## 🤝 Contributing

We welcome contributions! Please follow these guidelines:

### Development Workflow

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests (`anchor test`, `cargo test`, `npm test`)
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

### Code Style

- **Rust**: Follow `rustfmt` and `clippy` recommendations
- **TypeScript**: Follow ESLint configuration
- **Commit Messages**: Use conventional commits format
- **Tests**: Write tests for all new features

### Testing Requirements

- All new features must include tests
- Maintain 100% test pass rate
- Update test documentation if needed

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

The MIT License is one of the most permissive open-source licenses, allowing you to use, modify, and distribute the code with minimal restrictions. The only requirement is that you include the original copyright notice and license text when redistributing.
