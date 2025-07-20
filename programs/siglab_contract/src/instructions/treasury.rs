use anchor_lang::prelude::*;
use crate::state::{Treasury, TokenType, WithdrawalReason};
use crate::error::InsuranceError;
use crate::events::{TreasuryWithdrawn};

#[derive(Accounts)]
pub struct InitializeTreasury<'info> {
    #[account(
        init,
        payer = admin,
        space = Treasury::space(),
        seeds = [b"treasury"],
        bump
    )]
    pub treasury: Account<'info, Treasury>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
    
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct DepositFunds<'info> {
    #[account(
        mut,
        seeds = [b"treasury"],
        bump = treasury.bump
    )]
    pub treasury: Account<'info, Treasury>,
    
    #[account(mut)]
    pub depositor: Signer<'info>,
}

#[derive(Accounts)]
pub struct WithdrawFunds<'info> {
    #[account(
        mut,
        seeds = [b"treasury"],
        bump = treasury.bump,
        constraint = treasury.authority == admin.key() @ InsuranceError::Unauthorized
    )]
    pub treasury: Account<'info, Treasury>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
    
    /// CHECK: Recipient account for withdrawal
    pub recipient: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct UpdateTreasuryBalance<'info> {
    #[account(
        mut,
        seeds = [b"treasury"],
        bump = treasury.bump
    )]
    pub treasury: Account<'info, Treasury>,
}

pub fn initialize_treasury(
    ctx: Context<InitializeTreasury>,
    minimum_reserve_ratio: u16,
) -> Result<()> {
    let treasury = &mut ctx.accounts.treasury;
    let clock = Clock::get()?;
    
    // Validate minimum reserve ratio (should be between 10% and 50%)
    require!(
        minimum_reserve_ratio >= 1000 && minimum_reserve_ratio <= 5000,
        InsuranceError::InvalidInput
    );
    
    // Initialize treasury
    treasury.authority = ctx.accounts.admin.key();
    treasury.usdc_token_account = Pubkey::default(); // Will be set later when tokens are integrated
    treasury.sol_token_account = Pubkey::default(); // Will be set later when tokens are integrated
    treasury.usdc_mint = Pubkey::default(); // Will be set later when tokens are integrated
    treasury.total_usdc_balance = 0;
    treasury.total_sol_balance = 0;
    treasury.total_premiums_collected_usdc = 0;
    treasury.total_premiums_collected_sol = 0;
    treasury.total_payouts_disbursed_usdc = 0;
    treasury.total_payouts_disbursed_sol = 0;
    treasury.current_reserve_ratio = 10000; // 100% (no exposure yet)
    treasury.minimum_reserve_ratio = minimum_reserve_ratio;
    treasury.total_coverage_exposure = 0;
    treasury.deposit_count = 0;
    treasury.withdrawal_count = 0;
    treasury.last_update_timestamp = clock.unix_timestamp;
    treasury.created_at = clock.unix_timestamp;
    treasury.bump = ctx.bumps.treasury;
    
    Ok(())
}

pub fn deposit_funds(
    ctx: Context<DepositFunds>,
    amount: u64,
    token_type: TokenType,
) -> Result<()> {
    let treasury = &mut ctx.accounts.treasury;
    let clock = Clock::get()?;
    
    require!(amount > 0, InsuranceError::InvalidInput);
    
    // For now, we'll just track the amounts in the treasury state
    // In a full implementation, this would include actual SPL token transfers
    match token_type {
        TokenType::USDC => {
            treasury.total_usdc_balance += amount;
        }
        TokenType::SOL => {
            treasury.total_sol_balance += amount;
        }
    }
    
    treasury.deposit_count += 1;
    treasury.current_reserve_ratio = treasury.calculate_reserve_ratio();
    treasury.last_update_timestamp = clock.unix_timestamp;
    
    Ok(())
}

pub fn withdraw_funds(
    ctx: Context<WithdrawFunds>,
    amount: u64,
    token_type: TokenType,
    reason: WithdrawalReason,
) -> Result<()> {
    let treasury = &mut ctx.accounts.treasury;
    let clock = Clock::get()?;
    
    require!(amount > 0, InsuranceError::InvalidInput);
    
    // Check available balance
    match token_type {
        TokenType::USDC => {
            require!(
                treasury.total_usdc_balance >= amount,
                InsuranceError::InsufficientTreasury
            );
        }
        TokenType::SOL => {
            require!(
                treasury.total_sol_balance >= amount,
                InsuranceError::InsufficientTreasury
            );
        }
    }
    
    // For admin withdrawals, check that it doesn't violate reserve requirements
    if matches!(reason, WithdrawalReason::AdminWithdrawal) {
        let available_liquidity = treasury.available_liquidity();
        require!(
            amount <= available_liquidity,
            InsuranceError::ReserveRatioViolation
        );
    }
    
    // Update treasury balances (in a full implementation, this would include actual transfers)
    match token_type {
        TokenType::USDC => {
            treasury.total_usdc_balance -= amount;
        }
        TokenType::SOL => {
            treasury.total_sol_balance -= amount;
        }
    }
    
    treasury.withdrawal_count += 1;
    treasury.current_reserve_ratio = treasury.calculate_reserve_ratio();
    treasury.last_update_timestamp = clock.unix_timestamp;
    
    // Emit withdrawal event
    emit!(TreasuryWithdrawn {
        admin: ctx.accounts.admin.key(),
        amount,
        timestamp: clock.unix_timestamp,
    });
    
    Ok(())
}

pub fn update_treasury_balance(ctx: Context<UpdateTreasuryBalance>) -> Result<()> {
    let treasury = &mut ctx.accounts.treasury;
    let clock = Clock::get()?;
    
    // Refresh reserve ratio calculation
    treasury.current_reserve_ratio = treasury.calculate_reserve_ratio();
    treasury.last_update_timestamp = clock.unix_timestamp;
    
    Ok(())
}

/// Validate treasury solvency before operations
pub fn validate_treasury_solvency(treasury: &Treasury, additional_exposure: u64) -> Result<()> {
    let new_exposure = treasury.total_coverage_exposure + additional_exposure;
    let total_balance = treasury.total_usdc_balance + treasury.total_sol_balance;
    
    if new_exposure > 0 {
        let required_reserves = (new_exposure * treasury.minimum_reserve_ratio as u64) / 10000;
        require!(
            total_balance >= required_reserves,
            InsuranceError::SolvencyCheckFailed
        );
    }
    
    Ok(())
}

/// Helper function for premium collection (to be used in policy instructions)
pub fn process_premium_payment(
    treasury: &mut Treasury,
    amount: u64,
    is_usdc: bool,
    timestamp: i64,
) -> Result<()> {
    treasury.record_premium(amount, is_usdc, timestamp);
    Ok(())
}

/// Helper function for payout disbursement (to be used in payout instructions)
pub fn process_payout_disbursement(
    treasury: &mut Treasury,
    amount: u64,
    is_usdc: bool,
    timestamp: i64,
) -> Result<()> {
    treasury.record_payout(amount, is_usdc, timestamp)?;
    Ok(())
}