# FlashPool v2 — Testnet Deployment Guide

## Deployed Contracts (Stellar Testnet)

| Contract | Address | Explorer |
|----------|---------|----------|
| **FLASH Token (SAC)** | `CC64MIMYTVOY27745VMTBJAJ5G6HOIK42N2C2RYVXWGJCH46FCFTCFYL` | [View](https://stellar.expert/explorer/testnet/contract/CC64MIMYTVOY27745VMTBJAJ5G6HOIK42N2C2RYVXWGJCH46FCFTCFYL) |
| **USDC Token (SAC)** | `CCRKSTMLQFHUDJ6ORHYTBVETGFLJTJ7XLXITLAFOTGEZK3J6UMEDZ3JA` | [View](https://stellar.expert/explorer/testnet/contract/CCRKSTMLQFHUDJ6ORHYTBVETGFLJTJ7XLXITLAFOTGEZK3J6UMEDZ3JA) |
| **TinyAMM (FLASH/USDC)** | `CDA5NMKGGFVMSK753LHMCHTD6MXBBO7RAXFJUD3QJLEXXTX25K6E3UES` | [View](https://stellar.expert/explorer/testnet/contract/CDA5NMKGGFVMSK753LHMCHTD6MXBBO7RAXFJUD3QJLEXXTX25K6E3UES) |
| **Treasury** | `CDRPWM7JWXVF4PST3EHZFAGDWV5RVY5DV3TS4NGVQAZ6UC45CTKHTAVZ` | [View](https://stellar.expert/explorer/testnet/contract/CDRPWM7JWXVF4PST3EHZFAGDWV5RVY5DV3TS4NGVQAZ6UC45CTKHTAVZ) |
| **Campaign Manager v2** | `CBU2BI3VSNUCGBF25VWJB7SW75X4YNN22SWSY4W2IKG72XISSXTYNRJ2` | [View](https://stellar.expert/explorer/testnet/contract/CBU2BI3VSNUCGBF25VWJB7SW75X4YNN22SWSY4W2IKG72XISSXTYNRJ2) |

**Admin Account:** `GBA4CHQIICSVSPMOMAWZMT5IP6VC5CQO56B4LH3ZMMJAG4IDVVLGNYPR`

**Asset Issuer:** Same as admin (FLASH and USDC are classic wrapped SAC tokens issued by admin)

## Current State

- TinyAMM seeded: 10,000 USDC + 100,000 FLASH (1 FLASH = 0.1 USDC)
- Treasury funded: 500,000 FLASH
- Campaign Manager approved: 200,000 FLASH allowance from Treasury
- Frontend `.env.local` configured with all contract IDs

## How It Works

### Creating a Campaign

A campaign creator pays USDC to incentivize liquidity for a target pool:

```bash
stellar contract invoke \
  --id CBU2BI3VSNUCGBF25VWJB7SW75X4YNN22SWSY4W2IKG72XISSXTYNRJ2 \
  --source-account flashpool-admin \
  --network testnet \
  -- create_campaign \
  --fee_usdc 10000000000 \
  --adapter_id 3 \
  --pool_address <TARGET_POOL> \
  --lp_token <LP_TOKEN> \
  --duration 100000 \
  --target_lp 5000000000 \
  --bonus_flash 0 \
  --min_reward_flash 0 \
  --creator GBA4CHQIICSVSPMOMAWZMT5IP6VC5CQO56B4LH3ZMMJAG4IDVVLGNYPR
```

What happens internally:
1. USDC transferred from creator to Campaign Manager
2. Portion swapped USDC → FLASH via TinyAMM (buy pressure)
3. Remainder + FLASH added as liquidity to TinyAMM (deepens core pair)
4. Surplus FLASH = reward pool for the campaign

### Joining a Campaign

Users deposit a token into a campaign's target pool:

```bash
stellar contract invoke \
  --id CBU2BI3VSNUCGBF25VWJB7SW75X4YNN22SWSY4W2IKG72XISSXTYNRJ2 \
  --source-account <USER> \
  --network testnet \
  -- join_campaign \
  --id 1 \
  --amount 1000000000 \
  --token <TOKEN_ADDRESS> \
  --user <USER_PUBLIC_KEY>
```

What happens:
1. Token transferred from user to Campaign Manager
2. Half swapped for the other pair token via Hoops adapter
3. Both deposited as liquidity via Hoops adapter
4. User gets a rank (1st, 2nd, 3rd...) and weight

### Claiming Rewards

After the campaign ends, users claim vested FLASH rewards:

```bash
stellar contract invoke \
  --id CBU2BI3VSNUCGBF25VWJB7SW75X4YNN22SWSY4W2IKG72XISSXTYNRJ2 \
  --source-account <USER> \
  --network testnet \
  -- claim \
  --id 1 \
  --user <USER_PUBLIC_KEY>
```

What happens:
1. First claim calculates total reward (base + bonus if target met)
2. Linear vesting: rewards unlock over ~7 days after campaign ends
3. Users can claim multiple times as rewards vest
4. LP tokens returned only when 100% vested

### Reading State

```bash
# Get campaign details
stellar contract invoke --id CBU2BI3VSNUCGBF25VWJB7SW75X4YNN22SWSY4W2IKG72XISSXTYNRJ2 \
  --source-account flashpool-admin --network testnet \
  -- get_campaign --id 1

# Get user position
stellar contract invoke --id CBU2BI3VSNUCGBF25VWJB7SW75X4YNN22SWSY4W2IKG72XISSXTYNRJ2 \
  --source-account flashpool-admin --network testnet \
  -- get_user_position --id 1 --user <USER_PUBLIC_KEY>

# Check TinyAMM reserves
stellar contract invoke --id CDA5NMKGGFVMSK753LHMCHTD6MXBBO7RAXFJUD3QJLEXXTX25K6E3UES \
  --source-account flashpool-admin --network testnet \
  -- get_reserves

# Check Treasury FLASH balance
stellar contract invoke --id CC64MIMYTVOY27745VMTBJAJ5G6HOIK42N2C2RYVXWGJCH46FCFTCFYL \
  --source-account flashpool-admin --network testnet \
  -- balance --id CDRPWM7JWXVF4PST3EHZFAGDWV5RVY5DV3TS4NGVQAZ6UC45CTKHTAVZ
```

## Running the Frontend

```bash
# Ensure .env.local has the contract IDs (already set up)
npm install
npm run dev
# Open http://localhost:3000
```

The frontend connects to testnet and reads campaign data from the deployed contracts. Users connect via Freighter/xBull wallet to sign transactions.

## Testing a Full Flow

1. Fund a test user account via [friendbot](https://friendbot.stellar.org)
2. Give the test user some USDC: `stellar contract invoke --id <USDC> -- transfer --from <ADMIN> --to <USER> --amount 10000000000`
3. Create a campaign (admin creates it, targeting a pool)
4. Join the campaign (user deposits tokens)
5. Wait for campaign to end (or use testnet's faster ledger times)
6. Claim rewards (user claims vested FLASH)

## Admin Key Management

The testnet admin key is stored at `~/.config/stellar/identity/flashpool-admin.toml`. This is a local file — do NOT use this key on mainnet. Generate a new secure key for mainnet deployment.
