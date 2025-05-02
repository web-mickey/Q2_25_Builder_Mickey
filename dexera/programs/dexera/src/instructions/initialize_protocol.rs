#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;

use crate::ProtocolConfig;

// Initializing DEX
#[derive(Accounts)]
pub struct InitializeProtocol<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        seeds = [b"config"],
        bump,
        space = ProtocolConfig::INIT_SPACE
    )]
    pub config: Account<'info, ProtocolConfig>,

    /// CHECK: Needs to be initialized outside the protocol
    #[account(mut)]
    pub protocol_fee_account: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl InitializeProtocol<'_> {
    pub fn initialize_protocol(&mut self, fee: u16, bumps: InitializeProtocolBumps) -> Result<()> {
        self.config.set_inner(ProtocolConfig {
            admin: self.admin.key(),
            protocol_fee_account: self.protocol_fee_account.key(),
            config_bump: bumps.config,
            fee,
        });
        Ok(())
    }
}
