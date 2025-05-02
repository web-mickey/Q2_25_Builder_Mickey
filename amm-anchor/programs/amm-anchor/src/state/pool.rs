use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Pool {
    pub id: u64,
    pub owner: Pubkey,
    pub mint_x: Pubkey,
    pub mint_y: Pubkey,
    pub mint_lp: Pubkey,
    pub fee: u16,
    pub locked: bool,
    pub pool_bump: u8,
    pub lp_bump: u8,
}
