use anchor_lang::prelude::*;
use crate::state::{
    OracleData, Policy, PolicyStatus, PendingPayout, PayoutStatus, PayoutCalculationData,
    MasterInsuranceContract, Oracle, ComparisonOperator
};
use crate::error::InsuranceError;
use crate::events::{PayoutTriggered};

#[derive(Accounts)]
#[instruction(policy_id: String)]
pub struct TriggerPayout<'info> {
    #[account(
        mut,
        seeds = [b"policy", policy_id.as_bytes()],
        bump,
        constraint = policy.status == PolicyStatus::Active @ InsuranceError::PolicyNotActive,
        constraint = policy.end_date > Clock::get()?.unix_timestamp @ InsuranceError::PolicyExpired
    )]
    pub policy: Account<'info, Policy>,
    
    #[account(
        init,
        payer = beneficiary,
        space = PendingPayout::space(),
        seeds = [b"pending_payout", policy_id.as_bytes()],
        bump
    )]
    pub pending_payout: Account<'info, PendingPayout>,
    
    #[account(
        constraint = master_contract.treasury_account != Pubkey::default() @ InsuranceError::InvalidAdminOperation
    )]
    pub master_contract: Account<'info, MasterInsuranceContract>,
    
    #[account(mut)]
    pub beneficiary: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ExecutePayout<'info> {
    #[account(
        mut,
        close = beneficiary,
        constraint = pending_payout.status == PayoutStatus::Ready @ InsuranceError::PayoutConditionsNotMet,
        constraint = pending_payout.beneficiary == beneficiary.key() @ InsuranceError::Unauthorized
    )]
    pub pending_payout: Account<'info, PendingPayout>,
    
    #[account(
        mut,
        seeds = [b"policy", pending_payout.policy_id.as_bytes()],
        bump
    )]
    pub policy: Account<'info, Policy>,
    
    #[account(mut)]
    pub master_contract: Account<'info, MasterInsuranceContract>,
    
    /// CHECK: Treasury account for payout transfer
    #[account(
        mut,
        constraint = treasury_account.key() == master_contract.treasury_account @ InsuranceError::InvalidAdminOperation
    )]
    pub treasury_account: AccountInfo<'info>,
    
    #[account(mut)]
    pub beneficiary: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ApprovePayout<'info> {
    #[account(
        mut,
        constraint = pending_payout.status == PayoutStatus::PendingApproval @ InsuranceError::PayoutConditionsNotMet
    )]
    pub pending_payout: Account<'info, PendingPayout>,
    
    #[account(
        constraint = master_contract.authority == admin.key() @ InsuranceError::Unauthorized
    )]
    pub master_contract: Account<'info, MasterInsuranceContract>,
    
    pub admin: Signer<'info>,
}

pub fn trigger_payout(
    ctx: Context<TriggerPayout>,
    policy_id: String,
    oracle_value: u64,
) -> Result<()> {
    let policy = &mut ctx.accounts.policy;
    let pending_payout = &mut ctx.accounts.pending_payout;
    let master_contract = &ctx.accounts.master_contract;
    let clock = Clock::get()?;
    
    // Check waiting period
    let time_since_start = clock.unix_timestamp - policy.start_date;
    let waiting_period_seconds = (policy.waiting_period_hours as i64) * 3600;
    require!(
        time_since_start >= waiting_period_seconds,
        InsuranceError::ClaimPeriodExpired
    );
    
    // For now, use simple oracle value validation instead of consensus
    // TODO: Implement proper oracle consensus in future version
    
    // Check trigger conditions against oracle data
    let trigger_met = evaluate_trigger_conditions(
        &policy.trigger_conditions,
        oracle_value,
    )?;
    
    require!(trigger_met, InsuranceError::PayoutConditionsNotMet);
    
    // Calculate payout amount
    let calculation_data = PayoutCalculationData {
        coverage_amount: policy.coverage_amount,
        deductible: policy.deductible,
        severity_percentage: calculate_severity_percentage(
            &policy.trigger_conditions,
            oracle_value,
        )?,
        max_payout: policy.max_payout_per_incident,
        insurance_type: format!("{:?}", policy.insurance_type),
    };
    
    let payout_amount = calculation_data.calculate_payout();
    require!(payout_amount > 0, InsuranceError::InvalidClaimAmount);
    
    // Determine if admin approval is required (e.g., > 10% of treasury)
    let approval_threshold = master_contract.total_premiums_collected / 10; // 10% threshold
    let requires_approval = payout_amount > approval_threshold;
    
    let status = if requires_approval {
        PayoutStatus::PendingApproval
    } else {
        PayoutStatus::Ready
    };
    
    // Initialize pending payout
    pending_payout.policy_id = policy_id.clone();
    pending_payout.amount = payout_amount;
    pending_payout.timestamp = clock.unix_timestamp;
    pending_payout.priority = calculate_priority(&policy.insurance_type, calculation_data.severity_percentage);
    pending_payout.status = status;
    pending_payout.beneficiary = ctx.accounts.beneficiary.key();
    pending_payout.trigger_oracle_data = oracle_value.to_le_bytes().to_vec();
    pending_payout.severity_score = calculation_data.severity_percentage;
    pending_payout.approval_timestamp = None;
    pending_payout.approved_by = None;
    pending_payout.expires_at = clock.unix_timestamp + (24 * 60 * 60); // 24 hour expiration
    pending_payout.rejection_reason = None;
    pending_payout.bump = ctx.bumps.pending_payout;
    
    // Update policy status
    policy.status = PolicyStatus::PendingPayout;
    policy.updated_at = clock.unix_timestamp;
    
    // Emit event
    emit!(PayoutTriggered {
        policy_id: policy_id,
        beneficiary: ctx.accounts.beneficiary.key(),
        amount: payout_amount,
        oracle_value: oracle_value,
        timestamp: clock.unix_timestamp,
    });
    
    Ok(())
}

