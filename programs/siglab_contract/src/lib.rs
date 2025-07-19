use anchor_lang::prelude::*;

declare_id!("8epbA4eCd1ieFndY5y8gZzNqmu91rMUdaY3rDVX5tZKj");

pub mod constants;
pub mod error;
pub mod events;
pub mod instructions;
pub mod state;

use instructions::*;
use state::*;

#[program]
pub mod siglab_contract {
    use super::*;

    pub fn initialize_master_contract(
        ctx: Context<InitializeMasterContract>,
        params: InitializeParams,
    ) -> Result<()> {
        instructions::admin::initialize_master_contract(ctx, params)
    }

    pub fn create_policy(
        ctx: Context<CreatePolicy>,
        params: CreatePolicyParams,
    ) -> Result<()> {
        instructions::policy::create_policy(ctx, params)
    }

    pub fn pay_premium(
        ctx: Context<PayPremium>,
        amount: u64,
    ) -> Result<()> {
        instructions::policy::pay_premium(ctx, amount)
    }

    pub fn trigger_payout(
        ctx: Context<TriggerPayout>,
        oracle_data: OracleData,
    ) -> Result<()> {
        instructions::payout::trigger_payout(ctx, oracle_data)
    }

    pub fn update_oracle_data(
        ctx: Context<UpdateOracleData>,
        data: OracleData,
    ) -> Result<()> {
        instructions::oracle::update_oracle_data(ctx, data)
    }

    pub fn pause_contract(ctx: Context<PauseContract>) -> Result<()> {
        instructions::admin::pause_contract(ctx)
    }

    pub fn resume_contract(ctx: Context<ResumeContract>) -> Result<()> {
        instructions::admin::resume_contract(ctx)
    }

    pub fn withdraw_treasury(
        ctx: Context<WithdrawTreasury>,
        amount: u64,
    ) -> Result<()> {
        instructions::admin::withdraw_treasury(ctx, amount)
    }
}
