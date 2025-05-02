use anchor_lang::prelude::*;

use crate::Profile;

#[derive(Accounts)]
#[instruction(profile_id:u64)]
pub struct CreateProfile<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        init,
        payer = creator,
        seeds = [b"profile", creator.key().as_ref()],
        bump,
        space = Profile::INIT_SPACE
    )]
    pub profile: Account<'info, Profile>,

    pub system_program: Program<'info, System>,
}

impl CreateProfile<'_> {
    pub fn create_profile(&mut self, profile_id: u64, bumps: CreateProfileBumps) -> Result<()> {
        let clock = Clock::get()?;
        let now = clock.unix_timestamp;
        let one_month_in_seconds = 30 * 24 * 60 * 60;

        self.profile.set_inner(Profile {
            profile_id,
            creator: self.creator.key(),
            created_timestamp: now,
            expiration_timestamp: now + one_month_in_seconds,
            locked: false,
            profile_bump: bumps.profile,
        });

        Ok(())
    }
}
