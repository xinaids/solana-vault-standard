use anchor_lang::prelude::*;
use crate::{
    error::VaultError,
    events::{AuthorityTransferred, CuratorSet, VaultStatusChanged, WeightsUpdated},
    state::{AllocatorVault, ChildAllocation},
};

pub fn pause(ctx: Context<Admin>) -> Result<()> {
    ctx.accounts.vault.paused = true;
    emit!(VaultStatusChanged { vault: ctx.accounts.vault.key(), paused: true });
    Ok(())
}

pub fn unpause(ctx: Context<Admin>) -> Result<()> {
    ctx.accounts.vault.paused = false;
    emit!(VaultStatusChanged { vault: ctx.accounts.vault.key(), paused: false });
    Ok(())
}

pub fn transfer_authority(ctx: Context<Admin>, new_authority: Pubkey) -> Result<()> {
    ctx.accounts.vault.authority = new_authority;
    emit!(AuthorityTransferred { vault: ctx.accounts.vault.key(), new_authority });
    Ok(())
}

pub fn set_curator(ctx: Context<Admin>, new_curator: Pubkey) -> Result<()> {
    ctx.accounts.vault.curator = new_curator;
    emit!(CuratorSet { vault: ctx.accounts.vault.key(), new_curator });
    Ok(())
}

#[derive(Accounts)]
pub struct Admin<'info> {
    #[account(mut, has_one = authority)]
    pub vault: Account<'info, AllocatorVault>,
    pub authority: Signer<'info>,
}

pub fn update_weights(ctx: Context<UpdateWeights>, target_weight_bps: u16, max_weight_bps: u16) -> Result<()> {
    require!(target_weight_bps <= 10_000, VaultError::WeightSumExceeded);
    require!(max_weight_bps >= target_weight_bps, VaultError::WeightSumExceeded);
    ctx.accounts.child_allocation.target_weight_bps = target_weight_bps;
    ctx.accounts.child_allocation.max_weight_bps = max_weight_bps;
    emit!(WeightsUpdated { vault: ctx.accounts.vault.key() });
    Ok(())
}

#[derive(Accounts)]
pub struct UpdateWeights<'info> {
    #[account(has_one = authority)]
    pub vault: Account<'info, AllocatorVault>,
    pub authority: Signer<'info>,
    #[account(
        mut,
        constraint = child_allocation.allocator_vault == vault.key() @ VaultError::InvalidChildVault,
        constraint = child_allocation.allocator_vault == vault.key() @ VaultError::InvalidChildVault,
    )]
    pub child_allocation: Account<'info, ChildAllocation>,
}

// For curator-only operations
#[derive(Accounts)]
pub struct CuratorOp<'info> {
    #[account(mut, has_one = curator @ VaultError::Unauthorized)]
    pub vault: Account<'info, AllocatorVault>,
    pub curator: Signer<'info>,
}
