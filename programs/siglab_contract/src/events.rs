use anchor_lang::prelude::*;

#[event]
pub struct MasterContractInitialized {
    pub admin: Pubkey,
    pub treasury_mint: Pubkey,
    pub reserve_ratio: u64,
    pub timestamp: i64,
}

#[event]
pub struct PolicyCreated {
    pub policy_id: u64,
    pub owner: Pubkey,
    pub insurance_type: u8,
    pub coverage_amount: u64,
    pub premium_amount: u64,
    pub expiry_timestamp: i64,
}

#[event]
pub struct PremiumPaid {
    pub policy_id: u64,
    pub payer: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct PayoutTriggered {
    pub policy_id: u64,
    pub beneficiary: Pubkey,
    pub amount: u64,
    pub oracle_value: u64,
    pub timestamp: i64,
}

#[event]
pub struct OracleDataUpdated {
    pub oracle: Pubkey,
    pub data_type: String,
    pub value: u64,
    pub timestamp: i64,
}

#[event]
pub struct ContractPaused {
    pub admin: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct ContractResumed {
    pub admin: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct TreasuryWithdrawn {
    pub admin: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}