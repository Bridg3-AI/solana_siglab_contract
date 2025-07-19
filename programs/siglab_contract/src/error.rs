use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Contract is paused")]
    ContractPaused,
    #[msg("Unauthorized access")]
    Unauthorized,
    #[msg("Invalid insurance type")]
    InvalidInsuranceType,
    #[msg("Premium amount too low")]
    PremiumTooLow,
    #[msg("Coverage amount exceeds maximum")]
    CoverageExceedsMax,
    #[msg("Policy already active")]
    PolicyAlreadyActive,
    #[msg("Policy not active")]
    PolicyNotActive,
    #[msg("Policy expired")]
    PolicyExpired,
    #[msg("Invalid oracle data")]
    InvalidOracleData,
    #[msg("Insufficient oracles for consensus")]
    InsufficientOracles,
    #[msg("Oracle data stale")]
    OracleDataStale,
    #[msg("Payout conditions not met")]
    PayoutConditionsNotMet,
    #[msg("Insufficient treasury balance")]
    InsufficientTreasuryBalance,
    #[msg("Reserve ratio below minimum")]
    ReserveRatioBelowMinimum,
    #[msg("Withdrawal delay not met")]
    WithdrawalDelayNotMet,
    #[msg("Invalid parameters")]
    InvalidParameters,
    #[msg("Math overflow")]
    MathOverflow,
}