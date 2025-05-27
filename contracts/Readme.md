# Flash Campaign Manager (contracts folder)
cargo test -p flash_campaign_manager -- --nocapture --test-threads=1
cargo test -p flash_campaign_manager test::test_simulate_circulating_supply_growth -- --nocapture --test-threads=1

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
```

### To-Dos / next steps
Security review – overflow checks look good, but external review needed

Edge-case tests – negative paths (TooEarly, NothingToClaim, etc.)

Gas profiling – the happy paths fit comfortably, but compaction may help

Production token contracts – replace the reference stellar-asset token in tests with real deployments

Enjoy hacking! 🚀

#### a deployment of the contract can be found at CAGZUMVZ4BBEH5NG34633IXKQPDVOWJQ2DMUSZ3SCIISVMT2NL2NCNDC on public


### To run the tests and hack run this:

PS C:\flashtoken\contracts> cargo test -- --nocapture
   Compiling flash_campaign_manager v0.0.0 (C:\flashtoken\github\flashpool-protocol\contracts\flash_campaign_manager)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 7.16s
     Running unittests src\lib.rs (target\debug\deps\flash_campaign_manager-14059600053fe177.exe)


## what should the tests do?

Each test line in the plan below gets a comment checkbox; update as they pass/fail.
    1. Setup:
        - [x] create the admin account (god)
        - [x] create ten user accounts (alice, bob, charlie, dave, eve, frank, grace, heidi, ivan, judy)
        - [x] create three tokens (flash, usdc, and eurc)
        - [x] mint 10_000_000 flash, 1,000,000 usdc, and 1,250,000 eurc.
        - [x] distribute all the flash to god, and give a proportional amount of usdc and eurc to each user, with the user with the least receiving 100% less than the user with the most, and at a ratio of 1:1.25 usdc to eurc.  the maximum amount one user can receive is 5000 us and 6250 eurc.
    - deploy a soroswap factory, and create two pairs, flash/usdc and usdc/eurc

2. initialize the flash campaign manager contract.
    - ensure that it transfers the flash token to the contract.
    - ensure it transfers the correct amount of usdc to the contract.
    - ensure it deposits it into the usdc/flash pair.
3. create a campaign
    - create a campaign with the following parameters:
    - 500 usdc.
    - target pair is to be the usdc/eurc pair.
    - target amount is 75,000 usdc worth of liquidity added to the pair.
    - make sure the campaign correctly purchases flash with the usdc.
    - ensure the campaign creation then deeposits the remaining usdc along with the purchased flash into the usdc/flash pair.
    - ensure the campaign allocates the correct amount of flash to the campaign.
    - the amount of flash allocated to the campaign is the amount of flash that would be needed to sell to the usdc/flash pair to cause the price to drop to below what the price was before the campaign was created.
    - well the actual amount is 5% more than that required amount.
    - setup the campaigns so that they are able to end right after all the users join for the tests, so just increment the ledger by the number of blocks required.
4. join the campaign.
    - complete a camapgin with only one user.
    - complete a campaign with all users, but do not meet the target amount.
    - complete a campaign with all users, and meet the target amount.
5. compond a campaign.
    - to compond a camapgin there needs to have been volume to create fees against the target pair.
    - so after the users have joined the campaign, we need to create trades against the target pair, and generate fees.
    - then we need to make sure that calling compond on the campaign claims those fees, and then purchases more flash with the fees.
    - this flash should then be allocated to the users in the campaign and should be able to be claimed.
6. claim the rewards.
    - make sure that the users can claim their rewards after the campaign has ended.
    - make sure that the users can claim their rewards after the campaign has been compounded.
    - make sure that the users can claim their rewards after the campaign has been compounded and the campaign has ended.
    - make sure that users can not claim their rewardsd or withdraw their liquidity before the campaign has ended

```bash

