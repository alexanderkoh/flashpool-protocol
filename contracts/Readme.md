# Flash Campaign Manager (contracts folder)

This folder contains an **experimental Soroban contract** that lets users
crowd-fund FLASH/USDC liquidity-pair positions and share the resulting rewards.

.
├── flash_campaign_manager
│ ├── Cargo.toml
│ └── src
│ ├── lib.rs ← the contract
│ └── test.rs ← unit-tests (mock pair + reference token)
└── soroswap-contracts
└── soroswap_pair.wasm


---

## What the contract does

| Function | Purpose |
| -------- | ------- |
| `initialize`            | one-time set-up – stores admin + token addresses, pulls any FLASH balance from admin |
| `create_campaign`       | admin or any user pays USDC → converts part to FLASH + LP tokens → stores a `Campaign` |
| `join_campaign`         | user deposits token-0, half is swapped, LP minted; user gets proportional _weight_ |
| `compound`              | reinvests fees; converts USDC fees to FLASH and grows the reward pool |
| `claim`                 | after `end_ledger` user withdraws LP plus FLASH rewards/bonus |
| `set_surplus_bps`/`set_ttl` | admin tunables |

All important state transitions are **logged** with `log!()` – those
messages start with an emoji (✅, 📦, 👤, 🔁, 💸, …) for quick scanning.

---

## Unit tests

* `create_and_join_campaign` – covers init, campaign creation & joining  
* `compound_updates_reward_pool` – checks `compound` math/logs  
* `claim_after_unlock` – advances the ledger (`sequence_number`) and claims

### Log capture

Tests call `dump_logs` which prints **every log entry** emitted by the
contract after each major step.  
Run them with:

```bash
cargo test -- --nocapture
To-Dos / next steps
Security review – overflow checks look good, but external review needed

Edge-case tests – negative paths (TooEarly, NothingToClaim, etc.)

Gas profiling – the happy paths fit comfortably, but compaction may help

Production token contracts – replace the reference stellar-asset token in tests with real deployments

Enjoy hacking! 🚀