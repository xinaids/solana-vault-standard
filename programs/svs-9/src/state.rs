use anchor_lang::prelude::*;
use crate::constants::{ALLOCATOR_VAULT_SEED, CHILD_ALLOCATION_SEED};

#[account]
pub struct AllocatorVault {
    pub authority: Pubkey,
    pub curator: Pubkey,
    pub asset_mint: Pubkey,
    pub shares_mint: Pubkey,
    pub idle_vault: Pubkey,
    pub total_shares: u64,
    pub num_children: u8,
    pub idle_buffer_bps: u16,
    pub decimals_offset: u8,
    pub bump: u8,
    pub paused: bool,
    pub vault_id: u64,
    pub _reserved: [u8; 64],
}

impl AllocatorVault {
    pub const LEN: usize = 8 +
        32 + // authority
        32 + // curator
        32 + // asset_mint
        32 + // shares_mint
        32 + // idle_vault
        8 +  // total_shares
        1 +  // num_children
        2 +  // idle_buffer_bps
        1 +  // decimals_offset
        1 +  // bump
        1 +  // paused
        8 +  // vault_id
        64;  // _reserved

    pub const SEED_PREFIX: &'static [u8] = ALLOCATOR_VAULT_SEED;
}

#[account]
pub struct ChildAllocation {
    pub allocator_vault: Pubkey,
    pub child_vault: Pubkey,
    pub child_program: Pubkey,
    pub child_shares_account: Pubkey,
    pub target_weight_bps: u16,
    pub max_weight_bps: u16,
    pub deposited_assets: u64,
    pub index: u8,
    pub enabled: bool,
    pub bump: u8,
}

impl ChildAllocation {
    pub const LEN: usize = 8 +
        32 + // allocator_vault
        32 + // child_vault
        32 + // child_program
        32 + // child_shares_account
        2 +  // target_weight_bps
        2 +  // max_weight_bps
        8 +  // deposited_assets
        1 +  // index
        1 +  // enabled
        1;   // bump

    pub const SEED_PREFIX: &'static [u8] = CHILD_ALLOCATION_SEED;
}
