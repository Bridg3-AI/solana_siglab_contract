use anchor_lang::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub enum OracleType {
    Chainlink,
    Pyth,
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct OracleData {
    /// Oracle data value (price/event data)
    pub value: u64,
    /// Timestamp when data was created
    pub timestamp: i64,
    /// Confidence interval for the data
    pub confidence: u64,
    /// Digital signature for data verification
    pub signature: [u8; 64],
    /// Nonce to prevent replay attacks
    pub nonce: u64,
}

#[account]
#[derive(Debug)]
pub struct Oracle {
    /// Unique oracle identifier
    pub oracle_id: String,
    /// Authority pubkey that can update this oracle
    pub authority: Pubkey,
    /// Type of oracle (Chainlink/Pyth)
    pub oracle_type: OracleType,
    /// Whether this oracle is currently active
    pub is_active: bool,
    /// Timestamp of last data update
    pub last_update_timestamp: i64,
    /// Data feed address for external oracle sources
    pub data_feed_address: String,
    /// Latest oracle data
    pub latest_data: Option<OracleData>,
    /// Oracle reputation score (0-100)
    pub reputation_score: u8,
    /// Total number of updates provided
    pub update_count: u64,
    /// Health metrics for this oracle
    pub health_metrics: OracleHealthMetrics,
    /// Bump seed for PDA
    pub bump: u8,
}

impl Oracle {
    pub const MAX_ORACLE_ID_LENGTH: usize = 32;
    pub const MAX_DATA_FEED_ADDRESS_LENGTH: usize = 64;
    
    /// Calculate space required for Oracle account
    pub fn space() -> usize {
        8 + // discriminator
        4 + Self::MAX_ORACLE_ID_LENGTH + // oracle_id (String)
        32 + // authority
        1 + // oracle_type
        1 + // is_active
        8 + // last_update_timestamp
        4 + Self::MAX_DATA_FEED_ADDRESS_LENGTH + // data_feed_address (String)
        1 + 8 + 8 + 8 + 64 + 8 + // latest_data (Option<OracleData>)
        1 + // reputation_score
        8 + // update_count
        4 + 1 + 8 + 4 + 1 + // health_metrics (OracleHealthMetrics)
        1   // bump
    }
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct ConsensusData {
    /// Aggregated value from multiple oracles
    pub aggregated_value: u64,
    /// Confidence score based on oracle agreement (0-100)
    pub confidence_score: u8,
    /// Number of oracles that contributed to consensus
    pub oracle_count: u8,
    /// Timestamp when consensus was reached
    pub consensus_timestamp: i64,
    /// Median value from all oracle inputs
    pub median_value: u64,
    /// Standard deviation of oracle values
    pub standard_deviation: u64,
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct OracleHealthMetrics {
    /// Number of successful updates in the last 24 hours
    pub updates_24h: u32,
    /// Average accuracy score (0-100)
    pub accuracy_score: u8,
    /// Last health check timestamp
    pub last_health_check: i64,
    /// Number of failed validations
    pub failed_validations: u32,
    /// Circuit breaker status
    pub circuit_breaker_active: bool,
}

impl OracleHealthMetrics {
    pub fn new() -> Self {
        Self {
            updates_24h: 0,
            accuracy_score: 100,
            last_health_check: 0,
            failed_validations: 0,
            circuit_breaker_active: false,
        }
    }
    
    /// Update metrics after a successful oracle update
    pub fn record_successful_update(&mut self, current_timestamp: i64) {
        self.updates_24h += 1;
        self.last_health_check = current_timestamp;
        
        // Improve accuracy score for successful updates (max 100)
        if self.accuracy_score < 100 {
            self.accuracy_score = std::cmp::min(100, self.accuracy_score + 1);
        }
    }
    
    /// Record a failed validation
    pub fn record_failed_validation(&mut self, current_timestamp: i64) {
        self.failed_validations += 1;
        self.last_health_check = current_timestamp;
        
        // Decrease accuracy score for failures
        if self.accuracy_score > 0 {
            self.accuracy_score = self.accuracy_score.saturating_sub(5);
        }
        
        // Activate circuit breaker if too many failures
        if self.failed_validations >= 5 {
            self.circuit_breaker_active = true;
        }
    }
    
    /// Reset daily metrics (should be called every 24 hours)
    pub fn reset_daily_metrics(&mut self, current_timestamp: i64) {
        self.updates_24h = 0;
        self.last_health_check = current_timestamp;
        
        // Reset failed validations if oracle is performing well
        if self.accuracy_score > 80 {
            self.failed_validations = 0;
            self.circuit_breaker_active = false;
        }
    }
}

impl ConsensusData {
    /// Create consensus data from multiple oracle values
    pub fn from_oracle_values(values: &[u64], timestamp: i64) -> Self {
        let oracle_count = values.len() as u8;
        let aggregated_value = Self::calculate_weighted_average(values);
        let median_value = Self::calculate_median(values);
        let standard_deviation = Self::calculate_standard_deviation(values, aggregated_value);
        let confidence_score = Self::calculate_confidence_score(values, standard_deviation);
        
        Self {
            aggregated_value,
            confidence_score,
            oracle_count,
            consensus_timestamp: timestamp,
            median_value,
            standard_deviation,
        }
    }
    
    /// Calculate weighted average (for now, simple mean)
    fn calculate_weighted_average(values: &[u64]) -> u64 {
        if values.is_empty() {
            return 0;
        }
        let sum: u64 = values.iter().sum();
        sum / values.len() as u64
    }
    
    /// Calculate median value
    fn calculate_median(values: &[u64]) -> u64 {
        if values.is_empty() {
            return 0;
        }
        
        let mut sorted_values = values.to_vec();
        sorted_values.sort();
        
        let len = sorted_values.len();
        if len % 2 == 0 {
            (sorted_values[len / 2 - 1] + sorted_values[len / 2]) / 2
        } else {
            sorted_values[len / 2]
        }
    }
    
    /// Calculate standard deviation
    fn calculate_standard_deviation(values: &[u64], mean: u64) -> u64 {
        if values.len() <= 1 {
            return 0;
        }
        
        let variance: u64 = values
            .iter()
            .map(|&value| {
                let diff = if value > mean { value - mean } else { mean - value };
                diff * diff
            })
            .sum::<u64>() / values.len() as u64;
        
        // Simple integer square root approximation
        Self::integer_sqrt(variance)
    }
    
    /// Calculate confidence score based on agreement level
    fn calculate_confidence_score(values: &[u64], std_dev: u64) -> u8 {
        if values.is_empty() {
            return 0;
        }
        
        let mean = values.iter().sum::<u64>() / values.len() as u64;
        if mean == 0 {
            return 0;
        }
        
        // Confidence decreases as standard deviation increases relative to mean
        let coefficient_of_variation = (std_dev * 100) / mean;
        
        // Confidence score: higher CV means lower confidence
        if coefficient_of_variation > 100 {
            0
        } else {
            (100 - coefficient_of_variation) as u8
        }
    }
    
    /// Simple integer square root using binary search
    pub fn integer_sqrt(n: u64) -> u64 {
        if n == 0 {
            return 0;
        }
        
        let mut left = 1u64;
        let mut right = n;
        let mut result = 0u64;
        
        while left <= right {
            let mid = left + (right - left) / 2;
            
            if mid <= n / mid {
                result = mid;
                left = mid + 1;
            } else {
                right = mid - 1;
            }
        }
        
        result
    }
}