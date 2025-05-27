use soroban_sdk::Env;

/// Computes rank-based decay weight.
/// Formula: rank_weight = 10_000 / (rank ^ gamma)
/// All values are integers; result scaled by 10_000.
pub fn compute_rank_weight(_env: &Env, rank: u32, gamma: u32) -> i128 {
    if rank == 0 {
        return 0;
    }
    let rank_pow = (rank as u128).checked_pow(gamma).unwrap_or(u128::MAX);
    if rank_pow == 0 {
        return 0;
    }
    let base: u128 = 10_000 * 10_000;
    let result = base / rank_pow;
    result.min(i128::MAX as u128) as i128
}

/// Computes contribution weight based on how close deposit is to target.
/// Formula: contrib_weight = min(1, deposit / target), scaled by 10_000.
pub fn compute_contribution_weight(_env: &Env, deposit: i128, target: i128) -> i128 {
    if target <= 0 || deposit <= 0 {
        return 0;
    }
    let ratio = deposit
        .checked_mul(10_000)
        .unwrap_or(i128::MAX)
        .checked_div(target)
        .unwrap_or(0);
    ratio.min(10_000)
}

/// Computes overall score by multiplying both weights.
/// Returns final score scaled by 10_000 for fixed-point representation.
pub fn compute_score(_env: &Env, rank_weight: i128, contrib_weight: i128) -> i128 {
    rank_weight
        .checked_mul(contrib_weight)
        .unwrap_or(0)
        .checked_div(10_000)
        .unwrap_or(0)
}