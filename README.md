# Stellar Toronto Builder Summit—EasyA Consensus Hackathon 2025

## Hackathon Submission Template

## FlashPool: Decentralized Liquidity Campaign Platform

![FlashPool Diagram](https://github.com/alexanderkoh/flashpool-protocol/raw/main/flashpool.png)

FlashPool is a platform designed to enable users to create and participate in liquidity incentive campaigns for various token pairs, with rewards distributed in FLASH tokens. It aims to provide a user-friendly interface for launching campaigns, exploring active campaigns, and managing wallet connections for participation.

## Quick Demo

![FlashPool in Action](https://github.com/alexanderkoh/flashpool-protocol/raw/main/flashpool_gif.gif)

## Video Demo

[https://youtu.be/KtmUlvNVPd4](https://youtu.be/KtmUlvNVPd4)

## Technical Architecture Diagram

![Architecture Diagram](https://github.com/alexanderkoh/flashpool-protocol/blob/main/flow_final.png)

---

# Hackathon Submission

Project Name:

```text
FlashPool Protocol
```

Names of Team Members:

```text
- Bastian Alexander Koh
- Timothy Baker
- Hunter Sides
```

---

## Part 1: Narrative

**Problem statement:**

```text
Soroban, Stellar's new smart contract platform, is live—but it needs liquidity. Protocols and users alike face inefficient, generalized yield farming that doesn't scale, lacks transparency, and fails to incentivize deep and timely participation.
```

**User base**

```text
DeFi protocols on Soroban, DAO treasuries, grant programs, token communities seeking to bootstrap or deepen liquidity, and DeFi users across multiple chains.
```

**Impact**

```text
FlashPool optimizes yield distribution by rewarding participants dynamically based on rank, contribution size, and timing. It creates a new standard for programmatic, incentive-aligned liquidity bootstrapping. Its impact relies on simplifying the user experience for crypto-native users, as well as routing liquidity to multiple protocols at once.
```

**Why Stellar: How do you leverage Stellar Passkeys to provide a seamless and intuitive UX?**

```text
FlashPool integrates Stellar Passkeys to allow seamless auth and transaction signing. This brings Web2-like UX—biometric login, one-click signing—to DeFi on Stellar.
```

**Describe your experience building on Stellar:**

```text
Soroban docs and tooling are improving but still evolving. Dev cycle friction remains around contract testing and serialization. Passkey integration is fairly smooth, yet we also used Stellar Wallet Kit for its ease of integration, `launchtube` was critical to handle soroban ops.
```

---

## Part 2: Your Minimal Viable Product

**In-Depth Documentation and Reward Distribution Formulas:**

```text
https://bastianstudio.notion.site/Hackathon-Consensus-25-1f23206352c680ae8c48c8684828ca35?pvs=4
```

**URL to a public code repository:**

```text
https://github.com/alexanderkoh/flashpool-protocol
```

**GitHub Repo Description mentioning Stellar**

```text
A liquidity campaign engine on Soroban. Built for Stellar Toronto Builder Summit - Easy A Consensus Hackathon 2025.
```

**List of implemented features:**

```text
- Campaign creation with configurable reward logic
- Explore and view live campaigns
- Wallet connection and mocked auth via Freighter
- Dynamic APY and pool stats via Hoops API
- Passkey auth integration (scaffolded)
```

**List of technologies used to build the project:**

```text
- TypeScript
- Rust (Soroban Contracts)
- Next.js
- Tailwind CSS
- Shadcn/UI
- Recharts
```

**Included Packages:**

```text
@stellar/stellar-sdk: 10.5.1
passkey-kit: latest
launchtube: latest
```

**Deployed contract IDs on Stellar Expert:**

```text
https://stellar.expert/explorer/public/contract/CA6ZUMVZ4BBEH3NGJ4G33IXKQPDVOWJQ2DMU5Z3SCIISVMT2NL2WCNDC (CampaignManager V1)
```

**Optional: Link to deployed front-end:**

```text
[Deployed Front-end Link](https://flashpool.xyz)
[Fallback Front-end Link](https://flash.stellar.red)
```

**Well-commented, functional Rust code:**

```text
Included in `/contracts/flash_campaign_manager/`
```

**UI code with smart contract bindings and Passkeys integration:**

```text
Located in `hooks/useWallet.ts` and `lib/stellar-wallet-kit-provider.tsx` 
```

---

## Part 3: Technical Docs

**Overview:**

```text
FlashPool is a liquidity incentive protocol on Soroban enabling token campaigns that reward LPs based on timing, contribution, and rank.
```

**Technical Architecture Diagram:**

![Architecture Diagram](https://github.com/alexanderkoh/flashpool-protocol/blob/main/flow_final.png)

### Design Choices
**Reward Distribution Logic**

## 4. Reward Distribution Logic

The FlashPool protocol distributes rewards using a **hybrid model** that blends:

1. **Rank-based weighting** (to reward early participants)  
2. **Time-based boost windows** (to incentivize participation mid-way or during strategic moments)  
3. **Proportional contribution** (so that deposits closer to the fundraising target are weighted fairly)

Let:

- $r_i$ = user's deposit rank (1 = first depositor)  
- $D_i$ = deposit amount  
- $P$ = campaign target  
- $t_d$ = deposit timestamp  
- $t_b, t_e$ = boost window start and end times  
- $B$ = boost multiplier (e.g., 1.25 if within boost window)  
- $\gamma$ = rank decay steepness (e.g., 1 for 1/r, 2 for 1/r²)

### **4.1. Rank-Based Weighting**

Each user receives a reward weight based on the **order** in which they joined the campaign. The first depositor gets the highest weight, the second gets less, and so on.

This creates a strong **first-mover incentive**, making Flash Sales feel like a real-time race.

$$
\mathrm{rankWeight}_i = \frac{1}{r_i^\gamma}
$$

### **4.2. Boost Window Modifier**

To maintain energy throughout the Flash Event, the curator can define **a time-based boost window** (e.g., hours 6–8 in a 24h sale).

Any deposits made during this time window receive a **bonus multiplier** on their reward weight.

$$
B_i = 
\begin{cases}
1.25 & \text{if within boost window} \\\\
1.0 & \text{otherwise}
\end{cases}
$$

This creates **multiple engagement peaks** — one at the start, and one at the boost window.

### **4.3. Contribution Proportion**

Users who deposit more (up to the target) receive more rewards. Deposits that go beyond the fundraising target **do not yield additional reward weight**, to prevent whales from dominating.

$$
\mathrm{contribWeight}_i = \min\left(1, \frac{D_i}{P}\right)
$$

### **4.4. Final Score Formula**

All components are multiplied together to get the user’s score:

$$
S_i = \left(\frac{1}{r_i^\gamma}\right) \cdot \min\left(1, \frac{D_i}{P} \right) \cdot B_i
$$

Each user’s share of the reward pool is:

$$
\text{reward}_i = R \cdot \frac{S_i}{\sum_j S_j}
$$

**Storage:**

```text
Used maps and persistent variables in Soroban for campaign state. Chose persistent maps for flexible indexing.
```

**Contract State:**

```text
Campaigns are stored by ID. Each campaign stores start/end times, boost params, target TVL, and reward allocation.
```

**Emitted Events:**

```text
- CampaignCreated
- LiquidityContributed
- RewardsClaimed
```

**Passkey Implementation:**

```text
Passkey-kit is scaffolded. The goal is to allow biometric login and in-browser auth without modal popups or external redirects.
```

**Issues Overcome:**

```text
- Coordinating contract instantiation in a single transaction required rethinking reward minting logic.
- UX challenges due to mocked wallet flows.
- Limited SDK coverage for deep Soroban tooling.
```
---
