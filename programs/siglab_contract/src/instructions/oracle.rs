use anchor_lang::prelude::*;
use crate::state::{Oracle, OracleData, OracleType, MasterInsuranceContract, ConsensusData};
use crate::error::InsuranceError;
use anchor_lang::solana_program::ed25519_program;

#[derive(Accounts)]
#[instruction(oracle_id: String)]
pub struct RegisterOracle<'info> {
    #[account(
        init,
        payer = admin,
        space = Oracle::space(),
        seeds = [b"oracle", oracle_id.as_bytes()],
        bump
    )]
    pub oracle: Account<'info, Oracle>,
    
    #[account(
        mut,
        constraint = master_contract.authority == admin.key() @ InsuranceError::Unauthorized
    )]
    pub master_contract: Account<'info, MasterInsuranceContract>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
    
    pub oracle_authority: SystemAccount<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UnregisterOracle<'info> {
    #[account(
        mut,
        close = admin,
        seeds = [b"oracle", oracle.oracle_id.as_bytes()],
        bump = oracle.bump
    )]
    pub oracle: Account<'info, Oracle>,
    
    #[account(
        mut,
        constraint = master_contract.authority == admin.key() @ InsuranceError::Unauthorized
    )]
    pub master_contract: Account<'info, MasterInsuranceContract>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateOracleData<'info> {
    #[account(
        mut,
        seeds = [b"oracle", oracle.oracle_id.as_bytes()],
        bump = oracle.bump,
        constraint = oracle.authority == oracle_authority.key() @ InsuranceError::Unauthorized,
        constraint = oracle.is_active @ InsuranceError::OracleInactive
    )]
    pub oracle: Account<'info, Oracle>,
    
    pub oracle_authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateOracleStatus<'info> {
    #[account(
        mut,
        seeds = [b"oracle", oracle.oracle_id.as_bytes()],
        bump = oracle.bump
    )]
    pub oracle: Account<'info, Oracle>,
    
    #[account(
        constraint = master_contract.authority == admin.key() @ InsuranceError::Unauthorized
    )]
    pub master_contract: Account<'info, MasterInsuranceContract>,
    
    pub admin: Signer<'info>,
}

pub fn register_oracle(
    ctx: Context<RegisterOracle>,
    oracle_id: String,
    oracle_type: OracleType,
    data_feed_address: String,
) -> Result<()> {
    let oracle = &mut ctx.accounts.oracle;
    let master_contract = &mut ctx.accounts.master_contract;
    
    // Validate oracle_id length
    require!(
        oracle_id.len() <= Oracle::MAX_ORACLE_ID_LENGTH,
        InsuranceError::InvalidInput
    );
    
    // Validate data_feed_address length
    require!(
        data_feed_address.len() <= Oracle::MAX_DATA_FEED_ADDRESS_LENGTH,
        InsuranceError::InvalidInput
    );
    
    // Check if we haven't exceeded max oracles
    require!(
        master_contract.oracle_registry.len() < master_contract.max_oracles as usize,
        InsuranceError::MaxOraclesExceeded
    );
    
    // Check for duplicate oracle in registry
    require!(
        !master_contract.oracle_registry.contains(&oracle.key()),
        InsuranceError::OracleAlreadyRegistered
    );
    
    // Ensure only Pyth oracle type is supported
    require!(
        oracle_type == OracleType::Pyth,
        InsuranceError::InvalidOracleData
    );
    
    // Initialize oracle account
    oracle.oracle_id = oracle_id;
    oracle.authority = ctx.accounts.oracle_authority.key();
    oracle.oracle_type = oracle_type;
    oracle.is_active = true;
    oracle.last_update_timestamp = 0;
    oracle.data_feed_address = data_feed_address;
    oracle.latest_data = None;
    oracle.reputation_score = 100; // Start with perfect score
    oracle.update_count = 0;
    oracle.health_metrics = crate::state::OracleHealthMetrics::new();
    oracle.bump = ctx.bumps.oracle;
    
    // Add to master contract oracle registry
    master_contract.oracle_registry.push(oracle.key());
    
    Ok(())
}

pub fn unregister_oracle(ctx: Context<UnregisterOracle>) -> Result<()> {
    let oracle = &ctx.accounts.oracle;
    let master_contract = &mut ctx.accounts.master_contract;
    
    // Remove oracle from registry
    master_contract.oracle_registry.retain(|&x| x != oracle.key());
    
    // Oracle account will be closed automatically due to close constraint
    
    Ok(())
}

