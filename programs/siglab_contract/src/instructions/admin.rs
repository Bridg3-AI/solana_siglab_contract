use anchor_lang::prelude::*;
use crate::state::{MasterInsuranceContract, Treasury};
use crate::error::InsuranceError;
use crate::events::{ContractPaused, ContractResumed, ReserveRatioUpdated, TreasuryWithdrawn};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeParams {
    pub reserve_ratio: u64,
    pub max_oracles: u8,
    pub min_consensus_threshold: u8,
}

#[derive(Accounts)]
pub struct InitializeMasterContract<'info> {
    #[account(
        init,
        payer = admin,
        space = MasterInsuranceContract::space(),
        seeds = [b"master_contract"],
        bump
    )]
    pub master_contract: Account<'info, MasterInsuranceContract>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PauseContract<'info> {
    #[account(
        mut,
        seeds = [b"master_contract"],
        bump = master_contract.bump,
        constraint = master_contract.authority == admin.key() @ InsuranceError::Unauthorized,
        constraint = !master_contract.is_paused @ InsuranceError::ContractPaused
    )]
    pub master_contract: Account<'info, MasterInsuranceContract>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct ResumeContract<'info> {
    #[account(
        mut,
        seeds = [b"master_contract"],
        bump = master_contract.bump,
        constraint = master_contract.authority == admin.key() @ InsuranceError::Unauthorized,
        constraint = master_contract.is_paused @ InsuranceError::ContractMustBePaused
    )]
    pub master_contract: Account<'info, MasterInsuranceContract>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateReserveRatio<'info> {
    #[account(
        mut,
        seeds = [b"master_contract"],
        bump = master_contract.bump,
        constraint = master_contract.authority == admin.key() @ InsuranceError::Unauthorized
    )]
    pub master_contract: Account<'info, MasterInsuranceContract>,
    
    #[account(
        mut,
        seeds = [b"treasury"],
        bump = treasury.bump
    )]
    pub treasury: Account<'info, Treasury>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct WithdrawTreasury<'info> {
    #[account(
        mut,
        seeds = [b"master_contract"],
        bump = master_contract.bump,
        constraint = master_contract.authority == admin.key() @ InsuranceError::Unauthorized
    )]
    pub master_contract: Account<'info, MasterInsuranceContract>,
    
    #[account(
        mut,
        seeds = [b"treasury"],
        bump = treasury.bump
    )]
    pub treasury: Account<'info, Treasury>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
    
    /// CHECK: Recipient account for withdrawal
    pub recipient: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct TransferAuthority<'info> {
    #[account(
        mut,
        seeds = [b"master_contract"],
        bump = master_contract.bump,
        constraint = master_contract.authority == current_admin.key() @ InsuranceError::Unauthorized
    )]
    pub master_contract: Account<'info, MasterInsuranceContract>,
    
    #[account(mut)]
    pub current_admin: Signer<'info>,
    
    /// CHECK: New admin account
    pub new_admin: AccountInfo<'info>,
}

pub fn initialize_master_contract(
    ctx: Context<InitializeMasterContract>,
    params: InitializeParams,
) -> Result<()> {
    let master_contract = &mut ctx.accounts.master_contract;
    let clock = Clock::get()?;
    
    // Validate parameters
    require!(
        params.reserve_ratio >= 10 && params.reserve_ratio <= 50,
        InsuranceError::InvalidInput
    );
    require!(
        params.max_oracles >= 1 && params.max_oracles <= 10,
        InsuranceError::InvalidInput
    );
    require!(
        params.min_consensus_threshold >= 1 && params.min_consensus_threshold <= params.max_oracles,
        InsuranceError::InvalidInput
    );
    
    // Initialize master contract
    master_contract.authority = ctx.accounts.admin.key();
    master_contract.policies = Vec::new();
    master_contract.treasury_account = Pubkey::default(); // Will be set when treasury is initialized
    master_contract.total_premiums_collected = 0;
    master_contract.total_payouts_disbursed = 0;
    master_contract.active_policies_count = 0;
    master_contract.reserve_ratio = params.reserve_ratio;
    master_contract.is_paused = false;
    master_contract.created_at = clock.unix_timestamp;
    master_contract.updated_at = clock.unix_timestamp;
    master_contract.oracle_registry = Vec::new();
    master_contract.max_oracles = params.max_oracles;
    master_contract.min_consensus_threshold = params.min_consensus_threshold;
    master_contract.bump = ctx.bumps.master_contract;
    
    msg!("Master contract initialized with reserve ratio: {}%", params.reserve_ratio);
    Ok(())
}

pub fn pause_contract(ctx: Context<PauseContract>) -> Result<()> {
    let master_contract = &mut ctx.accounts.master_contract;
    let clock = Clock::get()?;
    
    master_contract.is_paused = true;
    master_contract.updated_at = clock.unix_timestamp;
    
    emit!(ContractPaused {
        admin: ctx.accounts.admin.key(),
        timestamp: clock.unix_timestamp,
    });
    
    msg!("Contract paused by admin: {}", ctx.accounts.admin.key());
    Ok(())
}

