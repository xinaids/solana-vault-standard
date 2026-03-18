use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use crate::{
    constants::ALLOCATOR_VAULT_SEED,
    error::VaultError,
    events::Allocate as AllocateEvent,
    state::{AllocatorVault, ChildAllocation},
};
use svs_math::{mul_div, Rounding};

pub fn handler(ctx: Context<Allocate>, amount: u64) -> Result<()> {
    require!(!ctx.accounts.vault.paused, VaultError::VaultPaused);
    require!(amount > 0, VaultError::ZeroAmount);
    require!(ctx.accounts.child_allocation.enabled, VaultError::ChildAllocationDisabled);
    require!(ctx.accounts.idle_vault.amount >= amount, VaultError::InsufficientLiquidity);

    // Check idle buffer after allocation
    let idle_after = ctx.accounts.idle_vault.amount
        .checked_sub(amount).ok_or(VaultError::MathOverflow)?;

    // Compute total assets (idle + child positions via remaining_accounts)
    let mut total: u128 = ctx.accounts.idle_vault.amount as u128;
    for child_info in ctx.remaining_accounts.iter() {
        let data = child_info.try_borrow_data()?;
        if data.len() < 8 + 32 + 32 + 32 + 32 + 8 { continue; }
        let offset = 8 + 32 + 32 + 32 + 32;
        let child_assets = u64::from_le_bytes(
            data[offset..offset+8].try_into().map_err(|_| VaultError::MathOverflow)?
        );
        total = total.checked_add(child_assets as u128).ok_or(VaultError::MathOverflow)?;
    }
    let total_assets = u64::try_from(total).map_err(|_| error!(VaultError::MathOverflow))?;

    // Check idle buffer constraint
    let min_idle = mul_div(
        total_assets,
        ctx.accounts.vault.idle_buffer_bps as u64,
        10_000,
        Rounding::Ceiling,
    ).map_err(|_| error!(VaultError::MathOverflow))?;
    require!(idle_after >= min_idle, VaultError::InsufficientBuffer);

    // Transfer from idle vault to child vault via signed transfer
    let asset_mint_key = ctx.accounts.vault.asset_mint;
    let vault_id_bytes = ctx.accounts.vault.vault_id.to_le_bytes();
    let bump = ctx.accounts.vault.bump;
    let signer_seeds: &[&[&[u8]]] = &[&[
        ALLOCATOR_VAULT_SEED,
        asset_mint_key.as_ref(),
        vault_id_bytes.as_ref(),
        &[bump],
    ]];

    let ix = anchor_spl::token_interface::spl_token_2022::instruction::transfer_checked(
        &ctx.accounts.token_program.key(),
        &ctx.accounts.idle_vault.key(),
        &ctx.accounts.asset_mint.key(),
        &ctx.accounts.child_asset_vault.key(),
        &ctx.accounts.vault.key(),
        &[],
        amount,
        ctx.accounts.asset_mint.decimals,
    )?;

    anchor_lang::solana_program::program::invoke_signed(
        &ix,
        &[
            ctx.accounts.idle_vault.to_account_info(),
            ctx.accounts.asset_mint.to_account_info(),
            ctx.accounts.child_asset_vault.to_account_info(),
            ctx.accounts.vault.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
        ],
        signer_seeds,
    )?;

    ctx.accounts.child_allocation.deposited_assets = ctx.accounts.child_allocation.deposited_assets
        .checked_add(amount).ok_or(VaultError::MathOverflow)?;

    emit!(AllocateEvent {
        vault: ctx.accounts.vault.key(),
        child_vault: ctx.accounts.child_allocation.child_vault,
        amount,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct Allocate<'info> {
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
    // remaining_accounts: [child_vault_state] for total_assets computation
}
