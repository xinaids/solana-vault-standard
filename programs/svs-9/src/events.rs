use anchor_lang::prelude::*;

#[event]
pub struct AllocatorInitialized {
    pub vault: Pubkey,
    pub authority: Pubkey,
    pub asset_mint: Pubkey,
    pub vault_id: u64,
}

#[event]
pub struct ChildAdded {
    pub vault: Pubkey,
    pub child_vault: Pubkey,
    pub target_weight_bps: u16,
}

#[event]
pub struct ChildRemoved {
    pub vault: Pubkey,
    pub child_vault: Pubkey,
}

#[event]
pub struct Deposit {
    pub vault: Pubkey,
    pub caller: Pubkey,
    pub assets: u64,
    pub shares: u64,
}

#[event]
pub struct Redeem {
    pub vault: Pubkey,
    pub caller: Pubkey,
    pub shares: u64,
    pub assets: u64,
}

#[event]
pub struct Allocate {
    pub vault: Pubkey,
    pub child_vault: Pubkey,
    pub amount: u64,
}

#[event]
pub struct Deallocate {
    pub vault: Pubkey,
    pub child_vault: Pubkey,
    pub shares: u64,
}

#[event]
pub struct WeightsUpdated {
    pub vault: Pubkey,
}

#[event]
pub struct CuratorSet {
    pub vault: Pubkey,
    pub new_curator: Pubkey,
}

#[event]
pub struct VaultStatusChanged {
    pub vault: Pubkey,
    pub paused: bool,
}

#[event]
pub struct AuthorityTransferred {
    pub vault: Pubkey,
    pub new_authority: Pubkey,
}
