# FlashPool v2 — Protocol Redesign Spec

## Overview

Full redesign of the FlashPool liquidity campaign protocol. Replaces the hackathon MVP with a mainnet-ready architecture using a protocol-owned AMM (TinyAMM) for the FLASH/USDC core pair, Hoops Finance adapters for multi-DEX target pool support, linear reward vesting, and a treasury contract for FLASH supply management.

## Goals

- Deploy to Stellar mainnet
- Support campaigns targeting pools on Aqua, Comet, Phoenix, and SoroSwap
- Eliminate all critical bugs from v1 (swap fee math, compound drain, storage scaling, double-join overwrite)
- Add reward vesting to retain liquidity beyond campaign duration
- Clean, auditable contracts with proper events and error handling

## Non-Goals (v1 boundary)

- Rolling campaigns with rank persistence (v1.1)
- Compound / mid-campaign reward growth (v1.1)
- Creator-configurable gamma, vesting, boost windows (v1.1)
- Multi-hop swaps for campaign creation
- Campaign cancellation / early termination
- Governance token or voting
- Cross-chain functionality

## System Architecture

Three contracts plus existing Hoops adapters:

```
┌─────────────────────────────────────────────────────────────┐
│                     FlashPool Protocol                       │
│                                                              │
│  ┌────────────┐  ┌──────────────────────┐  ┌─────────────┐ │
│  │  Treasury   │  │  Campaign Manager    │  │  TinyAMM    │ │
│  │  ~100 LOC   │  │  ~600 LOC            │  │  FLASH/USDC │ │
│  │             │  │                      │  │  (existing)  │ │
│  │  Holds FLASH│←─│  Pulls FLASH for     │──│  Swaps &    │ │
│  │  Approves   │  │  reward payouts      │  │  liquidity  │ │
│  │  spenders   │  │                      │  │             │ │
│  └────────────┘  │  Calls Hoops adapters │  └─────────────┘ │
│                  │  for target pool ops  │                   │
│                  └──────────┬───────────┘                   │
│                             │                                │
│                  ┌──────────▼───────────┐                   │
│                  │   Hoops Adapters     │                   │
│                  │   (existing code)    │                   │
│                  │   Aqua | Comet       │                   │
│                  │   Phoenix | SoroSwap │                   │
│                  └─────────────────────┘                   │
└─────────────────────────────────────────────────────────────┘
```

**Value flow:**
1. Creator pays USDC → Campaign Manager
2. Campaign Manager swaps portion to FLASH via TinyAMM → buy pressure + LP depth
3. Surplus FLASH = base reward pool (held by manager)
4. Manager pulls `bonus_flash` from Treasury (if specified) → also held by manager
5. Users deposit single token → Manager swaps half via Hoops adapter → deposits both sides as LP
6. On claim, manager transfers vested FLASH directly to user. LP returned after full vest

---

## Contract 1: Treasury (~100 LOC)

### Purpose

Holds the FLASH token supply. Gates who can spend it via allowance management.

### Storage

- `admin: Address` — can approve/revoke spenders and withdraw
- `flash_token: Address` — FLASH token contract address

### Functions

| Function | Auth | Description |
|----------|------|-------------|
| `initialize(admin, flash_token)` | admin | One-time setup |
| `approve_spender(spender, amount)` | admin | Calls `flash_token.approve(spender, amount)` |
| `revoke_spender(spender)` | admin | Calls `flash_token.approve(spender, 0)` |
| `get_allowance(spender)` | none | Returns `flash_token.allowance(treasury, spender)` |
| `withdraw(to, amount)` | admin | Emergency withdraw FLASH to any address |

### Deployment

1. Deploy Treasury with `initialize(admin, flash_token)`
2. Admin transfers total FLASH supply to Treasury address
3. Admin calls `approve_spender(campaign_manager, initial_allowance)`

---

## Contract 2: TinyAMM (Existing — No Changes)

### Purpose

Protocol-owned FLASH/USDC constant-product AMM. Canonical market for FLASH price discovery.

### Interface Used by Campaign Manager

| Function | Usage |
|----------|-------|
| `swap_exact_in(trader, sell_token, sell_amount, min_buy)` | Swap USDC→FLASH during campaign creation |
| `add_liquidity(to, amt0, amt1, min0, min1)` | Deepen core pair during campaign creation |
| `get_reserves()` | Calculate expected swap output for slippage checks |

### Configuration

- Fee: 30 bps (0.3%)
- Initialized with FLASH + USDC seed liquidity from admin

### Deployment

1. Deploy TinyAMM with `init(flash_token, usdc_token, 30)`
2. Admin seeds initial liquidity via `add_liquidity()`

---

## Contract 3: Campaign Manager (~600 LOC)

### Storage Model

**Instance storage (singleton):**

