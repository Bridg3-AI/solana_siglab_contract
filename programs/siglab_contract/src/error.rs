use anchor_lang::prelude::*;

/// Main error enum for the Insurance Contract
#[error_code]
pub enum InsuranceError {
    // === Core Contract Errors ===
    #[msg("Contract is currently paused and cannot process transactions")]
    ContractPaused,
    
    #[msg("Unauthorized access - caller does not have required permissions")]
    Unauthorized,
    
    #[msg("Invalid parameters provided to the instruction")]
    InvalidParameters,
    
    #[msg("Arithmetic operation resulted in overflow")]
    MathOverflow,
    
    // === Policy Management Errors ===
    #[msg("Premium amount is below the minimum required threshold")]
    InsufficientPremium,
    
    #[msg("Policy has expired and cannot be used for claims")]
    PolicyExpired,
    
    #[msg("Policy is not in active status")]
    PolicyNotActive,
    
    #[msg("Policy already exists with this ID")]
    PolicyAlreadyExists,
    
    #[msg("Policy not found with the provided ID")]
    PolicyNotFound,
    
    #[msg("Coverage amount exceeds the maximum allowed limit")]
    CoverageExceedsMaximum,
    
    #[msg("Invalid insurance type specified")]
    InvalidInsuranceType,
    
    // === Oracle Data Errors ===
    #[msg("Oracle data is invalid or corrupted")]
    InvalidOracleData,
    
    #[msg("Oracle data is stale and beyond acceptable threshold")]
    OracleDataStale,
    
    #[msg("Insufficient oracles for reaching consensus")]
    InsufficientOracles,
    
    #[msg("Oracle signature verification failed")]
    OracleSignatureInvalid,
    
    #[msg("Oracle consensus mechanism failed")]
    OracleConsensusFailure,
    
    #[msg("Oracle is not registered in the system")]
    OracleNotRegistered,
    
    #[msg("Oracle is currently inactive")]
    OracleInactive,
    
    #[msg("Oracle data is too old and cannot be used")]
    OracleDataTooOld,
    
    #[msg("Maximum number of oracles has been exceeded")]
    MaxOraclesExceeded,
    
    #[msg("Oracle is already registered")]
    OracleAlreadyRegistered,
    
    #[msg("Invalid input provided")]
    InvalidInput,
    
    // === Financial Operation Errors ===
    #[msg("Insufficient treasury balance to process payout")]
    InsufficientTreasury,
    
    #[msg("Insufficient reserves to maintain solvency requirements")]
    InsufficientReserves,
    
    #[msg("Reserve ratio is below minimum required threshold")]
    ReserveRatioBelowMinimum,
    
    #[msg("Solvency check failed - operation would violate financial constraints")]
    SolvencyCheckFailed,
    
    #[msg("Treasury operation failed due to internal error")]
    TreasuryOperationFailed,
    
    #[msg("Reserve ratio violation - operation exceeds allowed limits")]
    ReserveRatioViolation,
    
    #[msg("Invalid premium amount - must be within acceptable range")]
    InvalidPremiumAmount,
    
    // === Payout and Claims Errors ===
    #[msg("Payout conditions have not been met based on oracle data")]
    PayoutConditionsNotMet,
    
    #[msg("Claim has already been processed for this policy")]
    ClaimAlreadyProcessed,
    
    #[msg("Claim period has expired")]
    ClaimPeriodExpired,
    
    #[msg("Invalid claim amount requested")]
    InvalidClaimAmount,
    
    // === Administrative Errors ===
    #[msg("Admin withdrawal delay period has not been met")]
    WithdrawalDelayNotMet,
    
    #[msg("Operation requires contract to be paused")]
    ContractMustBePaused,
    
    #[msg("Operation requires contract to be active")]
    ContractMustBeActive,
    
    #[msg("Invalid admin operation parameters")]
    InvalidAdminOperation,
}