# FlashPool v2 — Mainnet Deployment Guide

## Pre-Launch Checklist

### Security

- [ ] Full contract audit by at least one independent auditor
- [ ] Verify all `require_auth()` calls are in place
- [ ] Review admin key management — use hardware wallet or multisig
- [ ] Test all functions on testnet with real user flows
- [ ] Verify vesting math with edge cases (0 elapsed, max elapsed, rounding)
- [ ] Verify re-join accumulation doesn't lose LP
- [ ] Check Treasury approval amount is appropriate for launch period

### Infrastructure

- [ ] Set up mainnet Soroban RPC endpoint (or use public nodes)
- [ ] Configure frontend with mainnet contract IDs
- [ ] Set up monitoring for contract events (CampaignCreated, UserJoined, RewardClaimed)
- [ ] Set up TTL monitoring — contracts need periodic `extend_ttl` to stay alive

### Token Setup

- [ ] Determine total FLASH supply for mainnet
- [ ] Create FLASH as Stellar classic asset, wrap as SAC
- [ ] Use existing mainnet USDC (Circle's USDC on Stellar, or other stablecoin)
- [ ] Decide initial FLASH/USDC price ratio for TinyAMM seeding

## Deployment Steps

### 1. Generate Mainnet Admin Key

```bash
# Generate a new key — DO NOT reuse testnet keys
stellar keys generate flashpool-mainnet-admin --network mainnet

# IMPORTANT: Back up the secret key securely
# Consider using a hardware wallet or multisig for production
```

### 2. Fund the Admin Account

Transfer XLM to the admin address for transaction fees. Need at minimum 100 XLM for deployment + operations.

### 3. Create and Wrap FLASH Token

```bash
ADMIN_PK=$(stellar keys address flashpool-mainnet-admin)

# Deploy FLASH as SAC
stellar contract asset deploy \
  --asset "FLASH:$ADMIN_PK" \
  --source-account flashpool-mainnet-admin \
  --network mainnet
```

### 4. Get Mainnet USDC Address

Use the official USDC on Stellar (issued by Circle) or wrap an existing stablecoin. The USDC SAC address on mainnet depends on the issuer.

```bash
# If wrapping your own test USDC (not recommended for production):
# stellar contract asset deploy --asset "USDC:$ADMIN_PK" --network mainnet

# For production, use the official USDC SAC address
```

### 5. Build Contracts for Mainnet

```bash
cd contracts

# Build with release profile (already optimized: opt-level=z, lto, strip)
cargo build --release -p flash-treasury -p flash-campaign-manager-v2 --target wasm32v1-none

# Build TinyAMM
cd /path/to/tinyamm/contracts
cargo build --release --target wasm32v1-none
```

### 6. Deploy TinyAMM

```bash
TINYAMM_WASM=/path/to/tinyamm/target/wasm32v1-none/release/tinyamm_contract.wasm

TINYAMM_ID=$(stellar contract deploy \
  --wasm $TINYAMM_WASM \
  --source-account flashpool-mainnet-admin \
  --network mainnet)

# Initialize with FLASH/USDC, 30bps fee
stellar contract invoke \
  --id $TINYAMM_ID \
  --source-account flashpool-mainnet-admin \
  --network mainnet \
  -- init \
  --token_a $FLASH_ID \
  --token_b $USDC_ID \
  --fee_bps 30
```

### 7. Seed TinyAMM with Initial Liquidity

This sets the initial FLASH/USDC price. Choose amounts carefully — this determines the starting price.

```bash
# Example: 100,000 FLASH + 10,000 USDC = 1 FLASH = $0.10
# Transfer tokens to TinyAMM
stellar contract invoke --id $FLASH_ID -- transfer \
  --from $ADMIN_PK --to $TINYAMM_ID --amount <FLASH_AMOUNT>

stellar contract invoke --id $USDC_ID -- transfer \
  --from $ADMIN_PK --to $TINYAMM_ID --amount <USDC_AMOUNT>

# Add liquidity
stellar contract invoke --id $TINYAMM_ID -- add_liquidity \
  --to $ADMIN_PK \
  --amount0_desired <USDC_AMOUNT> \
  --amount1_desired <FLASH_AMOUNT> \
  --amount0_min 0 \
  --amount1_min 0
```

### 8. Deploy Treasury

```bash
TREASURY_WASM=contracts/target/wasm32v1-none/release/flash_treasury.wasm

TREASURY_ID=$(stellar contract deploy \
  --wasm $TREASURY_WASM \
  --source-account flashpool-mainnet-admin \
  --network mainnet)

# Initialize
stellar contract invoke --id $TREASURY_ID -- initialize \
  --admin $ADMIN_PK --flash $FLASH_ID

# Fund with FLASH supply
stellar contract invoke --id $FLASH_ID -- transfer \
  --from $ADMIN_PK --to $TREASURY_ID --amount <FLASH_SUPPLY>
```

### 9. Deploy Campaign Manager

```bash
CM_WASM=contracts/target/wasm32v1-none/release/flash_campaign_manager_v2.wasm

CM_ID=$(stellar contract deploy \
  --wasm $CM_WASM \
  --source-account flashpool-mainnet-admin \
  --network mainnet)

# Initialize — use a real Hoops router address on mainnet
stellar contract invoke --id $CM_ID -- initialize \
  --admin $ADMIN_PK \
  --flash $FLASH_ID \
  --usdc $USDC_ID \
  --treasury $TREASURY_ID \
  --tinyamm $TINYAMM_ID \
  --hoops_router <HOOPS_ROUTER_MAINNET_ADDRESS>
```

### 10. Approve Campaign Manager

```bash
# Allow campaign manager to pull FLASH from treasury for bonus rewards
stellar contract invoke --id $TREASURY_ID -- approve_spender \
  --spender $CM_ID --amount <ALLOWANCE_AMOUNT>
```

### 11. Configure Frontend

Create `.env.local` with mainnet values:

```
NEXT_PUBLIC_CAMPAIGN_MANAGER_ID=<CM_ID>
NEXT_PUBLIC_TREASURY_ID=<TREASURY_ID>
NEXT_PUBLIC_TINYAMM_ID=<TINYAMM_ID>
NEXT_PUBLIC_FLASH_TOKEN_ID=<FLASH_ID>
NEXT_PUBLIC_USDC_TOKEN_ID=<USDC_ID>
NEXT_PUBLIC_SOROBAN_RPC_URL=https://soroban.stellar.org
NEXT_PUBLIC_NETWORK_PASSPHRASE=Public Global Stellar Network ; September 2015
```

### 12. Deploy Frontend

```bash
npm run build
# Deploy to your hosting provider (Vercel, Netlify, etc.)
```

## Post-Launch Operations

### TTL Management

Soroban contracts expire if their TTL isn't extended. Set up a cron job or monitoring:

```bash
# Extend TTL for all contracts (run weekly)
stellar contract extend --id $TREASURY_ID --durability persistent --ledgers-to-extend 500000
stellar contract extend --id $CM_ID --durability persistent --ledgers-to-extend 500000
stellar contract extend --id $CM_ID --durability instance --ledgers-to-extend 500000
```

### Monitoring

Watch for events to track protocol activity:

```bash
# Get recent events from Campaign Manager
stellar events --id $CM_ID --network mainnet --start-ledger <RECENT_LEDGER>
```

### Treasury Management

```bash
# Check Treasury balance
stellar contract invoke --id $FLASH_ID -- balance --id $TREASURY_ID

# Check Campaign Manager allowance
stellar contract invoke --id $TREASURY_ID -- get_allowance --spender $CM_ID

# Top up allowance if needed
stellar contract invoke --id $TREASURY_ID -- approve_spender \
  --spender $CM_ID --amount <NEW_AMOUNT>

# Emergency withdraw (admin only)
stellar contract invoke --id $TREASURY_ID -- withdraw \
  --to $ADMIN_PK --amount <AMOUNT>
```

### Updating Protocol Defaults

```bash
# Change gamma, vesting period, or surplus_bps
stellar contract invoke --id $CM_ID -- set_defaults \
  --admin $ADMIN_PK \
  --defaults '{"gamma":2,"vesting_period":100800,"surplus_bps":500,"ttl_thresh":172800,"ttl_bump":241920}'
```

## Differences from Testnet

| Aspect | Testnet | Mainnet |
|--------|---------|---------|
| USDC | Self-issued SAC token | Official Circle USDC on Stellar |
| Hoops Router | Admin placeholder | Real Hoops router address |
| Admin key | Local file | Hardware wallet / multisig |
| FLASH supply | Unlimited (issuer can mint) | Fixed, controlled by Treasury |
| RPC | soroban-testnet.stellar.org | soroban.stellar.org (or private) |
| Network passphrase | Test SDF Network | Public Global Stellar Network |

## Upgrade Path

The current contracts have no upgrade mechanism. To upgrade:

1. Deploy new contract version
2. Migrate state (if needed) via admin functions
3. Update Treasury to approve new contract
4. Revoke old contract's approval
5. Update frontend with new contract IDs

For v1.1 features (rolling campaigns, compound, etc.), new contract deployments will be needed.