running 3 tests
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[INITIALIZE] initialize(admin Contract(CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAK3IM))"]
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[INITIALIZE] initialize(admin Contract(CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAK3IM))"]
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[INITIALIZE] initialize(admin Contract(CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAK3IM))"]
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[INITIALIZE] pulled 1000000 FLASH from admin"]
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[INITIALIZE] pulled 1000000 FLASH from admin"]
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[INITIALIZE] pulled 1000000 FLASH from admin"]
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[CREATE CAMPAIGN] take_fee: 1000 USDC from Contract(CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAK3IM)"]
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[CREATE CAMPAIGN] take_fee: 1000 USDC from Contract(CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAK3IM)"]
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[CREATE CAMPAIGN] take_fee: 1000 USDC from Contract(CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAK3IM)"]
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[CREATE CAMPAIGN] reserves_before: rf 100000  ru 100"]
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[CREATE CAMPAIGN] reserves_before: rf 100000  ru 100"]
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[CREATE CAMPAIGN] reserves_before: rf 100000  ru 100"]
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[CREATE CAMPAIGN] fee_split: swap 281 USDC | add_liq 719 USDC"]
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[CREATE CAMPAIGN] fee_split: swap 281 USDC | add_liq 719 USDC"]
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[CREATE CAMPAIGN] fee_split: swap 281 USDC | add_liq 719 USDC"]
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[CREATE CAMPAIGN] swap_result: got 73753 FLASH"]
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[CREATE CAMPAIGN] swap_result: got 73753 FLASH"]
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[CREATE CAMPAIGN] swap_result: got 73753 FLASH"]
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[CREATE CAMPAIGN] add_liquidity: need 49531 FLASH (donated 0), minted 42 LP"]
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[CREATE CAMPAIGN] surplus 24222 FLASH => reward_pool 24222 FLASH"]
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[CREATE CAMPAIGN] add_liquidity: need 49531 FLASH (donated 0), minted 42 LP"]
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[CREATE CAMPAIGN] add_liquidity: need 49531 FLASH (donated 0), minted 42 LP"]
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[CREATE CAMPAIGN] surplus 24222 FLASH => reward_pool 24222 FLASH"]
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[CREATE CAMPAIGN] surplus 24222 FLASH => reward_pool 24222 FLASH"]
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[CREATE CAMPAIGN] created id 1 lp_minted 42 reward 24222"]
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[CREATE CAMPAIGN] created id 1 lp_minted 42 reward 24222"]
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[CREATE CAMPAIGN] created id 1 lp_minted 42 reward 24222"]
── logs after create_campaign ─────────────────────────────
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[CREATE CAMPAIGN] take_fee: 1000 USDC from Contract(CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAK3IM)"]
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[CREATE CAMPAIGN] reserves_before: rf 100000  ru 100"]
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[CREATE CAMPAIGN] fee_split: swap 281 USDC | add_liq 719 USDC"]
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[CREATE CAMPAIGN] swap_result: got 73753 FLASH"]
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[CREATE CAMPAIGN] add_liquidity: need 49531 FLASH (donated 0), minted 42 LP"]
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[CREATE CAMPAIGN] surplus 24222 FLASH => reward_pool 24222 FLASH"]
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[CREATE CAMPAIGN] created id 1 lp_minted 42 reward 24222"]
──────────────────────────────────────────────────

[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[JOIN CAMPAIGN] deposit 2000 of token0 from Contract(CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAMDR4)"]
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[JOIN CAMPAIGN] deposit 2000 of token0 from Contract(CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAMDR4)"]
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[JOIN CAMPAIGN] deposit 2000 of token0 from Contract(CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAMDR4)"]
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[JOIN CAMPAIGN] swap_half: 1000 token0 => 3 token1"]
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[JOIN CAMPAIGN] swap_half: 1000 token0 => 3 token1"]
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[JOIN CAMPAIGN] swap_half: 1000 token0 => 3 token1"]
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[JOIN CAMPAIGN] minted 42 LP for user Contract(CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAMDR4)"]
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[JOIN CAMPAIGN] minted 42 LP for user Contract(CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAMDR4)"]
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[JOIN CAMPAIGN] minted 42 LP for user Contract(CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAMDR4)"]
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[JOIN CAMPAIGN] join(id 1) user Contract(CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAMDR4) lp 42 weight 42"]
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[JOIN CAMPAIGN] join(id 1) user Contract(CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAMDR4) lp 42 weight 42"]
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[JOIN CAMPAIGN] join(id 1) user Contract(CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAMDR4) lp 42 weight 42"]
── logs after join_campaign ─────────────────────────────
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[JOIN CAMPAIGN] deposit 2000 of token0 from Contract(CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAMDR4)"]
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[JOIN CAMPAIGN] swap_half: 1000 token0 => 3 token1"]
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[JOIN CAMPAIGN] minted 42 LP for user Contract(CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAMDR4)"]
[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[JOIN CAMPAIGN] join(id 1) user Contract(CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAMDR4) lp 42 weight 42"]
──────────────────────────────────────────────────

[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[CLAIM] id 1 user Contract(CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAMDR4) flash 24222 lp 42"]
Writing test snapshot file for test "test::create_and_join_campaign" to "t[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[COMPOUND] withdraw_from_pair: token0 26247 token1 97"]
e── logs after claim ─────────────────────────────
st[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[CLAIM] id 1 user Contract(CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAMDR4) flash 24222 lp 42"]
_──────────────────────────────────────────────────

snapshots\\test\\creat[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[COMPOUND] re-deposit minted 42 LP"]
e_and[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[COMPOUND] compound(id 1) fee_lp 0 gain 0"]
_join_campaign.── logs after compound ─────────────────────────────
1[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[COMPOUND] withdraw_from_pair: token0 26247 token1 97"]
.[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[COMPOUND] re-deposit minted 42 LP"]
j[Diagnostic Event] contract:CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARQG5, topics:[log], data:["{}", "[COMPOUND] compound(id 1) fee_lp 0 gain 0"]
──────────────────────────────────────────────────

son".
Writing test snapshot file for test "test::compound_updates_reward_pool" to "test_snapshots\\test\\compound_updates_reward_pool.1.json".
Writing test snapshot file for test "test::claim_after_unlock" to "test_snapshots\\test\\claim_after_unlock.1.json".
test test::create_and_join_campaign ... ok
test test::compound_updates_reward_pool ... ok
test test::claim_after_unlock ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s

```