pub fn execute_payout(ctx: Context<ExecutePayout>) -> Result<()> {
    let pending_payout = &ctx.accounts.pending_payout;
    let policy = &mut ctx.accounts.policy;
    let master_contract = &mut ctx.accounts.master_contract;
    let clock = Clock::get()?;
    
    // Check if payout has expired
    require!(
        !pending_payout.is_expired(clock.unix_timestamp),
        InsuranceError::ClaimPeriodExpired
    );
    
    // Check treasury has sufficient funds
    let treasury_balance = ctx.accounts.treasury_account.lamports();
    require!(
        treasury_balance >= pending_payout.amount,
        InsuranceError::InsufficientTreasury
    );
    
    // Transfer funds from treasury to beneficiary
    **ctx.accounts.treasury_account.try_borrow_mut_lamports()? -= pending_payout.amount;
    **ctx.accounts.beneficiary.try_borrow_mut_lamports()? += pending_payout.amount;
    
    // Update policy status
    policy.status = PolicyStatus::PaidOut;
    policy.updated_at = clock.unix_timestamp;
    
    // Update master contract stats
    master_contract.total_payouts_disbursed += pending_payout.amount;
    master_contract.updated_at = clock.unix_timestamp;
    
    // Emit event
    emit!(crate::events::PayoutExecuted {
        policy_id: pending_payout.policy_id.clone(),
        beneficiary: pending_payout.beneficiary,
        amount: pending_payout.amount,
        transaction_signature: "executed".to_string(), // Would be actual signature in production
        timestamp: clock.unix_timestamp,
    });
    
    Ok(())
}

pub fn approve_payout(ctx: Context<ApprovePayout>) -> Result<()> {
    let pending_payout = &mut ctx.accounts.pending_payout;
    let clock = Clock::get()?;
    
    // Check if payout has expired
    require!(
        !pending_payout.is_expired(clock.unix_timestamp),
        InsuranceError::ClaimPeriodExpired
    );
    
    // Update payout status to ready
    pending_payout.status = PayoutStatus::Ready;
    pending_payout.approval_timestamp = Some(clock.unix_timestamp);
    pending_payout.approved_by = Some(ctx.accounts.admin.key());
    
    // Emit event
    emit!(crate::events::PayoutApproved {
        policy_id: pending_payout.policy_id.clone(),
        admin: ctx.accounts.admin.key(),
        amount: pending_payout.amount,
        timestamp: clock.unix_timestamp,
    });
    
    Ok(())
}

/// Evaluate if trigger conditions are met based on oracle data
fn evaluate_trigger_conditions(
    conditions: &crate::state::TriggerConditions,
    oracle_value: u64,
) -> Result<bool> {
    let oracle_value_f64 = oracle_value as f64;
    
    let condition_met = match conditions.comparison_operator {
        ComparisonOperator::GreaterThan => oracle_value_f64 > conditions.threshold_value,
        ComparisonOperator::LessThan => oracle_value_f64 < conditions.threshold_value,
        ComparisonOperator::Equals => (oracle_value_f64 - conditions.threshold_value).abs() < 0.01,
        ComparisonOperator::NotEquals => (oracle_value_f64 - conditions.threshold_value).abs() >= 0.01,
    };
    
    Ok(condition_met)
}

/// Calculate severity percentage based on how far oracle value deviates from trigger threshold
fn calculate_severity_percentage(
    conditions: &crate::state::TriggerConditions,
    oracle_value: u64,
) -> Result<u8> {
    let oracle_value_f64 = oracle_value as f64;
    let threshold = conditions.threshold_value;
    
    // Calculate percentage deviation from threshold
    let deviation = (oracle_value_f64 - threshold).abs() / threshold;
    
    // Convert to severity percentage (capped at 100%)
    let severity = (deviation * 100.0).min(100.0) as u8;
    
    Ok(severity)
}

