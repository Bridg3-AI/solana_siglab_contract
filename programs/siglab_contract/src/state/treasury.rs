use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};

#[account]
#[derive(Debug)]
pub struct Treasury {
    /// Treasury authority (admin)
    pub authority: Pubkey,
    
    /// Associated token account for USDC
    pub usdc_token_account: Pubkey,
    
    /// Associated token account for SOL (wrapped SOL)
    pub sol_token_account: Pubkey,
    
    /// USDC mint address
    pub usdc_mint: Pubkey,
    
    /// Total balance in USDC (tracked separately for accuracy)
    pub total_usdc_balance: u64,
    
    /// Total balance in SOL (in lamports)
    pub total_sol_balance: u64,
    
    /// Total premiums collected in USDC
    pub total_premiums_collected_usdc: u64,
    
    /// Total premiums collected in SOL
    pub total_premiums_collected_sol: u64,
    
    /// Total payouts disbursed in USDC
    pub total_payouts_disbursed_usdc: u64,
    
    /// Total payouts disbursed in SOL
    pub total_payouts_disbursed_sol: u64,
    
    /// Current reserve ratio percentage (basis points: 2000 = 20%)
    pub current_reserve_ratio: u16,
    
    /// Minimum reserve ratio required (basis points)
    pub minimum_reserve_ratio: u16,
    
    /// Total coverage exposure across all active policies
    pub total_coverage_exposure: u64,
    
    /// Number of deposit transactions
    pub deposit_count: u64,
    
    /// Number of withdrawal transactions
    pub withdrawal_count: u64,
    
    /// Last financial update timestamp
    pub last_update_timestamp: i64,
    
    /// Treasury creation timestamp
    pub created_at: i64,
    
    /// PDA bump seed
    pub bump: u8,
}

impl Treasury {
    /// Calculate space required for Treasury account
    pub fn space() -> usize {
        8 + // discriminator
        32 + // authority
        32 + // usdc_token_account
        32 + // sol_token_account
        32 + // usdc_mint
        8 + // total_usdc_balance
        8 + // total_sol_balance
        8 + // total_premiums_collected_usdc
        8 + // total_premiums_collected_sol
        8 + // total_payouts_disbursed_usdc
        8 + // total_payouts_disbursed_sol
        2 + // current_reserve_ratio
        2 + // minimum_reserve_ratio
        8 + // total_coverage_exposure
        8 + // deposit_count
        8 + // withdrawal_count
        8 + // last_update_timestamp
        8 + // created_at
        1   // bump
    }
    
    /// Calculate current reserve ratio in basis points
    pub fn calculate_reserve_ratio(&self) -> u16 {
        if self.total_coverage_exposure == 0 {
            return 10000; // 100% if no exposure
        }
        
        let total_balance = self.total_usdc_balance + self.total_sol_balance;
        if total_balance == 0 {
            return 0;
        }
        
        // Calculate ratio in basis points (10000 = 100%)
        let ratio = (total_balance * 10000) / self.total_coverage_exposure;
        std::cmp::min(ratio as u16, 10000)
    }
    
    /// Check if treasury meets minimum reserve requirements
    pub fn meets_reserve_requirement(&self) -> bool {
        self.calculate_reserve_ratio() >= self.minimum_reserve_ratio
    }
    
    /// Calculate available liquidity for new policies
    pub fn available_liquidity(&self) -> u64 {
        let total_balance = self.total_usdc_balance + self.total_sol_balance;
        let required_reserves = (self.total_coverage_exposure * self.minimum_reserve_ratio as u64) / 10000;
        
        if total_balance > required_reserves {
            total_balance - required_reserves
        } else {
            0
        }
    }
    
