use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::{
        spl_token_2022::{extension::ExtensionType, instruction::initialize_mint2},
        Token2022,
    },
    token_interface::{Mint, TokenAccount, TokenInterface},
};
use crate::{
    constants::{ALLOCATOR_VAULT_SEED, IDLE_VAULT_SEED, SHARES_SEED},
    error::VaultError,
    events::AllocatorInitialized,
    state::AllocatorVault,
};

pub fn handler(
    ctx: Context<Initialize>,
    vault_id: u64,
    idle_buffer_bps: u16,
    decimals_offset: u8,
) -> Result<()> {
    require!(idle_buffer_bps <= 10_000, VaultError::WeightSumExceeded);

    let vault_key = ctx.accounts.vault.key();
    let shares_mint_bump = ctx.bumps.shares_mint;

    // Create shares mint (Token-2022) following SVS-1 pattern
    let mint_size = ExtensionType::try_calculate_account_len::<anchor_spl::token_2022::spl_token_2022::state::Mint>(&[])
        .map_err(|_| VaultError::MathOverflow)?;
    let lamports = ctx.accounts.rent.minimum_balance(mint_size);

    let shares_mint_seeds: &[&[u8]] = &[SHARES_SEED, vault_key.as_ref(), &[shares_mint_bump]];

    invoke_signed(
        &anchor_lang::solana_program::system_instruction::create_account(
            &ctx.accounts.authority.key(),
            &ctx.accounts.shares_mint.key(),
            lamports,
            mint_size as u64,
            &ctx.accounts.token_2022_program.key(),
        ),
        &[
            ctx.accounts.authority.to_account_info(),
            ctx.accounts.shares_mint.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
        &[shares_mint_seeds],
    )?;

    let init_mint_ix = initialize_mint2(
        &ctx.accounts.token_2022_program.key(),
        &ctx.accounts.shares_mint.key(),
        &vault_key,
        None,
        9,
    )?;
    invoke_signed(
        &init_mint_ix,
        &[ctx.accounts.shares_mint.to_account_info()],
        &[shares_mint_seeds],
    )?;

    let vault = &mut ctx.accounts.vault;
    vault.authority = ctx.accounts.authority.key();
    vault.curator = ctx.accounts.authority.key();
    vault.asset_mint = ctx.accounts.asset_mint.key();
    vault.shares_mint = ctx.accounts.shares_mint.key();
    vault.idle_vault = ctx.accounts.idle_vault.key();
    vault.total_shares = 0;
    vault.num_children = 0;
    vault.idle_buffer_bps = idle_buffer_bps;
    vault.decimals_offset = decimals_offset;
    vault.bump = ctx.bumps.vault;
    vault.paused = false;
    vault.vault_id = vault_id;
    vault._reserved = [0u8; 64];

    emit!(AllocatorInitialized {
        vault: vault.key(),
        authority: ctx.accounts.authority.key(),
        asset_mint: ctx.accounts.asset_mint.key(),
        vault_id,
    });

    Ok(())
}

#[derive(Accounts)]
#[instruction(vault_id: u64)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    pub asset_mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = authority,
        space = AllocatorVault::LEN,
        seeds = [ALLOCATOR_VAULT_SEED, asset_mint.key().as_ref(), vault_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub vault: Account<'info, AllocatorVault>,

    /// CHECK: initialized via CPI in handler
    #[account(
        mut,
        seeds = [SHARES_SEED, vault.key().as_ref()],
        bump,
    )]
    pub shares_mint: UncheckedAccount<'info>,

    #[account(
        init,
        payer = authority,
        associated_token::mint = asset_mint,
        associated_token::authority = vault,
        associated_token::token_program = asset_token_program,
        // Note: idle_vault uses associated token but we need a PDA-derived address
        // We use associated token for simplicity
    )]
    pub idle_vault: InterfaceAccount<'info, TokenAccount>,

    pub asset_token_program: Interface<'info, TokenInterface>,
    pub token_2022_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}
