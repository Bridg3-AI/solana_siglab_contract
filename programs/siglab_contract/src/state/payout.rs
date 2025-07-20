use anchor_lang::prelude::*;

#[derive(Debug, Clone, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub enum PayoutStatus {
    Pending,
    PendingApproval,
    Ready,
    Executed,
    Rejected,
    Expired,
}

#[account]
#[derive(Debug)]
pub struct PendingPayout {
    /// Policy ID this payout is for
    pub policy_id: String,
    
    /// Payout amount in lamports
    pub amount: u64,
    
    /// Timestamp when payout was triggered
    pub timestamp: i64,
    
    /// Priority level for processing order
    pub priority: u8,
    
    /// Current payout status
    pub status: PayoutStatus,
    
    /// Policy holder's account to receive payout
    pub beneficiary: Pubkey,
    
    /// Oracle data that triggered the payout
    pub trigger_oracle_data: Vec<u8>,
    
    /// Calculated severity score (0-100)
    pub severity_score: u8,
    
    /// Admin approval timestamp (if required)
    pub approval_timestamp: Option<i64>,
    
    /// Approving admin's pubkey (if required)
    pub approved_by: Option<Pubkey>,
    
    /// Expiration timestamp for pending approvals
    pub expires_at: i64,
    
    /// Reason for rejection (if applicable)
    pub rejection_reason: Option<String>,
    
    /// Bump seed for PDA
    pub bump: u8,
}

impl PendingPayout {
    pub const MAX_POLICY_ID_LENGTH: usize = 32;
    pub const MAX_ORACLE_DATA_LENGTH: usize = 256;
    pub const MAX_REJECTION_REASON_LENGTH: usize = 128;
    
    /// Calculate space required for PendingPayout account
    pub fn space() -> usize {
        8 + // discriminator
        4 + Self::MAX_POLICY_ID_LENGTH + // policy_id (String)
        8 + // amount
        8 + // timestamp
        1 + // priority
        std::mem::size_of::<PayoutStatus>() + // status
        32 + // beneficiary
        4 + Self::MAX_ORACLE_DATA_LENGTH + // trigger_oracle_data (Vec<u8>)
        1 + // severity_score
        1 + 8 + // approval_timestamp (Option<i64>)
        1 + 32 + // approved_by (Option<Pubkey>)
        8 + // expires_at
        1 + 4 + Self::MAX_REJECTION_REASON_LENGTH + // rejection_reason (Option<String>)
        1   // bump
    }
    
    /// Check if payout has expired
    pub fn is_expired(&self, current_timestamp: i64) -> bool {
        current_timestamp > self.expires_at
    }
    
    /// Check if payout requires admin approval
    pub fn requires_approval(&self) -> bool {
        matches!(self.status, PayoutStatus::PendingApproval)
    }
    
    /// Check if payout is ready for execution
    pub fn is_ready_for_execution(&self) -> bool {
        matches!(self.status, PayoutStatus::Ready)
    }
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct PayoutCalculationData {
    /// Base coverage amount
    pub coverage_amount: u64,
    
    /// Deductible to subtract
    pub deductible: u64,
    
    /// Severity percentage (0-100)
    pub severity_percentage: u8,
    
    /// Maximum payout limit
    pub max_payout: u64,
    
    /// Insurance type for specific calculations
    pub insurance_type: String,
}

impl PayoutCalculationData {
    /// Calculate final payout amount
    pub fn calculate_payout(&self) -> u64 {
        // Start with coverage amount
        let mut payout = self.coverage_amount;
        
        // Apply severity percentage
        payout = (payout * self.severity_percentage as u64) / 100;
        
        // Subtract deductible
        if payout > self.deductible {
            payout -= self.deductible;
        } else {
            return 0; // Payout below deductible threshold
        }
        
        // Apply maximum payout limit
        if payout > self.max_payout {
            payout = self.max_payout;
        }
        
        payout
    }
}