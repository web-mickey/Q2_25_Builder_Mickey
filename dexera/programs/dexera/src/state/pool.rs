use anchor_lang::prelude::*;

#[account]
pub struct Pool {
    pub creator: Pubkey,
    pub mint_x: Pubkey,
    pub mint_y: Pubkey,
    pub mint_lp: Pubkey,
    pub pool_bump: u8,
    pub mint_lp_bump: u8,
    pub locked: bool,
}

impl Pool {
    pub const INIT_SPACE: usize = 8 + // discriminator
        32 + // creator
        32 + // mint_x
        32 + // mint_y
        32 + // mint_lp
        1 + // pool_bump
        1 + // mint_lp_bump
        1; // locked
}
