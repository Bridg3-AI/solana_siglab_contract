use anchor_lang::prelude::*;
use crate::state::OracleData;

#[derive(Accounts)]
pub struct UpdateOracleData<'info> {
    #[account(mut)]
    pub oracle: Signer<'info>,
}

pub fn update_oracle_data(_ctx: Context<UpdateOracleData>, _data: OracleData) -> Result<()> {
    Ok(())
}