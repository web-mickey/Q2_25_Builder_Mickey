use anchor_lang::prelude::*;

use crate::ProtocolConfig;

// Updating DEX config
#[derive(Accounts)]
pub struct UpdateProtocolConfig<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        has_one = admin,
        seeds = [b"config"],
        bump = config.config_bump,
    )]
    pub config: Account<'info, ProtocolConfig>,

    /// CHECK: Needs to be initialized outside the protocol
    #[account(mut)]
    pub protocol_fee_account: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl UpdateProtocolConfig<'_> {
    pub fn update_protocol_fee_account(&mut self, new_protocol_fee_account: Pubkey) -> Result<()> {
        self.config.protocol_fee_account = new_protocol_fee_account;

        Ok(())
    }
}
