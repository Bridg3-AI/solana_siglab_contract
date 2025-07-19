use anchor_lang::prelude::*;
use crate::error::InsuranceError;
use crate::state::*;
use crate::constants::*;
use crate::{require_not_paused, require_sufficient_premium};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreatePolicyParams {
    pub insurance_type: InsuranceType,
    pub coverage_amount: u64,
    pub premium_amount: u64,
    pub deductible: u64,
    pub policy_duration_days: u32,
    pub trigger_conditions: TriggerConditions,
    pub oracle_config: OracleConfig,
}

#[derive(Accounts)]
#[instruction(params: CreatePolicyParams)]
pub struct CreatePolicy<'info> {
    #[account(mut)]
    pub policy_holder: Signer<'info>,
    
    /// The master insurance contract account
    #[account(
        mut,
        seeds = [MASTER_CONTRACT_SEED],
        bump,
    )]
    pub master_contract: Account<'info, MasterInsuranceContract>,
    
    /// Policy account to be created
    #[account(
        init,
        payer = policy_holder,
        space = 8 + std::mem::size_of::<Policy>(),
        seeds = [POLICY_SEED, policy_holder.key().as_ref(), &master_contract.active_policies_count.to_le_bytes()],
        bump,
    )]
    pub policy_account: Account<'info, Policy>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PayPremium<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    #[account(mut)]
    pub policy_account: Account<'info, Policy>,
    
    #[account(mut)]
    pub master_contract: Account<'info, MasterInsuranceContract>,
}

pub fn create_policy(
    ctx: Context<CreatePolicy>,
    params: CreatePolicyParams,
) -> Result<()> {
    let master_contract = &mut ctx.accounts.master_contract;
    let policy_account = &mut ctx.accounts.policy_account;
    let policy_holder = &ctx.accounts.policy_holder;
    
    // Check contract is not paused
    require_not_paused!(master_contract.is_paused);
    
    // Validate parameters
    require!(
        params.coverage_amount > 0 && params.coverage_amount <= MAX_COVERAGE_AMOUNT,
        InsuranceError::CoverageExceedsMaximum
    );
    
    require_sufficient_premium!(params.premium_amount, MIN_PREMIUM_AMOUNT);
    
    require!(
        params.deductible <= params.coverage_amount,
        InsuranceError::InvalidParameters
    );
    
    require!(
        params.policy_duration_days > 0 && params.policy_duration_days <= 365,
        InsuranceError::InvalidParameters
    );
    
    // Generate unique policy ID
    let policy_id = format!("POL-{}-{}", 
        Clock::get()?.unix_timestamp,
        master_contract.active_policies_count
    );
    
    let current_time = Clock::get()?.unix_timestamp;
    let end_date = current_time + (params.policy_duration_days as i64 * 86400); // Convert days to seconds
    
    // Initialize policy
    policy_account.id = policy_id.clone();
    policy_account.user = policy_holder.key();
    policy_account.insurance_type = params.insurance_type;
    policy_account.coverage_amount = params.coverage_amount;
    policy_account.premium_amount = params.premium_amount;
    policy_account.deductible = params.deductible;
    policy_account.start_date = current_time;
    policy_account.end_date = end_date;
    policy_account.status = PolicyStatus::Active;
    policy_account.trigger_conditions = params.trigger_conditions;
    policy_account.oracle_config = params.oracle_config;
    policy_account.last_premium_paid = current_time;
    policy_account.payout_history = Vec::new();
    policy_account.created_at = current_time;
    policy_account.updated_at = current_time;
    
    // Update master contract
    master_contract.active_policies_count += 1;
    master_contract.updated_at = current_time;
    
    msg!("Policy created with ID: {} for user: {}", 
        policy_account.id, 
        policy_holder.key()
    );
    
    Ok(())
}

pub fn pay_premium(ctx: Context<PayPremium>, amount: u64) -> Result<()> {
    let policy_account = &mut ctx.accounts.policy_account;
    let master_contract = &mut ctx.accounts.master_contract;
    let payer = &ctx.accounts.payer;
    
    // Check contract is not paused
    require_not_paused!(master_contract.is_paused);
    
    // Validate policy is active
    require!(
        matches!(policy_account.status, PolicyStatus::Active),
        InsuranceError::PolicyNotActive
    );
    
    // Check policy hasn't expired
    let current_time = Clock::get()?.unix_timestamp;
    require!(
        current_time <= policy_account.end_date,
        InsuranceError::PolicyExpired
    );
    
    // Validate premium amount
    require!(
        amount >= policy_account.premium_amount,
        InsuranceError::InsufficientPremium
    );
    
    // Validate payer is policy holder
    require!(
        payer.key() == policy_account.user,
        InsuranceError::Unauthorized
    );
    
    // Update payment record
    policy_account.last_premium_paid = current_time;
    policy_account.updated_at = current_time;
    
    // Update master contract financial tracking
    master_contract.total_premiums_collected = master_contract
        .total_premiums_collected
        .checked_add(amount)
        .ok_or(InsuranceError::MathOverflow)?;
    
    master_contract.updated_at = current_time;
    
    msg!("Premium paid: {} lamports for policy: {}", amount, policy_account.id);
    
    Ok(())
}