    /// Update balances after a transaction
    pub fn update_balances(&mut self, usdc_change: i64, sol_change: i64, timestamp: i64) {
        // Update USDC balance
        if usdc_change >= 0 {
            self.total_usdc_balance += usdc_change as u64;
        } else {
            self.total_usdc_balance = self.total_usdc_balance.saturating_sub((-usdc_change) as u64);
        }
        
        // Update SOL balance
        if sol_change >= 0 {
            self.total_sol_balance += sol_change as u64;
        } else {
            self.total_sol_balance = self.total_sol_balance.saturating_sub((-sol_change) as u64);
        }
        
        // Update reserve ratio
        self.current_reserve_ratio = self.calculate_reserve_ratio();
        self.last_update_timestamp = timestamp;
    }
    
    /// Record premium collection
    pub fn record_premium(&mut self, amount: u64, is_usdc: bool, timestamp: i64) {
        if is_usdc {
            self.total_premiums_collected_usdc += amount;
            self.total_usdc_balance += amount;
        } else {
            self.total_premiums_collected_sol += amount;
            self.total_sol_balance += amount;
        }
        
        self.current_reserve_ratio = self.calculate_reserve_ratio();
        self.last_update_timestamp = timestamp;
    }
    
    /// Record payout disbursement
    pub fn record_payout(&mut self, amount: u64, is_usdc: bool, timestamp: i64) -> Result<()> {
        if is_usdc {
            require!(self.total_usdc_balance >= amount, crate::error::InsuranceError::InsufficientTreasury);
            self.total_payouts_disbursed_usdc += amount;
            self.total_usdc_balance -= amount;
        } else {
            require!(self.total_sol_balance >= amount, crate::error::InsuranceError::InsufficientTreasury);
            self.total_payouts_disbursed_sol += amount;
            self.total_sol_balance -= amount;
        }
        
        self.current_reserve_ratio = self.calculate_reserve_ratio();
        self.last_update_timestamp = timestamp;
        Ok(())
    }
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct DepositInfo {
    /// Amount deposited
    pub amount: u64,
    /// Token type (USDC or SOL)
    pub token_type: TokenType,
    /// Depositor address
    pub depositor: Pubkey,
    /// Timestamp of deposit
    pub timestamp: i64,
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct WithdrawalInfo {
    /// Amount withdrawn
    pub amount: u64,
    /// Token type (USDC or SOL)
    pub token_type: TokenType,
    /// Recipient address
    pub recipient: Pubkey,
    /// Timestamp of withdrawal
    pub timestamp: i64,
    /// Reason for withdrawal
    pub reason: WithdrawalReason,
}

#[derive(Debug, Clone, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub enum TokenType {
    USDC,
    SOL,
}

#[derive(Debug, Clone, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub enum WithdrawalReason {
    AdminWithdrawal,
    PolicyPayout,
    PremiumRefund,
    EmergencyWithdrawal,
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct FinancialReport {
    /// Total treasury balance (USDC + SOL)
    pub total_balance: u64,
    /// Current reserve ratio in basis points
    pub reserve_ratio: u16,
    /// Total premiums collected
    pub total_premiums: u64,
    /// Total payouts disbursed
    pub total_payouts: u64,
    /// Net profit/loss
    pub net_result: i64,
    /// Total coverage exposure
    pub coverage_exposure: u64,
    /// Available liquidity for new policies
    pub available_liquidity: u64,
    /// Number of transactions
    pub transaction_count: u64,
    /// Report generation timestamp
    pub timestamp: i64,
}

impl FinancialReport {
    pub fn from_treasury(treasury: &Treasury) -> Self {
        let total_balance = treasury.total_usdc_balance + treasury.total_sol_balance;
        let total_premiums = treasury.total_premiums_collected_usdc + treasury.total_premiums_collected_sol;
        let total_payouts = treasury.total_payouts_disbursed_usdc + treasury.total_payouts_disbursed_sol;
        let net_result = total_premiums as i64 - total_payouts as i64;
        let transaction_count = treasury.deposit_count + treasury.withdrawal_count;
        
        Self {
            total_balance,
            reserve_ratio: treasury.current_reserve_ratio,
            total_premiums,
            total_payouts,
            net_result,
            coverage_exposure: treasury.total_coverage_exposure,
            available_liquidity: treasury.available_liquidity(),
            transaction_count,
            timestamp: treasury.last_update_timestamp,
        }
    }
}