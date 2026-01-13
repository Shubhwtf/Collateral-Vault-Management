#!/usr/bin/env bash

set -euo pipefail

# Simple Solana dev setup helper for this project
# - Creates or reuses a local dev wallet (payer / mint authority)
# - Ensures Solana is pointed at devnet (by default)
# - Airdrops SOL for fees
# - Creates an SPL token mint (USDT-like) if one does not exist
# - Creates an associated token account for a target wallet
# - Mints tokens into the target wallet's token account
#
# Usage:
#   ./setup-solana-dev.sh                 # mints to the local Solana CLI wallet
#   ./setup-solana-dev.sh <WALLET_PUBKEY> # mints to a specific wallet (e.g. Phantom)
#
# Requirements:
# - solana CLI installed and on PATH
# - spl-token CLI installed (usually comes with Solana tools)
#
# This script is safe to re-run; it will reuse existing keypairs and accounts
# when possible.

RPC_URL_DEFAULT="https://api.devnet.solana.com"
KEYPAIR_PATH_DEFAULT="$HOME/.config/solana/id.json"
MINT_KEYPAIR_DEFAULT="./.dev/dev-usdt-mint-keypair.json"
MINT_AMOUNT_DEFAULT="1000000000000" # 1,000,000 tokens with 6 decimals
AIR_DROP_SOL_DEFAULT="2"

RPC_URL="${SOLANA_RPC_URL:-$RPC_URL_DEFAULT}"
KEYPAIR_PATH="${SOLANA_KEYPAIR_PATH:-$KEYPAIR_PATH_DEFAULT}"
MINT_KEYPAIR="${DEV_USDT_MINT_KEYPAIR:-$MINT_KEYPAIR_DEFAULT}"
MINT_AMOUNT="${DEV_USDT_MINT_AMOUNT:-$MINT_AMOUNT_DEFAULT}"
AIR_DROP_SOL="${DEV_SOL_AIRDROP_AMOUNT:-$AIR_DROP_SOL_DEFAULT}"

echo "=== Collateral Vault Management System - Solana Dev Setup ==="
echo "RPC URL:          $RPC_URL"
echo "Payer keypair:    $KEYPAIR_PATH"
echo "Mint keypair:     $MINT_KEYPAIR"
if [ "${1-}" != "" ]; then
  echo "Target wallet:    $1"
else
  echo "Target wallet:    (Solana CLI default wallet)"
fi
echo

command -v solana >/dev/null 2>&1 || { echo "solana CLI not found on PATH"; exit 1; }
command -v spl-token >/dev/null 2>&1 || { echo "spl-token CLI not found on PATH"; exit 1; }

echo "-> Configuring Solana CLI..."
solana config set --url "$RPC_URL" >/dev/null
solana config set --keypair "$KEYPAIR_PATH" >/dev/null

if [ ! -f "$KEYPAIR_PATH" ]; then
  echo "-> Generating new payer wallet keypair at $KEYPAIR_PATH ..."
  mkdir -p "$(dirname "$KEYPAIR_PATH")"
  solana-keygen new --no-bip39-passphrase --outfile "$KEYPAIR_PATH"
else
  echo "-> Using existing payer wallet keypair at $KEYPAIR_PATH"
fi

PAYER_WALLET_PUBKEY="$(solana-keygen pubkey "$KEYPAIR_PATH")"
if [ "${1-}" != "" ]; then
  TARGET_WALLET_PUBKEY="$1"
else
  TARGET_WALLET_PUBKEY="$PAYER_WALLET_PUBKEY"
fi

echo "Payer wallet public key:  $PAYER_WALLET_PUBKEY"
echo "Target wallet public key: $TARGET_WALLET_PUBKEY"

echo "-> Checking wallet balance..."
CURRENT_BALANCE_OUTPUT=$(solana balance "$PAYER_WALLET_PUBKEY" 2>/dev/null || echo "0 SOL")
CURRENT_BALANCE_SOL=$(echo "$CURRENT_BALANCE_OUTPUT" | grep -oE '[0-9]+\.[0-9]+' | head -1 || echo "0")