pub fn update_oracle_data(ctx: Context<UpdateOracleData>, data: OracleData) -> Result<()> {
    let oracle = &mut ctx.accounts.oracle;
    let clock = Clock::get()?;
    
    // Check data reasonableness and manipulation prevention
    validate_data_reasonableness(oracle, &data, 50)?; // Max 50% change
    
    // Validate timestamp (data should not be older than 5 minutes)
    let max_age = 5 * 60; // 5 minutes in seconds
    require!(
        clock.unix_timestamp - data.timestamp <= max_age,
        InsuranceError::OracleDataTooOld
    );
    
    // Verify signature
    let signature_result = verify_oracle_signature(&oracle.authority, &data);
    if signature_result.is_err() {
        update_oracle_health(oracle, false, clock.unix_timestamp)?;
        return signature_result;
    }
    
    // Check for replay attacks using nonce
    if let Some(ref last_data) = oracle.latest_data {
        require!(
            data.nonce > last_data.nonce,
            InsuranceError::InvalidOracleData
        );
    }
    
    // Update oracle data
    oracle.latest_data = Some(data);
    oracle.last_update_timestamp = clock.unix_timestamp;
    oracle.update_count += 1;
    
    // Update health metrics for successful update
    update_oracle_health(oracle, true, clock.unix_timestamp)?;
    
    Ok(())
}

/// Verify Ed25519 signature for oracle data
fn verify_oracle_signature(oracle_authority: &Pubkey, data: &OracleData) -> Result<()> {
    // Create message to verify (value + timestamp + confidence + nonce)
    let message = create_oracle_message(data);
    
    // For now, we'll implement a basic signature check
    // In a production environment, you would use proper Ed25519 verification
    // This requires additional dependencies or instruction verification
    
    // Placeholder verification - check that signature is not all zeros
    let signature_valid = !data.signature.iter().all(|&x| x == 0);
    
    require!(signature_valid, InsuranceError::OracleSignatureInvalid);
    
    Ok(())
}

/// Create message for signature verification
fn create_oracle_message(data: &OracleData) -> Vec<u8> {
    let mut message = Vec::new();
    message.extend_from_slice(&data.value.to_le_bytes());
    message.extend_from_slice(&data.timestamp.to_le_bytes());
    message.extend_from_slice(&data.confidence.to_le_bytes());
    message.extend_from_slice(&data.nonce.to_le_bytes());
    message
}

/// Parse Pyth oracle data format
pub fn parse_pyth_format(raw_data: &[u8]) -> Result<OracleData> {
    // Pyth Network format: value (8 bytes) + timestamp (8 bytes) + confidence (8 bytes)
    require!(raw_data.len() >= 24, InsuranceError::InvalidOracleData);
    
    let value = u64::from_le_bytes(raw_data[0..8].try_into().unwrap());
    let timestamp = i64::from_le_bytes(raw_data[8..16].try_into().unwrap());
    let confidence = u64::from_le_bytes(raw_data[16..24].try_into().unwrap());
    
    Ok(OracleData {
        value,
        timestamp,
        confidence,
        signature: [0; 64], // Will be set by caller
        nonce: 0, // Will be set by caller
    })
}

/// Validate Pyth price account data format
pub fn validate_pyth_price_data(
    price_account_data: &[u8],
    expected_product_id: &[u8; 32],
) -> Result<bool> {
    // Basic Pyth price account validation
    require!(
        price_account_data.len() >= 208, // Minimum Pyth price account size
        InsuranceError::InvalidOracleData
    );
    
    // Validate magic number (first 4 bytes should be Pyth magic)
    let magic = u32::from_le_bytes([
        price_account_data[0],
        price_account_data[1], 
        price_account_data[2],
        price_account_data[3]
    ]);
    
    // Pyth magic number: 0xa1b2c3d4
    require!(
        magic == 0xa1b2c3d4,
        InsuranceError::InvalidOracleData
    );
    
    // Additional validation can be added here for product ID matching
    // if needed for specific insurance products
    
    Ok(true)
}

/// Extract price data from Pyth price account
pub fn extract_pyth_price_data(price_account_data: &[u8]) -> Result<(i64, u64, i64)> {
    // Validate account format first
    validate_pyth_price_data(price_account_data, &[0; 32])?;
    
    // Extract price (bytes 208-215)
    let price = i64::from_le_bytes([
        price_account_data[208],
        price_account_data[209],
        price_account_data[210], 
        price_account_data[211],
        price_account_data[212],
        price_account_data[213],
        price_account_data[214],
        price_account_data[215],
    ]);
    
    // Extract confidence (bytes 216-223)
    let confidence = u64::from_le_bytes([
        price_account_data[216],
        price_account_data[217],
        price_account_data[218],
        price_account_data[219], 
        price_account_data[220],
        price_account_data[221],
        price_account_data[222],
        price_account_data[223],
    ]);
    
    // Extract timestamp (bytes 256-263)  
    let timestamp = i64::from_le_bytes([
        price_account_data[256],
        price_account_data[257],
        price_account_data[258],
        price_account_data[259],
        price_account_data[260], 
        price_account_data[261],
        price_account_data[262],
        price_account_data[263],
    ]);
    
    Ok((price, confidence, timestamp))
}

