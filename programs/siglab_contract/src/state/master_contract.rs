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

