const SCALE: i128 = 10_000;

/// Rank-based decay weight.
/// Formula: SCALE * SCALE / rank^gamma
/// Rank 1 with gamma=2 → 100_000_000. Rank 10 → 1_000_000.
pub fn compute_rank_weight(rank: u32, gamma: u32) -> i128 {
    if rank == 0 {
        return 0;
    }
    let rank_pow = (rank as u128).checked_pow(gamma).unwrap_or(u128::MAX);
    if rank_pow == 0 {
        return 0;
    }
    let base = (SCALE as u128) * (SCALE as u128);
    let result = base / rank_pow;
    result.min(i128::MAX as u128) as i128
}

/// Contribution weight: min(SCALE, deposit * SCALE / target).
/// Caps at SCALE so deposits beyond target get no extra weight.
pub fn compute_contribution_weight(deposit: i128, target: i128) -> i128 {
    if target <= 0 || deposit <= 0 {
        return 0;
    }
    let ratio = deposit
        .checked_mul(SCALE)
        .unwrap_or(i128::MAX)
        .checked_div(target)
        .unwrap_or(0);
    ratio.min(SCALE)
}

/// Combined score = rank_weight * contrib_weight / SCALE.
pub fn compute_score(rank_weight: i128, contrib_weight: i128) -> i128 {
    rank_weight
        .checked_mul(contrib_weight)
        .unwrap_or(0)
        .checked_div(SCALE)
        .unwrap_or(0)
}

/// Vesting percentage: min(SCALE, elapsed * SCALE / vesting_period).
/// Returns 0..10_000 representing 0%..100%.
pub fn compute_vested_pct(elapsed: u32, vesting_period: u32) -> i128 {
    if vesting_period == 0 {
        return SCALE;
    }
    let pct = (elapsed as i128)
        .checked_mul(SCALE)
        .unwrap_or(i128::MAX)
        .checked_div(vesting_period as i128)
        .unwrap_or(SCALE);
    pct.min(SCALE)
}
