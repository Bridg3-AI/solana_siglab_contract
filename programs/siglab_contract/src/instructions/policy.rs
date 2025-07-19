use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreatePolicyParams {
    pub insurance_type: u8,
    pub coverage_amount: u64,
    pub premium_amount: u64,
    pub expiry_timestamp: i64,
}

#[derive(Accounts)]
pub struct CreatePolicy<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PayPremium<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
}

pub fn create_policy(
    _ctx: Context<CreatePolicy>,
    _params: CreatePolicyParams,
) -> Result<()> {
    Ok(())
}

pub fn pay_premium(_ctx: Context<PayPremium>, _amount: u64) -> Result<()> {
    Ok(())
}