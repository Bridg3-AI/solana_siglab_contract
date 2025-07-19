use anchor_lang::prelude::*;

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct OracleData {
    pub value: u64,
    pub timestamp: i64,
}