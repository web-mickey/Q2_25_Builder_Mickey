#![allow(unexpected_cfgs)]
pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("76EPMJ575d4TwV2FiK5miCMco9yGViDDWTtiJw3SZQ9a");

#[program]
pub mod amm_anchor {
    use super::*;

    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        pool_id: u64,
        fee: u16,
        amount: u64,
        max_x: u64,
        max_y: u64,
    ) -> Result<()> {
        ctx.accounts.initialize(pool_id, fee, ctx.bumps)?;
        ctx.accounts.deposit(amount, max_x, max_y)
    }

    pub fn withdraw_from_pool(
        ctx: Context<Withdraw>,
        pool_id: u64,
        amount: u64,
        min_x: u64,
        min_y: u64,
    ) -> Result<()> {
        ctx.accounts
            .withdraw_liquidity_from_pool(amount, min_x, min_y)
    }

    pub fn swap(ctx: Context<Swap>, pool_id: u64, is_x: bool, amount: u64, min: u64) -> Result<()> {
        ctx.accounts.swap_tokens(is_x, amount, min)
    }
}
