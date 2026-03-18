use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    burn_checked, transfer_checked, BurnChecked, Mint, TokenAccount, TokenInterface, TransferChecked,
};
use crate::{
    constants::ALLOCATOR_VAULT_SEED,
    error::VaultError,
    events::Redeem as RedeemEvent,
    state::AllocatorVault,
};
use svs_math::{convert_to_assets, Rounding};

pub fn handler(ctx: Context<Redeem>, shares: u64, min_assets_out: u64) -> Result<()> {
    require!(!ctx.accounts.vault.paused, VaultError::VaultPaused);
    require!(shares > 0, VaultError::ZeroAmount);
    require!(shares <= ctx.accounts.vault.total_shares, VaultError::InsufficientShares);

    // Compute total_assets
    let idle_balance = ctx.accounts.idle_vault.amount;
    let mut total: u128 = idle_balance as u128;

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

    let assets = convert_to_assets(shares, total_assets, ctx.accounts.vault.total_shares, ctx.accounts.vault.decimals_offset, Rounding::Floor)
        .map_err(|_| error!(VaultError::MathOverflow))?;

    require!(assets >= min_assets_out, VaultError::SlippageExceeded);
    require!(assets > 0, VaultError::ZeroAmount);
    require!(idle_balance >= assets, VaultError::InsufficientLiquidity);

    // Transfer assets from idle vault to user
    let asset_mint_key = ctx.accounts.vault.asset_mint;
    let vault_id_bytes = ctx.accounts.vault.vault_id.to_le_bytes();
    let bump = ctx.accounts.vault.bump;
    let signer_seeds: &[&[&[u8]]] = &[&[
        ALLOCATOR_VAULT_SEED,
        asset_mint_key.as_ref(),
        vault_id_bytes.as_ref(),
        &[bump],
    ]];

    transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.idle_vault.to_account_info(),
                mint: ctx.accounts.asset_mint.to_account_info(),
                to: ctx.accounts.user_asset_account.to_account_info(),
                authority: ctx.accounts.vault.to_account_info(),
            },
            signer_seeds,
        ),
        assets,
        ctx.accounts.asset_mint.decimals,
    )?;

    // Burn shares
    burn_checked(
        CpiContext::new(
            ctx.accounts.shares_token_program.to_account_info(),
            BurnChecked {
                mint: ctx.accounts.shares_mint.to_account_info(),
                from: ctx.accounts.user_shares_account.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        shares,
        9,
    )?;

    ctx.accounts.vault.total_shares = ctx.accounts.vault.total_shares
        .checked_sub(shares).ok_or(VaultError::MathOverflow)?;

    emit!(RedeemEvent {
        vault: ctx.accounts.vault.key(),
        caller: ctx.accounts.user.key(),
        shares,
        assets,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct Redeem<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub vault: Box<Account<'info, AllocatorVault>>,

    pub asset_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        constraint = idle_vault.key() == vault.idle_vault @ VaultError::InvalidChildVault,
    )]
    pub idle_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = shares_mint.key() == vault.shares_mint @ VaultError::InvalidChildVault,
    )]
    pub shares_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(mut)]
    pub user_asset_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut)]
    pub user_shares_account: Box<InterfaceAccount<'info, TokenAccount>>,

    pub token_program: Interface<'info, TokenInterface>,
    pub shares_token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    // remaining_accounts: [child_vault_state] per child
}
