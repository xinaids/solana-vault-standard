use anchor_lang::prelude::*;
use anchor_spl::token_interface::TokenAccount;
use crate::{
    error::VaultError,
    events::ChildRemoved,
    state::{AllocatorVault, ChildAllocation},
};

pub fn handler(ctx: Context<RemoveChild>) -> Result<()> {
    require!(ctx.accounts.child_shares_account.amount == 0, VaultError::ChildHasShares);

    ctx.accounts.vault.num_children = ctx.accounts.vault.num_children.saturating_sub(1);

    emit!(ChildRemoved {
        vault: ctx.accounts.vault.key(),
        child_vault: ctx.accounts.child_allocation.child_vault,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct RemoveChild<'info> {
    #[account(mut, has_one = authority)]
    pub vault: Account<'info, AllocatorVault>,

    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        constraint = child_allocation.allocator_vault == vault.key() @ VaultError::InvalidChildVault,
        close = authority,
    )]
    pub child_allocation: Account<'info, ChildAllocation>,

    #[account(
        constraint = child_shares_account.key() == child_allocation.child_shares_account @ VaultError::InvalidChildVault,
    )]
    pub child_shares_account: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
}
