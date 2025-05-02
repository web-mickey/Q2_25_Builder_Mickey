#![allow(unexpected_cfgs)]
pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("GSfSHoQyvYNvifuotN1qLjin1rggxDgFcGSYW37X4a8A");

#[program]
pub mod dexera {
    use super::*;

    pub fn initialize_protocol(ctx: Context<InitializeProtocol>, fee: u16) -> Result<()> {
        ctx.accounts.initialize_protocol(fee, ctx.bumps)
    }

    pub fn update_protocol_fee_account(
        ctx: Context<UpdateProtocolConfig>,
        new_protocol_fee_account: Pubkey,
    ) -> Result<()> {
        ctx.accounts
            .update_protocol_fee_account(new_protocol_fee_account)
    }

    pub fn create_profile(ctx: Context<CreateProfile>, profile_id: u64) -> Result<()> {
        ctx.accounts.create_profile(profile_id, ctx.bumps)
    }

    pub fn create_pool(ctx: Context<CreatePool>) -> Result<()> {
        ctx.accounts.create_pool_state(ctx.bumps)
    }

    pub fn deposit_liquidity(
        ctx: Context<DepositLiquidity>,
        lp_tokens_amount: u64,
        max_x_tokens: u64,
        max_y_tokens: u64,
    ) -> Result<()> {
        ctx.accounts
            .deposit_liquidity(lp_tokens_amount, max_x_tokens, max_y_tokens)
    }

    pub fn swap_exact_in(
        ctx: Context<SwapTokens>,
        max_amount_in: u64,
        amount_out: u64,
        referrer: Option<Pubkey>,
    ) -> Result<()> {
        ctx.accounts
            .swap_exact_in(max_amount_in, amount_out, ctx.program_id, referrer)
    }

    pub fn swap_exact_out(
        ctx: Context<SwapTokens>,
        amount_in: u64,
        min_amount_out: u64,
        referrer: Option<Pubkey>,
    ) -> Result<()> {
        ctx.accounts
            .swap_exact_out(amount_in, min_amount_out, ctx.program_id, referrer)
    }

    pub fn withdraw_liquidity(
        ctx: Context<WithdrawLiquidity>,
        amount: u64,
        min_x: u64,
        min_y: u64,
    ) -> Result<()> {
        ctx.accounts.withdraw_liquidity(amount, min_x, min_y)
    }
}
