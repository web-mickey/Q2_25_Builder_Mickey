use anchor_lang::prelude::*;

#[account]
pub struct Profile {
    pub profile_id: u64,
    pub creator: Pubkey,
    pub created_timestamp: i64,
    pub expiration_timestamp: i64,
    pub locked: bool,
    pub profile_bump: u8,
}

impl Profile {
    pub const INIT_SPACE: usize = 8 + // discriminator
        8 + // profile_id
        32 + // creator
        8 + // created_timestamp
        8 + // expiration_timestamp
        1 + // locked
        1; // profile_bump
}