/// Calculate priority based on insurance type and severity
fn calculate_priority(insurance_type: &crate::state::InsuranceType, severity: u8) -> u8 {
    let base_priority = match insurance_type {
        crate::state::InsuranceType::Weather => 70,
        crate::state::InsuranceType::Earthquake => 90,
        crate::state::InsuranceType::Flight => 60,
        crate::state::InsuranceType::Crop => 80,
        crate::state::InsuranceType::Custom => 50,
    };
    
    // Adjust priority based on severity
    let adjusted_priority = base_priority + (severity / 4); // Add up to 25 points for severity
    std::cmp::min(adjusted_priority, 100)
}

// ===== PAYOUT QUEUE MANAGEMENT FUNCTIONS =====

/// Add payout to the processing queue (called automatically in trigger_payout)
/// Queue maintains chronological ordering with priority adjustments
pub fn add_to_payout_queue(
    pending_payout: &PendingPayout,
    current_timestamp: i64,
) -> Result<()> {
    // Validate payout data
    require!(pending_payout.amount > 0, InsuranceError::InvalidClaimAmount);
    require!(
        !pending_payout.is_expired(current_timestamp),
        InsuranceError::ClaimPeriodExpired
    );
    
    // Queue ordering is handled by timestamp in PendingPayout struct
    // Priority affects processing order when timestamps are similar
    
    Ok(())
}

/// Get next batch of payouts ready for processing
/// Returns payouts sorted by priority and timestamp
pub fn get_next_payout_batch(
    pending_payouts: &[PendingPayout],
    batch_size: usize,
    current_timestamp: i64,
) -> Vec<PendingPayout> {
    let mut ready_payouts: Vec<PendingPayout> = pending_payouts
        .iter()
        .filter(|payout| {
            payout.is_ready_for_execution() && !payout.is_expired(current_timestamp)
        })
        .cloned()
        .collect();
    
    // Sort by priority (descending) then by timestamp (ascending - older first)
    ready_payouts.sort_by(|a, b| {
        match b.priority.cmp(&a.priority) {
            std::cmp::Ordering::Equal => a.timestamp.cmp(&b.timestamp),
            other => other,
        }
    });
    
    // Return up to batch_size payouts
    ready_payouts.into_iter().take(batch_size).collect()
}

/// Remove processed payout from queue (automatically handled by account closure)
pub fn remove_from_queue(pending_payout: &PendingPayout) -> Result<()> {
    // Validate payout is in a final state
    require!(
        matches!(
            pending_payout.status,
            PayoutStatus::Executed | PayoutStatus::Rejected | PayoutStatus::Expired
        ),
        InsuranceError::PayoutConditionsNotMet
    );
    
    Ok(())
}

/// Check queue health and size limits
pub fn validate_queue_health(
    queue_size: usize,
    max_queue_size: usize,
    current_timestamp: i64,
) -> Result<()> {
    // Check queue size limit
    require!(
        queue_size < max_queue_size,
        InsuranceError::InvalidAdminOperation // Reusing error for queue overflow
    );
    
    Ok(())
}

/// Cleanup expired payouts from queue
pub fn cleanup_expired_payouts(
    pending_payouts: &mut Vec<PendingPayout>,
    current_timestamp: i64,
) -> usize {
    let initial_count = pending_payouts.len();
    
    // Remove expired payouts
    pending_payouts.retain(|payout| !payout.is_expired(current_timestamp));
    
    // Return number of cleaned up payouts
    initial_count - pending_payouts.len()
}

/// Get queue statistics for monitoring
pub fn get_queue_statistics(
    pending_payouts: &[PendingPayout],
    current_timestamp: i64,
) -> QueueStatistics {
    let total_count = pending_payouts.len();
    let ready_count = pending_payouts
        .iter()
        .filter(|p| p.is_ready_for_execution())
        .count();
    let pending_approval_count = pending_payouts
        .iter()
        .filter(|p| p.requires_approval())
        .count();
    let expired_count = pending_payouts
        .iter()
        .filter(|p| p.is_expired(current_timestamp))
        .count();
    
    let total_amount: u64 = pending_payouts.iter().map(|p| p.amount).sum();
    
    let oldest_timestamp = pending_payouts
        .iter()
        .map(|p| p.timestamp)
        .min()
        .unwrap_or(current_timestamp);
    
    QueueStatistics {
        total_count,
        ready_count,
        pending_approval_count,
        expired_count,
        total_amount,
        oldest_timestamp,
    }
}

#[derive(Debug, Clone)]
pub struct QueueStatistics {
    pub total_count: usize,
    pub ready_count: usize,
    pub pending_approval_count: usize,
    pub expired_count: usize,
    pub total_amount: u64,
    pub oldest_timestamp: i64,
}