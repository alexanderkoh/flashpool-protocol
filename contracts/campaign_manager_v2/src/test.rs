#![cfg(test)]

use soroban_sdk::Env;
use crate::rewards;

#[test]
fn test_rank_weight_first() {
    let env = Env::default();
    let w = rewards::compute_rank_weight(1, 2);
    assert_eq!(w, 100_000_000);
}

#[test]
fn test_rank_weight_tenth() {
    let env = Env::default();
    let w = rewards::compute_rank_weight(10, 2);
    assert_eq!(w, 1_000_000);
}

#[test]
fn test_rank_weight_first_is_100x_tenth() {
    let env = Env::default();
    let w1 = rewards::compute_rank_weight(1, 2);
    let w10 = rewards::compute_rank_weight(10, 2);
    assert_eq!(w1 / w10, 100);
}

#[test]
fn test_rank_weight_zero_returns_zero() {
    let env = Env::default();
    assert_eq!(rewards::compute_rank_weight(0, 2), 0);
}

#[test]
fn test_contrib_weight_at_target() {
    let env = Env::default();
    let w = rewards::compute_contribution_weight(1000, 1000);
    assert_eq!(w, 10_000);
}

#[test]
fn test_contrib_weight_half_target() {
    let env = Env::default();
    let w = rewards::compute_contribution_weight(500, 1000);
    assert_eq!(w, 5_000);
}

#[test]
fn test_contrib_weight_over_target_capped() {
    let env = Env::default();
    let w = rewards::compute_contribution_weight(2000, 1000);
    assert_eq!(w, 10_000);
}

#[test]
fn test_contrib_weight_zero_deposit() {
    let env = Env::default();
    assert_eq!(rewards::compute_contribution_weight(0, 1000), 0);
}

#[test]
fn test_contrib_weight_zero_target() {
    let env = Env::default();
    assert_eq!(rewards::compute_contribution_weight(1000, 0), 0);
}

#[test]
fn test_compute_score() {
    let env = Env::default();
    let rank_w = rewards::compute_rank_weight(1, 2);
    let contrib_w = rewards::compute_contribution_weight(500, 1000);
    let score = rewards::compute_score(rank_w, contrib_w);
    assert_eq!(score, 50_000_000);
}

#[test]
fn test_compute_vested_pct_zero_elapsed() {
    let env = Env::default();
    let pct = rewards::compute_vested_pct(0, 100_000);
    assert_eq!(pct, 0);
}

#[test]
fn test_compute_vested_pct_half() {
    let env = Env::default();
    let pct = rewards::compute_vested_pct(50_000, 100_000);
    assert_eq!(pct, 5_000);
}

#[test]
fn test_compute_vested_pct_full() {
    let env = Env::default();
    let pct = rewards::compute_vested_pct(100_000, 100_000);
    assert_eq!(pct, 10_000);
}

#[test]
fn test_compute_vested_pct_over() {
    let env = Env::default();
    let pct = rewards::compute_vested_pct(200_000, 100_000);
    assert_eq!(pct, 10_000);
}
