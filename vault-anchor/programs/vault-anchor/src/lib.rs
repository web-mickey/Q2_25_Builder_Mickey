#![allow(unexpected_cfgs)]
use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

declare_id!("DZA294uXy8zdcW8FZEXYCHMJ6Z1NriEuT7D6V5S1vjQH");

#[program]
pub mod vault_anchor {
    use super::*;

    pub fn create_vault(ctx: Context<CreateVault>) -> Result<()> {
        ctx.accounts.create_vault(&ctx.bumps)
    }

    pub fn deposit(ctx: Context<Payment>, amount: u64) -> Result<()> {
        ctx.accounts.deposit(amount)
    }

    pub fn withdraw(ctx: Context<Payment>, amount: u64) -> Result<()> {
        ctx.accounts.withdraw(amount)
    }

    pub fn close(ctx: Context<Close>) -> Result<()> {
        ctx.accounts.close()
    }
}

#[derive(Accounts)]
pub struct CreateVault<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init,
        payer=user,
        seeds = [b"state", user.key().as_ref()],
        bump,
        space = VaultState::INIT_SPACE
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        seeds = [b"vault", vault_state.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl CreateVault<'_> {
    pub fn create_vault(&mut self, bumps: &CreateVaultBumps) -> Result<()> {
        self.vault_state.state_bump = bumps.vault_state;
        self.vault_state.vault_bump = bumps.vault;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Payment<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        seeds = [b"state", user.key().as_ref()],
        bump,
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl Payment<'_> {
    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        let cpi_program = self.system_program.to_account_info();

        let from = self.user.to_account_info();
        let to = self.vault.to_account_info();

        let accounts = Transfer {
            from: from.to_account_info(),
            to: to.to_account_info(),
        };

        let cpi_context = CpiContext::new(cpi_program, accounts);

        transfer(cpi_context, amount)
    }

    pub fn withdraw(&mut self, amount: u64) -> Result<()> {
        let cpi_program = self.system_program.to_account_info();

        let from = self.vault.to_account_info();
        let to = self.user.to_account_info();

        let accounts = Transfer {
            from: from.to_account_info(),
            to: to.to_account_info(),
        };

        let pda_signing_seeds = [
            b"vault",
            self.vault_state.to_account_info().key.as_ref(),
            &[self.vault_state.vault_bump],
        ];

        let seeds = &[&pda_signing_seeds[..]];

        // Needs to be signed by the vault PDA
        let cpi_context = CpiContext::new_with_signer(cpi_program, accounts, seeds);

        transfer(cpi_context, amount)
    }
}

#[derive(Accounts)]
pub struct Close<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"state", user.key().as_ref()],
        bump,
        close = user
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl Close<'_> {
    pub fn close(&mut self) -> Result<()> {
        let cpi_program = self.system_program.to_account_info();

        let from = self.vault.to_account_info();
        let to = self.user.to_account_info();

        let accounts = Transfer {
            from: from.to_account_info(),
            to: to.to_account_info(),
        };

        let pda_signing_seeds = [
            b"vault",
            self.vault_state.to_account_info().key.as_ref(),
            &[self.vault_state.vault_bump],
        ];

        let seeds = &[&pda_signing_seeds[..]];

        // Needs to be signed by the vault PDA
        let cpi_context = CpiContext::new_with_signer(cpi_program, accounts, seeds);

        transfer(cpi_context, self.vault.lamports())
    }
}

#[account]
#[derive(InitSpace)]
pub struct VaultState {
    pub vault_bump: u8,
    pub state_bump: u8,
}
