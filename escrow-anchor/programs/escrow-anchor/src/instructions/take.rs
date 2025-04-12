use anchor_lang::prelude::*;

use crate::Escrow;

use anchor_spl::{
    associated_token::AssociatedToken,
    token::{close_account, CloseAccount},
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

#[derive(Accounts)]
pub struct Take<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,

    #[account(mut)]
    pub maker: SystemAccount<'info>,

    #[account(
        mint::token_program = token_program
    )]
    pub mint_a: InterfaceAccount<'info, Mint>,

    #[account(
        mint::token_program = token_program
    )]
    pub mint_b: InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = mint_b,
        associated_token::authority = maker,
        associated_token::token_program = token_program,
    )]
    pub maker_ata_b: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = mint_a,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub taker_ata_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = taker,
        associated_token::token_program = token_program
    )]
    pub taker_ata_b: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program
    )]
    pub vault_ata_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        close = maker,
        has_one = mint_b,
        has_one = mint_a,
        has_one = maker,
        seeds = [
            b"escrow",
            escrow.maker.as_ref(),
            escrow.escrow_id.to_le_bytes().as_ref()
        ],
        bump = escrow.bump,
    )]
    pub escrow: Account<'info, Escrow>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl Take<'_> {
    pub fn send(&mut self) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = TransferChecked {
            from: self.taker_ata_b.to_account_info(),
            to: self.maker_ata_b.to_account_info(),
            authority: self.taker.to_account_info(),
            mint: self.mint_b.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        transfer_checked(cpi_ctx, self.escrow.receive_mint_b, self.mint_b.decimals)?;

        Ok(())
    }

    pub fn withdraw(&mut self) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();

        let transfer_checked_options = TransferChecked {
            from: self.vault_ata_a.to_account_info(),
            to: self.taker_ata_a.to_account_info(),
            authority: self.escrow.to_account_info(),
            mint: self.mint_a.to_account_info(),
        };

        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"escrow",
            self.maker.key.as_ref(),
            &self.escrow.escrow_id.to_le_bytes()[..],
            &[self.escrow.bump],
        ]];

        let cpi_context =
            CpiContext::new_with_signer(cpi_program, transfer_checked_options, &signer_seeds);

        transfer_checked(cpi_context, self.vault_ata_a.amount, self.mint_a.decimals)
    }

    pub fn close(&mut self) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();

        let close_options = CloseAccount {
            account: self.vault_ata_a.to_account_info(),
            destination: self.maker.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"escrow",
            self.maker.key.as_ref(),
            &self.escrow.escrow_id.to_le_bytes()[..],
            &[self.escrow.bump],
        ]];

        let cpi_context = CpiContext::new_with_signer(cpi_program, close_options, &signer_seeds);

        close_account(cpi_context)
    }
}