```rust
CoreConfig {
    admin: Address,
    flash: Address,
    usdc: Address,
    treasury: Address,
    tinyamm: Address,
    hoops_router: Address,
    next_campaign_id: u32,
    defaults: ProtocolDefaults,
}

ProtocolDefaults {
    gamma: u32,             // rank decay exponent, default: 2
    vesting_period: u32,    // ledgers after campaign end, default: ~7 days
    surplus_bps: u32,       // protocol fee split, default: 500 (5%)
    ttl_thresh: u32,
    ttl_bump: u32,
}
```

**Persistent storage — keyed by campaign ID:**

```rust
Campaign {
    adapter_id: i128,
    pool_address: Address,
    lp_token: Address,
    creator: Address,
    end_ledger: u32,
    vesting_end: u32,       // end_ledger + vesting_period
    target_lp: i128,
    total_lp: i128,
    total_weight: i128,
    reward_flash: i128,
    bonus_flash: i128,
    user_count: u32,
}
```

**Persistent storage — keyed by (campaign_id, user_address):**

```rust
UserPos {
    lp: i128,
    weight: i128,
    rank: u32,
    joined_ledger: u32,
    total_reward: i128,     // calculated on first claim
    claimed: i128,          // FLASH claimed so far
}
```

### Public Functions

| Function | Auth | Description |
|----------|------|-------------|
| `initialize(admin, flash, usdc, treasury, tinyamm, hoops_router)` | admin | One-time config setup |
| `create_campaign(fee_usdc, adapter_id, pool_address, lp_token, duration, target_lp, bonus_flash, min_reward_flash, creator)` | creator | Create campaign, swap USDC→FLASH via TinyAMM |
| `join_campaign(id, amount, token, user)` | user | Deposit into target pool via Hoops adapter |
| `claim(id, user)` | user | Claim vested FLASH, LP returned when fully vested |
| `set_defaults(admin, defaults)` | admin | Update protocol defaults |
| `get_campaign(id)` | none | Read campaign state |
| `get_user_position(id, user)` | none | Read user position + vesting status |

### Function Logic

#### `create_campaign`

```
1. creator.require_auth()
2. Verify no active campaign for this pool (or previous one has ended)
3. Transfer fee_usdc from creator → campaign manager
4. Calculate split using surplus_bps:
   - s_min = sqrt(reserve_usdc * (reserve_usdc + fee_usdc)) - reserve_usdc
   - swap_amount = min(s_min + fee_usdc * surplus_bps / 10_000, fee_usdc)
   - liq_amount = fee_usdc - swap_amount
5. Call tinyamm.swap_exact_in(manager, usdc, swap_amount, 0) → flash_out
6. Calculate flash_needed for balanced liquidity: liq_amount * flash_reserve / usdc_reserve
7. Call tinyamm.add_liquidity(manager, flash_needed, liq_amount, 0, 0)
8. surplus = flash_out - flash_needed (FLASH remaining in manager = base reward pool)
9. ensure!(surplus >= min_reward_flash, SlippageExceeded)
10. If bonus_flash > 0: flash_token.transfer_from(treasury, manager, bonus_flash)
11. Increment next_campaign_id
12. Store Campaign {
      reward_flash: surplus,
      bonus_flash,
      end_ledger: current + duration,
      vesting_end: current + duration + defaults.vesting_period,
      user_count: 0,
      total_lp: 0,
      total_weight: 0,
    }
13. Emit CampaignCreated event
```

#### `join_campaign`

```
1. user.require_auth()
2. Load campaign, ensure current_ledger < end_ledger
3. Transfer token from user → campaign manager
4. Get Hoops adapter address from router by adapter_id
5. Swap half of user's token for the other side of the pair via adapter.swap_in_pool()
6. Deposit both tokens into target pool via adapter.add_liq_in_pool() → receive LP tokens
7. Increment user_count, assign rank = user_count
8. Compute weight:
   - rank_weight = 10_000 * 10_000 / rank^gamma
   - contrib_weight = min(10_000, lp * 10_000 / target_lp)
   - weight = rank_weight * contrib_weight / 10_000
9. If user already has position: accumulate (lp += new_lp, recompute weight on total)
10. Update campaign: total_lp += lp, total_weight += weight (subtract old weight if re-join)
11. Store/update UserPos
12. Emit UserJoined event
```

#### `claim`

```
1. user.require_auth()
2. Load campaign, ensure current_ledger >= end_ledger
3. Load UserPos, ensure weight > 0
4. If total_reward == 0 (first claim):
   - base = reward_flash * weight / total_weight
   - bonus = if total_lp >= target_lp { bonus_flash * weight / total_weight } else { 0 }
   - total_reward = base + bonus
5. Calculate vesting:
   - elapsed = current_ledger - end_ledger
   - vested_pct = min(10_000, elapsed * 10_000 / vesting_period)
   - claimable = (total_reward * vested_pct / 10_000) - claimed
6. ensure!(claimable > 0, NothingToClaim)
7. flash_token.transfer(manager, user, claimable)  // manager holds reward FLASH
8. claimed += claimable
9. If claimed >= total_reward (fully vested):
   - Transfer LP tokens to user
   - Remove UserPos from storage
10. Else: save updated UserPos
11. Emit RewardClaimed event
```

