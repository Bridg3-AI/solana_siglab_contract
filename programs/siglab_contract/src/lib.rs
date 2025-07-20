use anchor_lang::prelude::*;

declare_id!("8epbA4eCd1ieFndY5y8gZzNqmu91rMUdaY3rDVX5tZKj");

pub mod constants;
pub mod error;
pub mod events;
pub mod instructions;
pub mod state;
pub mod utils;

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
        policy_id: String,
        oracle_value: u64,
    ) -> Result<()> {
        instructions::payout::trigger_payout(ctx, policy_id, oracle_value)
    }

    pub fn execute_payout(ctx: Context<ExecutePayout>) -> Result<()> {
        instructions::payout::execute_payout(ctx)
    }

    pub fn approve_payout(ctx: Context<ApprovePayout>) -> Result<()> {
        instructions::payout::approve_payout(ctx)
    }

    pub fn register_oracle(
        ctx: Context<RegisterOracle>,
        oracle_id: String,
        oracle_type: OracleType,
        data_feed_address: String,
    ) -> Result<()> {
        instructions::oracle::register_oracle(ctx, oracle_id, oracle_type, data_feed_address)
    }

    pub fn unregister_oracle(ctx: Context<UnregisterOracle>) -> Result<()> {
        instructions::oracle::unregister_oracle(ctx)
    }

    pub fn update_oracle_data(
        ctx: Context<UpdateOracleData>,
        data: OracleData,
    ) -> Result<()> {
        instructions::oracle::update_oracle_data(ctx, data)
    }

    pub fn update_oracle_status(
        ctx: Context<UpdateOracleStatus>,
        is_active: bool,
    ) -> Result<()> {
        instructions::oracle::update_oracle_status(ctx, is_active)
    }

    pub fn emergency_oracle_override(
        ctx: Context<EmergencyOracleOverride>,
        corrected_data: OracleData,
        reason: String,
    ) -> Result<()> {
        instructions::oracle::emergency_oracle_override(ctx, corrected_data, reason)
    }

    pub fn reset_oracle_circuit_breaker(ctx: Context<ResetOracleCircuitBreaker>) -> Result<()> {
        instructions::oracle::reset_oracle_circuit_breaker(ctx)
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