pub fn resume_contract(ctx: Context<ResumeContract>) -> Result<()> {
    let master_contract = &mut ctx.accounts.master_contract;
    let clock = Clock::get()?;
    
    master_contract.is_paused = false;
    master_contract.updated_at = clock.unix_timestamp;
    
    emit!(ContractResumed {
        admin: ctx.accounts.admin.key(),
        timestamp: clock.unix_timestamp,
    });
    
    msg!("Contract resumed by admin: {}", ctx.accounts.admin.key());
    Ok(())
}

pub fn update_reserve_ratio(
    ctx: Context<UpdateReserveRatio>,
    new_reserve_ratio: u64,
) -> Result<()> {
    let master_contract = &mut ctx.accounts.master_contract;
    let treasury = &mut ctx.accounts.treasury;
    let clock = Clock::get()?;
    
    // Validate new reserve ratio
    require!(
        new_reserve_ratio >= 10 && new_reserve_ratio <= 50,
        InsuranceError::InvalidInput
    );
    
    // Check that the new ratio doesn't violate current solvency
    let total_balance = treasury.total_usdc_balance + treasury.total_sol_balance;
    if treasury.total_coverage_exposure > 0 {
        let required_reserves = (treasury.total_coverage_exposure * new_reserve_ratio) / 100;
        require!(
            total_balance >= required_reserves,
            InsuranceError::ReserveRatioViolation
        );
    }
    
    let old_ratio = master_contract.reserve_ratio;
    master_contract.reserve_ratio = new_reserve_ratio;
    master_contract.updated_at = clock.unix_timestamp;
    
    // Update treasury minimum reserve ratio
    treasury.minimum_reserve_ratio = (new_reserve_ratio * 100) as u16; // Convert to basis points
    treasury.current_reserve_ratio = treasury.calculate_reserve_ratio();
    treasury.last_update_timestamp = clock.unix_timestamp;
    
    emit!(ReserveRatioUpdated {
        admin: ctx.accounts.admin.key(),
        old_ratio,
        new_ratio: new_reserve_ratio,
        timestamp: clock.unix_timestamp,
    });
    
    msg!("Reserve ratio updated from {}% to {}%", old_ratio, new_reserve_ratio);
    Ok(())
}

pub fn withdraw_treasury(
    ctx: Context<WithdrawTreasury>,
    amount: u64,
    token_type: crate::state::TokenType,
) -> Result<()> {
    let master_contract = &mut ctx.accounts.master_contract;
    let treasury = &mut ctx.accounts.treasury;
    let clock = Clock::get()?;
    
    require!(amount > 0, InsuranceError::InvalidInput);
    
    // Check available balance
    match token_type {
        crate::state::TokenType::USDC => {
            require!(
                treasury.total_usdc_balance >= amount,
                InsuranceError::InsufficientTreasury
            );
        }
        crate::state::TokenType::SOL => {
            require!(
                treasury.total_sol_balance >= amount,
                InsuranceError::InsufficientTreasury
            );
        }
    }
    
    // Check that withdrawal doesn't violate reserve requirements
    let available_liquidity = treasury.available_liquidity();
    require!(
        amount <= available_liquidity,
        InsuranceError::ReserveRatioViolation
    );
    
    // Update treasury balances (in a full implementation, this would include actual transfers)
    match token_type {
        crate::state::TokenType::USDC => {
            treasury.total_usdc_balance -= amount;
        }
        crate::state::TokenType::SOL => {
            treasury.total_sol_balance -= amount;
        }
    }
    
    treasury.withdrawal_count += 1;
    treasury.current_reserve_ratio = treasury.calculate_reserve_ratio();
    treasury.last_update_timestamp = clock.unix_timestamp;
    master_contract.updated_at = clock.unix_timestamp;
    
    emit!(TreasuryWithdrawn {
        admin: ctx.accounts.admin.key(),
        amount,
        timestamp: clock.unix_timestamp,
    });
    
    msg!("Treasury withdrawal: {} tokens by admin: {}", amount, ctx.accounts.admin.key());
    Ok(())
}

pub fn transfer_authority(
    ctx: Context<TransferAuthority>,
) -> Result<()> {
    let master_contract = &mut ctx.accounts.master_contract;
    let clock = Clock::get()?;
    
    let old_authority = master_contract.authority;
    master_contract.authority = ctx.accounts.new_admin.key();
    master_contract.updated_at = clock.unix_timestamp;
    
    msg!("Authority transferred from {} to {}", old_authority, ctx.accounts.new_admin.key());
    Ok(())
}

/// Helper function to check if contract is paused
pub fn require_not_paused(master_contract: &MasterInsuranceContract) -> Result<()> {
    require!(!master_contract.is_paused, InsuranceError::ContractPaused);
    Ok(())
}

/// Helper function to check admin authorization
pub fn require_admin_authority(master_contract: &MasterInsuranceContract, admin: &Pubkey) -> Result<()> {
    require!(master_contract.authority == *admin, InsuranceError::Unauthorized);
    Ok(())
}