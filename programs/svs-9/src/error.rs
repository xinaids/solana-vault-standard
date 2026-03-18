use anchor_lang::prelude::*;

#[error_code]
pub enum VaultError {
    #[msg("Vault is paused")]
    VaultPaused,
    #[msg("Amount must be greater than zero")]
    ZeroAmount,
    #[msg("Deposit too small")]
    DepositTooSmall,
    #[msg("Insufficient shares")]
    InsufficientShares,
    #[msg("Slippage exceeded")]
    SlippageExceeded,
    #[msg("Arithmetic overflow")]
    MathOverflow,
    #[msg("Division by zero")]
    DivisionByZero,
    #[msg("Max children reached")]
    MaxChildrenReached,
    #[msg("Child already registered")]
    ChildAlreadyRegistered,
    #[msg("Child not found")]
    ChildNotFound,
    #[msg("Child has active shares")]
    ChildHasShares,
    #[msg("Child allocation disabled")]
    ChildAllocationDisabled,
    #[msg("Invalid child program")]
    InvalidChildProgram,
    #[msg("Invalid child vault")]
    InvalidChildVault,
    #[msg("Unsupported child variant")]
    UnsupportedChildVariant,
    #[msg("Insufficient idle buffer")]
    InsufficientBuffer,
    #[msg("Insufficient liquidity for redeem")]
    InsufficientLiquidity,
    #[msg("Weight sum exceeds 10000 bps")]
    WeightSumExceeded,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Invalid account data")]
    InvalidAccountData,
    #[msg("Child allocation not empty")]
    ChildNotEmpty,
}
