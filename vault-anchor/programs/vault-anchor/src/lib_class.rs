#![allow(unexpected_cfgs)]
use anchor_lang::{
    prelude::*, solana_program::{program::invoke, system_instruction}, system_program::{transfer, Transfer}
};

declare_id!("DZA294uXy8zdcW8FZEXYCHMJ6Z1NriEuT7D6V5S1vjQH");

#[program]
pub mod vault_anchor {
    use super::*;

    pub fn create_vault(ctx: Context<CreateVault>) -> Result<()> {
        ctx.accounts.create_vault(&ctx.bumps)
    }

    pub fn deposit(ctx: Context<Deposit>, amount:u64) -> Result<()> {
        ctx.accounts.deposit(amount)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount:u64) -> Result<()> {
        ctx.accounts.withdraw(amount)
    }

    pub fn close(ctx: Context<Close>) -> Result<()> {
        ctx.accounts.close()
    }
}

// This is not only setting up the accounts but it already creates them if needed
// #[account(init does that
#[derive(Accounts)]
pub struct CreateVault<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init,
        payer = user, 
        seeds = [b"state", user.key().as_ref()],
        bump,
        space = VaultState::INIT_SPACE
    )]
    pub vault_state: Account<'info, VaultState>,
    
    #[account(
        seeds =[b"vault", vault_state.key().as_ref()],
        bump,
    )]
    pub vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl CreateVault<'_> {
    // This function is actually used to fill the data of vault state
    pub fn create_vault(&mut self, bumps : &CreateVaultBumps) -> Result<()> {
        self.vault_state.state_bump = bumps.vault_state;
        self.vault_state.vault_bump = bumps.vault;
        Ok(())
    }   
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        seeds = [b"state", user.key().as_ref()],
        bump,
    )]
    pub vault_state: Account<'info, VaultState>,
    
    #[account(
        mut,
        seeds =[b"vault", vault_state.key().as_ref()],
        bump,
    )]
    pub vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl Deposit<'_>{
    pub fn deposit(&mut self, amount:u64) -> Result<()> {
        let cpi_program = self.system_program.to_account_info();

        let from = self.user.to_account_info();
        let to = self.vault.to_account_info(); 

        let accounts = Transfer{
            from: from.to_account_info(),
            to: to.to_account_info(),
        }; 

        let cpi_context = CpiContext::new(cpi_program, accounts);

        transfer(cpi_context, amount)
    }

    pub fn deposit2(ctx: Context<Deposit>, amount:u64) -> Result<()> {
        let from_pubkey = ctx.accounts.vault.to_account_info();
        let to_pubkey = ctx.accounts.user.to_account_info();
        let program_id = ctx.accounts.system_program.to_account_info();
    
        let instruction =
            &system_instruction::transfer(&from_pubkey.key(), &to_pubkey.key(), amount);
    
        invoke(instruction, &[from_pubkey, to_pubkey, program_id])?;
    
        Ok(())
    }
}


#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        seeds = [b"state", user.key().as_ref()],
        bump,
    )]
    pub vault_state: Account<'info, VaultState>,
    
    #[account(
        mut,
        seeds =[b"vault", vault_state.key().as_ref()],
        bump,
    )]
    pub vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl Withdraw<'_>{
    pub fn withdraw(&mut self, amount:u64) -> Result<()> {
        let cpi_program = self.system_program.to_account_info();

        let from = self.vault.to_account_info();
        let to = self.user.to_account_info(); 

        let accounts = Transfer{
            from: from.to_account_info(),
            to: to.to_account_info(),
        };

        let pda_signing_seeds = [
            b"vault",
            self.vault_state.to_account_info().key.as_ref(),
             &[self.vault_state.vault_bump]
        ];

        let seeds = &[&pda_signing_seeds[..]];

        // Needs to be signed by the vault PDA
        let cpi_context = CpiContext::new_with_signer(cpi_program, accounts,seeds);

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
        seeds =[b"vault", vault_state.key().as_ref()],
        bump,
    )]
    pub vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl Close<'_>{
    pub fn close(&mut self) -> Result<()> {
        let cpi_program = self.system_program.to_account_info();

        let from = self.vault.to_account_info();
        let to = self.user.to_account_info(); 

        let accounts = Transfer{
            from: from.to_account_info(),
            to: to.to_account_info(),
        };

        let pda_signing_seeds = [
            b"vault",
            self.vault_state.to_account_info().key.as_ref(),
             &[self.vault_state.vault_bump]
        ];

        let seeds = &[&pda_signing_seeds[..]];

        let cpi_context = CpiContext::new_with_signer(cpi_program, accounts,seeds);

        transfer(cpi_context, self.vault.lamports())
    }
}

#[account]
pub struct VaultState {
    pub vault_bump: u8,
    pub state_bump: u8,
}

impl Space for VaultState {
    const INIT_SPACE:usize = 8 + 1+1;
}