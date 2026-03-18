use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    mint_to_checked, transfer_checked, Mint, MintToChecked, TokenAccount, TokenInterface, TransferChecked,
};
use anchor_spl::associated_token::AssociatedToken;
use crate::{
    constants::{ALLOCATOR_VAULT_SEED, MIN_DEPOSIT, SHARES_SEED},
    error::VaultError,
    events::Deposit as DepositEvent,
    state::AllocatorVault,
};
use svs_math::{convert_to_shares, Rounding};

pub fn handler(ctx: Context<Deposit>, assets: u64, min_shares_out: u64) -> Result<()> {
    require!(!ctx.accounts.vault.paused, VaultError::VaultPaused);
    require!(assets >= MIN_DEPOSIT, VaultError::DepositTooSmall);

    // Compute total_assets from remaining_accounts
    // remaining_accounts: [child_vault_state] per child (read-only)
    let idle_balance = ctx.accounts.idle_vault.amount;
    let mut total: u128 = idle_balance as u128;

    for child_info in ctx.remaining_accounts.iter() {
        let data = child_info.try_borrow_data()?;
        if data.len() < 8 + 32 + 32 + 32 + 32 + 8 {
            continue;
        }
        // Read total_assets from child vault at offset 136 (8 disc + 32+32+32+32)
        let offset = 8 + 32 + 32 + 32 + 32;
        let child_assets = u64::from_le_bytes(
            data[offset..offset+8].try_into().map_err(|_| VaultError::MathOverflow)?
        );
        total = total.checked_add(child_assets as u128).ok_or(VaultError::MathOverflow)?;
    }

    let total_assets = u64::try_from(total).map_err(|_| error!(VaultError::MathOverflow))?;
    let total_shares = ctx.accounts.vault.total_shares;

    let shares = convert_to_shares(assets, total_assets, total_shares, ctx.accounts.vault.decimals_offset, Rounding::Floor)
        .map_err(|_| error!(VaultError::MathOverflow))?;
    require!(shares >= min_shares_out, VaultError::SlippageExceeded);
    require!(shares > 0, VaultError::ZeroAmount);

    // Transfer assets to idle vault
    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.user_asset_account.to_account_info(),
                mint: ctx.accounts.asset_mint.to_account_info(),
                to: ctx.accounts.idle_vault.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        assets,
        ctx.accounts.asset_mint.decimals,
    )?;

    // Mint allocator shares
    let asset_mint_key = ctx.accounts.vault.asset_mint;
    let vault_id_bytes = ctx.accounts.vault.vault_id.to_le_bytes();
    let bump = ctx.accounts.vault.bump;
    let signer_seeds: &[&[&[u8]]] = &[&[
        ALLOCATOR_VAULT_SEED,
        asset_mint_key.as_ref(),
        vault_id_bytes.as_ref(),
        &[bump],
    ]];

    mint_to_checked(
        CpiContext::new_with_signer(
            ctx.accounts.shares_token_program.to_account_info(),
            MintToChecked {
                mint: ctx.accounts.shares_mint.to_account_info(),
                to: ctx.accounts.user_shares_account.to_account_info(),
                authority: ctx.accounts.vault.to_account_info(),
            },
            signer_seeds,
        ),
        shares,
        9,
    )?;

    ctx.accounts.vault.total_shares = total_shares
        .checked_add(shares).ok_or(VaultError::MathOverflow)?;

    emit!(DepositEvent {
        vault: ctx.accounts.vault.key(),
        caller: ctx.accounts.user.key(),
        assets,
        shares,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct Deposit<'info> {
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

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = shares_mint,
        associated_token::authority = user,
        associated_token::token_program = shares_token_program,
    )]
    pub user_shares_account: Box<InterfaceAccount<'info, TokenAccount>>,

    pub token_program: Interface<'info, TokenInterface>,
    pub shares_token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    // remaining_accounts: [child_vault_state] per child (for total_assets computation)
}
