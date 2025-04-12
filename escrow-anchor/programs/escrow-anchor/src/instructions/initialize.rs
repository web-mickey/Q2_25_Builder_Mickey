use anchor_lang::prelude::*;

use crate::{Escrow, ANCHOR_DISCRIMINATOR};

use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

#[derive(Accounts)]
#[instruction(escrow_id:u64)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    #[account(
        mint::token_program = token_program
    )]
    pub mint_a: InterfaceAccount<'info, Mint>,

    #[account(
        mint::token_program = token_program
    )]
    pub mint_b: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub user_ata_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = maker,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        mint::token_program = token_program,
    )]
    pub vault_ata_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init, 
        payer = maker,
        space = 8 + Escrow::INIT_SPACE,
        seeds = [b"escrow", maker.key().as_ref(), escrow_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub escrow: Account<'info, Escrow>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl Initialize<'_> {
    pub fn initalize(&mut self, escrow_id: u64, receive_mint_b: u64, bumps: &InitializeBumps) -> Result<()> {
        self.escrow.set_inner(Escrow {
            escrow_id,
            maker: self.maker.key(),
            mint_a: self.mint_a.key(),
            mint_b: self.mint_b.key(),
            receive_mint_b,
            // It cannot be self.escrow.bump because it does not exist already
            // you dumb lmao
            bump: bumps.escrow,
        });

        Ok(())
    }

    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();

        let transfer_checked_options = TransferChecked {
            from: self.user_ata_a.to_account_info(),
            to: self.vault_ata_a.to_account_info(),
            mint: self.mint_a.to_account_info(),
            authority: self.maker.to_account_info(),
        };

        let cpi_context = CpiContext::new(cpi_program, transfer_checked_options);

        transfer_checked(cpi_context, amount, self.mint_a.decimals)
    }
}