pub fn update_oracle_status(ctx: Context<UpdateOracleStatus>, is_active: bool) -> Result<()> {
    let oracle = &mut ctx.accounts.oracle;
    oracle.is_active = is_active;
    Ok(())
}

/// Get consensus data from multiple oracles
pub fn get_consensus_data(
    master_contract: &MasterInsuranceContract,
    oracle_accounts: &[Account<Oracle>],
) -> Result<Option<ConsensusData>> {
    let clock = Clock::get()?;
    
    // Check if we have minimum consensus threshold
    let active_oracles: Vec<_> = oracle_accounts
        .iter()
        .filter(|oracle| oracle.is_active && oracle.latest_data.is_some())
        .collect();
    
    require!(
        active_oracles.len() >= master_contract.min_consensus_threshold as usize,
        InsuranceError::InsufficientOracles
    );
    
    // Extract valid oracle values (not older than 10 minutes)
    let max_age = 10 * 60; // 10 minutes in seconds
    let mut valid_values = Vec::new();
    
    for oracle in active_oracles {
        if let Some(ref data) = oracle.latest_data {
            if clock.unix_timestamp - data.timestamp <= max_age {
                valid_values.push(data.value);
            }
        }
    }
    
    require!(
        valid_values.len() >= master_contract.min_consensus_threshold as usize,
        InsuranceError::InsufficientOracles
    );
    
    // Remove outliers (values beyond 2 standard deviations)
    let filtered_values = remove_outliers(&valid_values)?;
    
    require!(
        filtered_values.len() >= master_contract.min_consensus_threshold as usize,
        InsuranceError::InsufficientOracles
    );
    
    // Create consensus data
    let consensus = ConsensusData::from_oracle_values(&filtered_values, clock.unix_timestamp);
    
    Ok(Some(consensus))
}

/// Remove statistical outliers from oracle values
fn remove_outliers(values: &[u64]) -> Result<Vec<u64>> {
    if values.len() <= 2 {
        return Ok(values.to_vec());
    }
    
    // Calculate mean and standard deviation
    let mean = values.iter().sum::<u64>() / values.len() as u64;
    let variance = values
        .iter()
        .map(|&x| {
            let diff = if x > mean { x - mean } else { mean - x };
            diff * diff
        })
        .sum::<u64>() / values.len() as u64;
    
    let std_dev = ConsensusData::integer_sqrt(variance);
    
    // Keep values within 2 standard deviations
    let threshold = std_dev * 2;
    let lower_bound = if mean > threshold { mean - threshold } else { 0 };
    let upper_bound = mean + threshold;
    
    let filtered: Vec<u64> = values
        .iter()
        .filter(|&&value| value >= lower_bound && value <= upper_bound)
        .copied()
        .collect();
    
    Ok(filtered)
}

/// Check consensus timeout for missing oracle data
pub fn check_consensus_timeout(
    oracle_accounts: &[Account<Oracle>],
    timeout_seconds: i64,
) -> Result<bool> {
    let clock = Clock::get()?;
    
    for oracle in oracle_accounts {
        if oracle.is_active {
            let time_since_update = clock.unix_timestamp - oracle.last_update_timestamp;
            if time_since_update > timeout_seconds {
                return Ok(true); // Timeout detected
            }
        }
    }
    
    Ok(false) // No timeout
}

/// Validate consensus data meets minimum requirements
pub fn validate_consensus_requirements(
    consensus: &ConsensusData,
    min_confidence: u8,
    min_oracles: u8,
) -> Result<bool> {
    require!(
        consensus.confidence_score >= min_confidence,
        InsuranceError::OracleConsensusFailure
    );
    
    require!(
        consensus.oracle_count >= min_oracles,
        InsuranceError::InsufficientOracles
    );
    
    Ok(true)
}

