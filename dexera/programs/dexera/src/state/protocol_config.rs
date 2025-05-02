use anchor_lang::prelude::*;

// Config state for whole DEX
#[account]
pub struct ProtocolConfig {
    pub admin: Pubkey,
    pub protocol_fee_account: Pubkey,
    pub config_bump: u8,
    pub fee: u16,
}

impl ProtocolConfig {
    pub const INIT_SPACE: usize = 8 + // discriminator
        32 + // admin
        32 + // protocol_fee_account
        1 + // config_bump
        2; // fee
}
