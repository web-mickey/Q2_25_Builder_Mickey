#[allow(unexpected_cfgs)]
pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("5mTeXxoS5W3VZB1MFp9x4revzgMFentyansfxNGVbvwH");

#[program]
pub mod escrow_anchor {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        escrow_id: u64,
        amount: u64,
        receive_amount_b: u64,
    ) -> Result<()> {
        ctx.accounts
            .initalize(escrow_id, receive_amount_b, &ctx.bumps)?;
        ctx.accounts.deposit(amount)
    }

    pub fn take(ctx: Context<Take>) -> Result<()> {
        ctx.accounts.send()?;
        ctx.accounts.withdraw()?;
        ctx.accounts.close()
    }

    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        ctx.accounts.withdraw()?;
        ctx.accounts.close()
    }
}
