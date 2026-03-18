use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use anchor_spl::associated_token::AssociatedToken;
use crate::{
    constants::{CHILD_ALLOCATION_SEED, MAX_CHILDREN, SVS1_VAULT_DISCRIMINATOR},
    error::VaultError,
    events::ChildAdded,
    state::{AllocatorVault, ChildAllocation},
};

pub fn handler(
    ctx: Context<AddChild>,
    target_weight_bps: u16,
    max_weight_bps: u16,
) -> Result<()> {
    require!(!ctx.accounts.vault.paused, VaultError::VaultPaused);
    require!(ctx.accounts.vault.num_children < MAX_CHILDREN, VaultError::MaxChildrenReached);
    require!(target_weight_bps <= 10_000, VaultError::WeightSumExceeded);
    require!(max_weight_bps >= target_weight_bps, VaultError::WeightSumExceeded);

    // Validate child vault discriminator
    let child_data = ctx.accounts.child_vault.try_borrow_data()?;
    require!(child_data.len() >= 8, VaultError::InvalidAccountData);
    let disc: [u8; 8] = child_data[..8].try_into().map_err(|_| VaultError::InvalidAccountData)?;
    require!(disc == SVS1_VAULT_DISCRIMINATOR, VaultError::UnsupportedChildVariant);
    drop(child_data);

    let index = ctx.accounts.vault.num_children;
    let child = &mut ctx.accounts.child_allocation;
    child.allocator_vault = ctx.accounts.vault.key();
    child.child_vault = ctx.accounts.child_vault.key();
    child.child_program = ctx.accounts.child_program.key();
    child.child_shares_account = ctx.accounts.child_shares_account.key();
    child.target_weight_bps = target_weight_bps;
    child.max_weight_bps = max_weight_bps;
    child.deposited_assets = 0;
    child.index = index;
    child.enabled = true;
    child.bump = ctx.bumps.child_allocation;

    ctx.accounts.vault.num_children += 1;

    emit!(ChildAdded {
        vault: ctx.accounts.vault.key(),
        child_vault: ctx.accounts.child_vault.key(),
        target_weight_bps,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct AddChild<'info> {
    #[account(mut, has_one = authority)]
    pub vault: Account<'info, AllocatorVault>,

    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: validated via discriminator check in handler
    pub child_vault: UncheckedAccount<'info>,

    /// CHECK: the program owning the child vault
    pub child_program: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = child_shares_mint,
        associated_token::authority = vault,
        associated_token::token_program = token_program,
    )]
    pub child_shares_account: InterfaceAccount<'info, TokenAccount>,

    pub child_shares_mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = authority,
        space = ChildAllocation::LEN,
        seeds = [CHILD_ALLOCATION_SEED, vault.key().as_ref(), child_vault.key().as_ref()],
        bump,
    )]
    pub child_allocation: Account<'info, ChildAllocation>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
