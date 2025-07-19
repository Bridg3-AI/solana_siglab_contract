use anchor_lang::prelude::*;

pub const MASTER_CONTRACT_SEED: &[u8] = b"master_contract";
pub const POLICY_SEED: &[u8] = b"policy";
pub const ORACLE_SEED: &[u8] = b"oracle";
pub const TREASURY_SEED: &[u8] = b"treasury";

pub const MAX_ORACLES: usize = 10;
pub const MIN_ORACLES_FOR_CONSENSUS: usize = 3;
pub const ORACLE_UPDATE_INTERVAL: i64 = 300; // 5 minutes

pub const MIN_PREMIUM_AMOUNT: u64 = 1_000_000; // 0.001 SOL
pub const MAX_COVERAGE_AMOUNT: u64 = 1_000_000_000_000; // 1000 SOL
pub const MIN_RESERVE_RATIO: u64 = 20; // 20%

pub const ADMIN_WITHDRAWAL_DELAY: i64 = 86400; // 24 hours