### Reward Formula

```
rank_weight = 10_000 * 10_000 / rank^gamma

contrib_weight = min(10_000, deposit * 10_000 / target)

score = rank_weight * contrib_weight / 10_000

user_reward = total_rewards * (score / sum_all_scores)
```

- `gamma = 2` (protocol default): rank 1 gets 100x weight of rank 10
- Contribution capped at target — whales can't absorb all rewards
- Fixed-point arithmetic scaled by 10,000

### Events

| Event | Data | Emitted by |
|-------|------|------------|
| `CampaignCreated` | `id, creator, adapter_id, pool_address, reward_flash, bonus_flash, duration, target_lp` | `create_campaign` |
| `UserJoined` | `campaign_id, user, lp, rank, weight` | `join_campaign` |
| `RewardClaimed` | `campaign_id, user, flash_amount, fully_vested: bool` | `claim` |
| `DefaultsUpdated` | `gamma, vesting_period, surplus_bps` | `set_defaults` |

### Error Codes

```rust
pub enum FlashError {
    // Initialization
    AlreadyInitialized = 1,
    NotInitialized = 2,

    // Authorization
    NotAdmin = 10,
    NotAuthorized = 11,

    // Campaign lifecycle
    CampaignNotFound = 20,
    CampaignNotEnded = 21,
    CampaignAlreadyEnded = 22,
    CampaignActiveForPool = 23,

    // User operations
    NothingToClaim = 30,
    InsufficientAmount = 31,
    PositionNotFound = 32,

    // External calls
    SwapFailed = 40,
    SlippageExceeded = 41,
    AdapterCallFailed = 42,
    InsufficientRewardPool = 43,

    // Config
    InvalidBps = 50,
    InvalidDuration = 51,
    InvalidTarget = 52,
}
```

---

## Frontend Integration

### In Scope

- Replace all mock data with real Soroban contract calls
- Campaign creation flow → build and submit `create_campaign` XDR
- Campaign discovery → read campaign state from chain via `get_campaign`
- Join campaign flow → build and submit `join_campaign` XDR
- Rewards page → read vesting status via `get_user_position`, submit `claim` XDR
- Wallet connection already functional (Stellar Wallets Kit)

### Not in Frontend Scope

- Admin dashboard (admin calls via CLI/scripts)
- Hoops API integration for dynamic pool discovery (hardcode supported pools initially)
- Analytics / historical charts (needs indexer)

### Contract Interaction Pattern

```
User action → Build Soroban contract call → Sign via wallet → Submit to network
```

Using `@stellar/stellar-sdk` to construct `InvokeContractArgs`, sign with the connected wallet via `signTransaction()`, and submit via Soroban RPC.

---

## Deployment Sequence

1. Deploy FLASH token (Stellar Asset Contract)
2. Deploy USDC token (or use existing testnet/mainnet USDC)
3. Deploy TinyAMM → `init(flash, usdc, 30)`
4. Seed TinyAMM with initial FLASH/USDC liquidity
5. Deploy Treasury → `initialize(admin, flash)`
6. Transfer FLASH supply to Treasury
7. Deploy Campaign Manager → `initialize(admin, flash, usdc, treasury, tinyamm, hoops_router)`
8. Treasury: `approve_spender(campaign_manager, initial_allowance)`
9. Deploy frontend pointing to contract addresses

---

## Design Accommodations for Future Features

These decisions ensure v1.1 features can be added without contract redeployment:

- **Rolling campaigns:** Campaign and UserPos data is NOT deleted after claim — past positions can be queried for rank persistence
- **New DEX adapters:** `adapter_id` pattern is extensible — new Hoops adapters slot in via router registration
- **Configurable parameters:** `ProtocolDefaults` is a struct — new fields can be added via contract upgrade
- **Governance:** Treasury admin can be transferred to a governance contract without migrating tokens
- **Compound:** Campaign struct has room for additional reward tracking fields via upgrade

---

## Testing Strategy

### Unit Tests (per contract)

**Treasury:**
- Initialize, double-init rejection
- Approve/revoke spender
- Withdraw
- Auth checks on all admin functions

**Campaign Manager:**
- Create campaign with correct reward calculation
- Join campaign — rank assignment, weight computation
- Join campaign twice — accumulation not overwrite
- Claim — linear vesting math at 0%, 25%, 50%, 100%
- Claim — LP returned only at full vest
- Claim — bonus only if target met
- Campaign not ended — TooEarly rejection
- Active campaign for pool — rejection
- Slippage protection on create

### Integration Tests

- Full flow: create → join (multiple users) → wait → claim (partial) → claim (full)
- Multi-adapter: create campaign on SoroSwap pool, another on Aqua pool
- Treasury allowance exhaustion: verify clean error when reward pool exceeds allowance
