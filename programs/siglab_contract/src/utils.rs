use anchor_lang::prelude::*;
use crate::error::InsuranceError;

/// Utility macros for common error checking patterns
pub mod macros {
    /// Require that the caller is authorized (has admin permissions)
    #[macro_export]
    macro_rules! require_authorized {
        ($condition:expr) => {
            require!($condition, crate::error::InsuranceError::Unauthorized)
        };
        ($condition:expr, $msg:expr) => {
            require!($condition, crate::error::InsuranceError::Unauthorized)
        };
    }

    /// Require that solvency requirements are met
    #[macro_export]
    macro_rules! require_solvency {
        ($treasury_balance:expr, $required_reserves:expr) => {
            require!(
                $treasury_balance >= $required_reserves,
                crate::error::InsuranceError::SolvencyCheckFailed
            )
        };
    }

    /// Require that oracle data is valid and not stale
    #[macro_export]
    macro_rules! require_valid_oracle_data {
        ($oracle_timestamp:expr, $current_time:expr, $staleness_threshold:expr) => {
            require!(
                $current_time - $oracle_timestamp <= $staleness_threshold,
                crate::error::InsuranceError::OracleDataStale
            )
        };
    }

    /// Require that policy is active
    #[macro_export]
    macro_rules! require_policy_active {
        ($policy_status:expr) => {
            require!(
                matches!($policy_status, crate::state::PolicyStatus::Active),
                crate::error::InsuranceError::PolicyNotActive
            )
        };
    }

    /// Require that contract is not paused
    #[macro_export]
    macro_rules! require_not_paused {
        ($is_paused:expr) => {
            require!(!$is_paused, crate::error::InsuranceError::ContractPaused)
        };
    }

    /// Require that premium amount is sufficient
    #[macro_export]
    macro_rules! require_sufficient_premium {
        ($premium_amount:expr, $minimum_premium:expr) => {
            require!(
                $premium_amount >= $minimum_premium,
                crate::error::InsuranceError::InsufficientPremium
            )
        };
    }
}

/// Error handling utilities
pub mod error_utils {
    use super::*;

    /// Convert system errors to InsuranceError with context
    pub fn handle_system_error(error: anchor_lang::error::Error) -> InsuranceError {
        msg!("System error occurred: {:?}", error);
        InsuranceError::InvalidParameters
    }

    /// Log error with context information
    pub fn log_error_with_context(error: &InsuranceError, context: &str, details: &str) {
        msg!("Error: {:?} | Context: {} | Details: {}", error, context, details);
    }

    /// Validate treasury balance and return appropriate error
    pub fn validate_treasury_balance(
        treasury_balance: u64,
        required_amount: u64,
        reserve_ratio: u64,
    ) -> Result<()> {
        if treasury_balance < required_amount {
            return Err(InsuranceError::InsufficientTreasury.into());
        }

        let required_reserves = treasury_balance
            .checked_mul(reserve_ratio)
            .and_then(|x| x.checked_div(100))
            .ok_or(InsuranceError::MathOverflow)?;

        if treasury_balance - required_amount < required_reserves {
            return Err(InsuranceError::ReserveRatioBelowMinimum.into());
        }

        Ok(())
    }

    /// Validate oracle data freshness
    pub fn validate_oracle_freshness(
        oracle_timestamp: i64,
        current_timestamp: i64,
        staleness_threshold: i64,
    ) -> Result<()> {
        require!(
            current_timestamp - oracle_timestamp <= staleness_threshold,
            InsuranceError::OracleDataStale
        );
        Ok(())
    }

    /// Validate policy eligibility for claims
    pub fn validate_policy_claim_eligibility(
        policy_end_date: i64,
        policy_status: &crate::state::PolicyStatus,
        current_timestamp: i64,
    ) -> Result<()> {
        require!(
            matches!(policy_status, crate::state::PolicyStatus::Active),
            InsuranceError::PolicyNotActive
        );

        require!(
            current_timestamp <= policy_end_date,
            InsuranceError::PolicyExpired
        );

        Ok(())
    }
}

/// Helper trait for adding context to Results
pub trait ResultExt<T> {
    fn with_context(self, context: &str) -> Result<T>;
    fn log_on_error(self, context: &str) -> Result<T>;
}

impl<T> ResultExt<T> for Result<T> {
    fn with_context(self, context: &str) -> Result<T> {
        self.map_err(|e| {
            msg!("Error in {}: {:?}", context, e);
            e
        })
    }

    fn log_on_error(self, context: &str) -> Result<T> {
        if let Err(ref e) = self {
            msg!("Error in {}: {:?}", context, e);
        }
        self
    }
}