/// Check for price/data manipulation and reasonableness
pub fn validate_data_reasonableness(
    oracle: &Oracle,
    new_data: &OracleData,
    max_change_percentage: u8,
) -> Result<bool> {
    // Check if circuit breaker is active
    require!(
        !oracle.health_metrics.circuit_breaker_active,
        InsuranceError::OracleConsensusFailure
    );
    
    // Check for extreme value swings (max 50% change per update)
    if let Some(ref last_data) = oracle.latest_data {
        let percentage_change = calculate_percentage_change(last_data.value, new_data.value);
        require!(
            percentage_change <= max_change_percentage,
            InsuranceError::InvalidOracleData
        );
    }
    
    // Validate confidence level
    require!(
        new_data.confidence > 0,
        InsuranceError::InvalidOracleData
    );
    
    Ok(true)
}

/// Calculate percentage change between two values
fn calculate_percentage_change(old_value: u64, new_value: u64) -> u8 {
    if old_value == 0 {
        return 100; // Max change if starting from 0
    }
    
    let difference = if new_value > old_value {
        new_value - old_value
    } else {
        old_value - new_value
    };
    
    let percentage = (difference * 100) / old_value;
    std::cmp::min(percentage as u8, 100)
}

/// Update oracle health metrics and reputation score
pub fn update_oracle_health(oracle: &mut Oracle, success: bool, current_timestamp: i64) -> Result<()> {
    if success {
        oracle.health_metrics.record_successful_update(current_timestamp);
        
        // Improve reputation score for successful updates
        if oracle.reputation_score < 100 {
            oracle.reputation_score = std::cmp::min(100, oracle.reputation_score + 1);
        }
    } else {
        oracle.health_metrics.record_failed_validation(current_timestamp);
        
        // Decrease reputation score for failures
        oracle.reputation_score = oracle.reputation_score.saturating_sub(3);
    }
    
    Ok(())
}

/// Emergency override for oracle data correction (admin only)
#[derive(Accounts)]
pub struct EmergencyOracleOverride<'info> {
    #[account(
        mut,
        seeds = [b"oracle", oracle.oracle_id.as_bytes()],
        bump = oracle.bump
    )]
    pub oracle: Account<'info, Oracle>,
    
    #[account(
        constraint = master_contract.authority == admin.key() @ InsuranceError::Unauthorized
    )]
    pub master_contract: Account<'info, MasterInsuranceContract>,
    
    pub admin: Signer<'info>,
}

pub fn emergency_oracle_override(
    ctx: Context<EmergencyOracleOverride>,
    corrected_data: OracleData,
    reason: String,
) -> Result<()> {
    let oracle = &mut ctx.accounts.oracle;
    let clock = Clock::get()?;
    
    // Log the override for governance transparency
    msg!("Emergency oracle override - Oracle: {}, Reason: {}", oracle.oracle_id, reason);
    
    // Apply corrected data
    oracle.latest_data = Some(corrected_data);
    oracle.last_update_timestamp = clock.unix_timestamp;
    
    // Reset circuit breaker if active
    oracle.health_metrics.circuit_breaker_active = false;
    
    // Mark as administrative override in metrics
    oracle.health_metrics.failed_validations = 0;
    
    Ok(())
}

/// Check if oracle system has sufficient health for operations
pub fn check_oracle_system_health(
    oracle_accounts: &[Account<Oracle>],
    min_healthy_oracles: u8,
) -> Result<bool> {
    let healthy_oracles = oracle_accounts
        .iter()
        .filter(|oracle| {
            oracle.is_active && 
            !oracle.health_metrics.circuit_breaker_active &&
            oracle.reputation_score >= 70
        })
        .count();
    
    require!(
        healthy_oracles >= min_healthy_oracles as usize,
        InsuranceError::InsufficientOracles
    );
    
    Ok(true)
}

/// Reset circuit breaker for a specific oracle (admin only)
#[derive(Accounts)]
pub struct ResetOracleCircuitBreaker<'info> {
    #[account(
        mut,
        seeds = [b"oracle", oracle.oracle_id.as_bytes()],
        bump = oracle.bump
    )]
    pub oracle: Account<'info, Oracle>,
    
    #[account(
        constraint = master_contract.authority == admin.key() @ InsuranceError::Unauthorized
    )]
    pub master_contract: Account<'info, MasterInsuranceContract>,
    
    pub admin: Signer<'info>,
}

pub fn reset_oracle_circuit_breaker(ctx: Context<ResetOracleCircuitBreaker>) -> Result<()> {
    let oracle = &mut ctx.accounts.oracle;
    
    oracle.health_metrics.circuit_breaker_active = false;
    oracle.health_metrics.failed_validations = 0;
    
    msg!("Circuit breaker reset for oracle: {}", oracle.oracle_id);
    
    Ok(())
}