# Check if balance is less than 1 SOL (using awk for floating point comparison)
NEEDS_AIRDROP=$(echo "$CURRENT_BALANCE_SOL" | awk '{if ($1 < 1.0) print "1"; else print "0"}')

if [ "$NEEDS_AIRDROP" = "1" ]; then
  echo "-> Current balance: ${CURRENT_BALANCE_SOL} SOL (low, requesting airdrop of ${AIR_DROP_SOL} SOL)..."
  set +e
  solana airdrop "$AIR_DROP_SOL" "$PAYER_WALLET_PUBKEY"
  set -e
  echo "-> Updated balance:"
  solana balance
else
  echo "-> Current balance: ${CURRENT_BALANCE_SOL} SOL (sufficient, skipping airdrop)"
fi

if [ -f "$MINT_KEYPAIR" ]; then
  echo "-> Using existing dev mint keypair at $MINT_KEYPAIR"
  MINT_ADDRESS="$(solana-keygen pubkey "$MINT_KEYPAIR")"
else
  echo "-> Generating new mint keypair..."
  mkdir -p "$(dirname "$MINT_KEYPAIR")"
  solana-keygen new --no-bip39-passphrase --outfile "$MINT_KEYPAIR"
  echo "-> Creating new SPL token mint..."
  spl-token create-token --decimals 6 --fee-payer "$KEYPAIR_PATH" "$MINT_KEYPAIR"
  MINT_ADDRESS="$(solana-keygen pubkey "$MINT_KEYPAIR")"
fi

echo "Mint address: $MINT_ADDRESS"

echo "-> Ensuring associated token account exists for target wallet..."
TOKEN_ACCOUNT_ADDRESS="$(spl-token create-account "$MINT_ADDRESS" --owner "$TARGET_WALLET_PUBKEY" --fee-payer "$KEYPAIR_PATH" 2>&1 | awk '/Creating account/{print $3} /Signature/ {next} /Error/ {exit 1}')"

if [ -z "$TOKEN_ACCOUNT_ADDRESS" ] || [ "$TOKEN_ACCOUNT_ADDRESS" = "" ]; then
  # If create-account reports it already exists, fetch it explicitly
  TOKEN_ACCOUNT_ADDRESS="$(spl-token accounts --owner "$TARGET_WALLET_PUBKEY" 2>/dev/null | awk -v mint="$MINT_ADDRESS" '$1==mint {print $3; exit}')"
  if [ -n "$TOKEN_ACCOUNT_ADDRESS" ]; then
    echo "-> Token account already exists: $TOKEN_ACCOUNT_ADDRESS"
  fi
fi

if [ -z "$TOKEN_ACCOUNT_ADDRESS" ] || [ "$TOKEN_ACCOUNT_ADDRESS" = "" ]; then
  # Try to get the associated token account address
  TOKEN_ACCOUNT_ADDRESS="$(spl-token address --token "$MINT_ADDRESS" --owner "$TARGET_WALLET_PUBKEY" 2>/dev/null || echo "")"
fi

echo "Token account address: $TOKEN_ACCOUNT_ADDRESS"

echo "-> Minting tokens to target wallet token account..."
spl-token mint "$MINT_ADDRESS" "$MINT_AMOUNT" "$TOKEN_ACCOUNT_ADDRESS" --fee-payer "$KEYPAIR_PATH"

echo
echo "=== Setup Complete ==="
echo "Payer wallet:       $PAYER_WALLET_PUBKEY"
echo "Target wallet:      $TARGET_WALLET_PUBKEY"
echo "Mint:               $MINT_ADDRESS"
echo "Token account:      $TOKEN_ACCOUNT_ADDRESS"
echo "Mint amount:        $MINT_AMOUNT (in base units, 6 decimals)"
echo
echo "You can now use these values in your backend .env:"
echo "  PAYER_KEYPAIR_PATH=$KEYPAIR_PATH"
echo "  USDT_MINT=$MINT_ADDRESS"
echo "  SOLANA_RPC_URL=$RPC_URL"


