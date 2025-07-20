use anchor_lang::prelude::*;
use super::policy::Policy;

#[account]
#[derive(Debug)]
pub struct MasterInsuranceContract {
    /// Authority that can manage the contract
    pub authority: Pubkey,
    
    /// List of policies (stored as Vec for Anchor compatibility)
    pub policies: Vec<Policy>,
    
    /// Treasury account for collecting premiums and paying claims
    pub treasury_account: Pubkey,
    
    /// Total premiums collected across all policies
    pub total_premiums_collected: u64,
    
    /// Total payouts disbursed to policyholders
    pub total_payouts_disbursed: u64,
    
    /// Number of currently active policies
    pub active_policies_count: u64,
    
    /// Reserve ratio as percentage (e.g., 20 = 20%)
    pub reserve_ratio: u64,
    
    /// Contract pause state
    pub is_paused: bool,
    
    /// Contract creation timestamp
    pub created_at: i64,
    
    /// Last update timestamp
    pub updated_at: i64,
    
    /// Registry of active oracle pubkeys
    pub oracle_registry: Vec<Pubkey>,
    
    /// Maximum number of oracles allowed
    pub max_oracles: u8,
    
    /// Minimum oracle consensus threshold
    pub min_consensus_threshold: u8,
    
    /// Bump seed for PDA
    pub bump: u8,
}

impl MasterInsuranceContract {
    pub fn space() -> usize {
        8 + // discriminator
        32 + // authority
        4 + (32 * 50) + // policies (assuming max 50 policies)
        32 + // treasury_account
        8 + // total_premiums_collected
        8 + // total_payouts_disbursed
        8 + // active_policies_count
        8 + // reserve_ratio
        1 + // is_paused
        8 + // created_at
        8 + // updated_at
        4 + (32 * 10) + // oracle_registry (max 10 oracles)
        1 + // max_oracles
        1 + // min_consensus_threshold
        1 // bump
    }
}

