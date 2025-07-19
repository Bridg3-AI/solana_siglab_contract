use anchor_lang::prelude::*;

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct Policy {
    /// Unique policy identifier
    pub id: String,
    
    /// Policyholder's public key
    pub user: Pubkey,
    
    /// Type of insurance (Weather, Earthquake, etc.)
    pub insurance_type: InsuranceType,
    
    /// Coverage amount in lamports
    pub coverage_amount: u64,
    
    /// Premium amount in lamports
    pub premium_amount: u64,
    
    /// Deductible amount in lamports
    pub deductible: u64,
    
    /// Policy start date (Unix timestamp)
    pub start_date: i64,
    
    /// Policy end date (Unix timestamp)
    pub end_date: i64,
    
    /// Current policy status
    pub status: PolicyStatus,
    
    /// Conditions that trigger payouts
    pub trigger_conditions: TriggerConditions,
    
    /// Oracle configuration for data feeds
    pub oracle_config: OracleConfig,
    
    /// Last premium payment timestamp
    pub last_premium_paid: i64,
    
    /// History of payouts made
    pub payout_history: Vec<PayoutRecord>,
    
    /// Policy creation timestamp
    pub created_at: i64,
    
    /// Last update timestamp
    pub updated_at: i64,
}

// Forward declarations - will be implemented in following subtasks
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub enum InsuranceType {
    Weather,
    Earthquake,
    Flight,
    Crop,
    Custom,
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub enum PolicyStatus {
    Active,
    Expired,
    Cancelled,
    PendingPayout,
    PaidOut,
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct TriggerConditions {
    pub threshold_value: f64,
    pub comparison_operator: ComparisonOperator,
    pub data_source: String,
    pub grace_period: i64,
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct OracleConfig {
    pub oracle_address: Pubkey,
    pub data_feed_id: String,
    pub required_confirmations: u8,
    pub staleness_threshold: i64,
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub enum ComparisonOperator {
    GreaterThan,
    LessThan,
    Equals,
    NotEquals,
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct PayoutRecord {
    pub amount: u64,
    pub timestamp: i64,
    pub transaction_id: String,
    pub oracle_data: String,
}