#!/bin/bash
set -e

# FlashPool v2 Deployment Script
# Prerequisites: soroban CLI installed, funded account on target network
#
# Usage: ADMIN_SECRET=S... ./scripts/deploy.sh [testnet|mainnet]

NETWORK="${1:-testnet}"
ADMIN_SECRET="${ADMIN_SECRET:?Set ADMIN_SECRET env var}"

echo "Deploying FlashPool v2 to $NETWORK..."
echo ""

# 1. Deploy FLASH token (Stellar Asset Contract)
echo "Step 1: Deploy FLASH SAC..."
# soroban contract asset deploy --asset FLASH:$ADMIN_PUBLIC --network $NETWORK

# 2. Get USDC address (use existing on network)
echo "Step 2: USDC address..."
# USDC_ADDRESS=<existing USDC contract on network>

# 3. Deploy TinyAMM
echo "Step 3: Deploy TinyAMM..."
# TINYAMM_ID=$(soroban contract deploy --wasm tinyamm.wasm --network $NETWORK --source $ADMIN_SECRET)
# soroban contract invoke --id $TINYAMM_ID --fn init -- --token_a $FLASH --token_b $USDC --fee_bps 30

# 4. Seed TinyAMM liquidity
echo "Step 4: Seed TinyAMM..."
# soroban contract invoke --id $TINYAMM_ID --fn add_liquidity -- ...

# 5. Deploy Treasury
echo "Step 5: Deploy Treasury..."
# TREASURY_ID=$(soroban contract deploy --wasm flash_treasury.wasm --network $NETWORK --source $ADMIN_SECRET)
# soroban contract invoke --id $TREASURY_ID --fn initialize -- --admin $ADMIN_PUBLIC --flash $FLASH

# 6. Fund Treasury with FLASH
echo "Step 6: Fund Treasury..."
# soroban contract invoke --id $FLASH --fn transfer -- --from $ADMIN_PUBLIC --to $TREASURY_ID --amount 1000000000

# 7. Deploy Campaign Manager
echo "Step 7: Deploy Campaign Manager..."
# CM_ID=$(soroban contract deploy --wasm flash_campaign_manager_v2.wasm --network $NETWORK --source $ADMIN_SECRET)
# soroban contract invoke --id $CM_ID --fn initialize -- ...

# 8. Treasury: approve campaign manager
echo "Step 8: Approve Campaign Manager..."
# soroban contract invoke --id $TREASURY_ID --fn approve_spender -- --spender $CM_ID --amount 500000000

echo ""
echo "Deployment complete!"
echo "TinyAMM:          ${TINYAMM_ID:-<not set>}"
echo "Treasury:         ${TREASURY_ID:-<not set>}"
echo "Campaign Manager: ${CM_ID:-<not set>}"
