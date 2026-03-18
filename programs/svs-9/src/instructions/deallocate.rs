use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use crate::{
    constants::ALLOCATOR_VAULT_SEED,
    error::VaultError,
    events::Deallocate as DeallocateEvent,
    state::{AllocatorVault, ChildAllocation},
};

pub fn handler(ctx: Context<Deallocate>, amount: u64) -> Result<()> {
    require!(!ctx.accounts.vault.paused, VaultError::VaultPaused);
    require!(amount > 0, VaultError::ZeroAmount);

    let asset_mint_key = ctx.accounts.vault.asset_mint;
    let vault_id_bytes = ctx.accounts.vault.vault_id.to_le_bytes();
    let bump = ctx.accounts.vault.bump;
    let signer_seeds: &[&[&[u8]]] = &[&[
        ALLOCATOR_VAULT_SEED,
        asset_mint_key.as_ref(),
        vault_id_bytes.as_ref(),
        &[bump],
    ]];

    // Transfer from child vault back to idle vault
    let ix = anchor_spl::token_interface::spl_token_2022::instruction::transfer_checked(
        &ctx.accounts.token_program.key(),
        &ctx.accounts.child_asset_vault.key(),
        &ctx.accounts.asset_mint.key(),
        &ctx.accounts.idle_vault.key(),
        &ctx.accounts.vault.key(),
        &[],
        amount,
        ctx.accounts.asset_mint.decimals,
    )?;

    anchor_lang::solana_program::program::invoke_signed(
        &ix,
        &[
            ctx.accounts.child_asset_vault.to_account_info(),
            ctx.accounts.asset_mint.to_account_info(),
            ctx.accounts.idle_vault.to_account_info(),
            ctx.accounts.vault.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
        ],
        signer_seeds,
    )?;

    // Update deposited_assets (subtract returned amount, floor at 0)
    ctx.accounts.child_allocation.deposited_assets =
        ctx.accounts.child_allocation.deposited_assets.saturating_sub(amount);

    emit!(DeallocateEvent {
        vault: ctx.accounts.vault.key(),
        child_vault: ctx.accounts.child_allocation.child_vault,
        shares: amount,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct Deallocate<'info> {
    #[account(mut, has_one = curator @ VaultError::Unauthorized)]
    pub vault: Box<Account<'info, AllocatorVault>>,

    pub curator: Signer<'info>,

    #[account(
        mut,
        constraint = child_allocation.allocator_vault == vault.key() @ VaultError::InvalidChildVault,
        constraint = child_allocation.allocator_vault == vault.key() @ VaultError::InvalidChildVault,
    )]
    pub child_allocation: Box<Account<'info, ChildAllocation>>,

    pub asset_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        constraint = idle_vault.key() == vault.idle_vault @ VaultError::InvalidChildVault,
    )]
    pub idle_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK: child vault's asset token account
    #[account(mut)]
    pub child_asset_vault: UncheckedAccount<'info>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}
