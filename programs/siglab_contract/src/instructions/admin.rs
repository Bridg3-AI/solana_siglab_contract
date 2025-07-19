use anchor_lang::prelude::*;
use crate::error::InsuranceError;
use crate::utils::error_utils::*;
use crate::{require_authorized, require_not_paused};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeParams {
    pub reserve_ratio: u64,
}

#[derive(Accounts)]
pub struct InitializeMasterContract<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PauseContract<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct ResumeContract<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct WithdrawTreasury<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
}

pub fn initialize_master_contract(
    ctx: Context<InitializeMasterContract>,
    params: InitializeParams,
) -> Result<()> {
    require!(
        params.reserve_ratio >= 10 && params.reserve_ratio <= 50,
        InsuranceError::InvalidParameters
    );
    
    msg!("Initializing master contract with reserve ratio: {}%", params.reserve_ratio);
    Ok(())
}

pub fn pause_contract(ctx: Context<PauseContract>) -> Result<()> {
    require_authorized!(true); // TODO: Add actual authority check
    msg!("Contract paused by admin");
    Ok(())
}

pub fn resume_contract(ctx: Context<ResumeContract>) -> Result<()> {
    require_authorized!(true); // TODO: Add actual authority check
    msg!("Contract resumed by admin");
    Ok(())
}

pub fn withdraw_treasury(ctx: Context<WithdrawTreasury>, amount: u64) -> Result<()> {
    require_authorized!(true); // TODO: Add actual authority check
    require!(amount > 0, InsuranceError::InvalidParameters);
    
    msg!("Treasury withdrawal requested: {} lamports", amount);
    Ok(())
}