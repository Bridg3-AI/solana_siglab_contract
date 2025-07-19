use anchor_lang::prelude::*;
use crate::state::OracleData;

#[derive(Accounts)]
pub struct TriggerPayout<'info> {
    #[account(mut)]
    pub beneficiary: Signer<'info>,
}

pub fn trigger_payout(_ctx: Context<TriggerPayout>, _oracle_data: OracleData) -> Result<()> {
    Ok(())
}