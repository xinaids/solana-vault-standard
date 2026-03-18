use anchor_lang::prelude::*;

pub mod constants;
pub mod error;
pub mod events;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("9JuGhpA1n6MmTx8dCyM7JcN2QH2okjeFXwuGAzFS3rQC");

#[program]
pub mod svs_9 {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        vault_id: u64,
        idle_buffer_bps: u16,
        decimals_offset: u8,
    ) -> Result<()> {
        instructions::initialize::handler(ctx, vault_id, idle_buffer_bps, decimals_offset)
    }

    pub fn add_child(
        ctx: Context<AddChild>,
        target_weight_bps: u16,
        max_weight_bps: u16,
    ) -> Result<()> {
        instructions::add_child::handler(ctx, target_weight_bps, max_weight_bps)
    }

    pub fn remove_child(ctx: Context<RemoveChild>) -> Result<()> {
        instructions::remove_child::handler(ctx)
    }

    pub fn deposit(ctx: Context<Deposit>, assets: u64, min_shares_out: u64) -> Result<()> {
        instructions::deposit::handler(ctx, assets, min_shares_out)
    }

    pub fn redeem(ctx: Context<Redeem>, shares: u64, min_assets_out: u64) -> Result<()> {
        instructions::redeem::handler(ctx, shares, min_assets_out)
    }

    pub fn allocate(ctx: Context<Allocate>, amount: u64) -> Result<()> {
        instructions::allocate::handler(ctx, amount)
    }

    pub fn deallocate(ctx: Context<Deallocate>, amount: u64) -> Result<()> {
        instructions::deallocate::handler(ctx, amount)
    }

    pub fn update_weights(
        ctx: Context<UpdateWeights>,
        target_weight_bps: u16,
        max_weight_bps: u16,
    ) -> Result<()> {
        instructions::admin::update_weights(ctx, target_weight_bps, max_weight_bps)
    }

    pub fn set_curator(ctx: Context<Admin>, new_curator: Pubkey) -> Result<()> {
        instructions::admin::set_curator(ctx, new_curator)
    }

    pub fn pause(ctx: Context<Admin>) -> Result<()> {
        instructions::admin::pause(ctx)
    }

    pub fn unpause(ctx: Context<Admin>) -> Result<()> {
        instructions::admin::unpause(ctx)
    }

    pub fn transfer_authority(ctx: Context<Admin>, new_authority: Pubkey) -> Result<()> {
        instructions::admin::transfer_authority(ctx, new_authority)
    }
}
