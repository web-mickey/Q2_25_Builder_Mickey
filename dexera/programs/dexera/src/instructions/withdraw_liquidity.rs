use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer_checked, TransferChecked},
    token_interface::{burn, Burn, Mint, TokenAccount, TokenInterface},
};
use constant_product_curve::ConstantProduct;

use crate::{errors::ErrorCode, Pool};

#[derive(Accounts)]
pub struct WithdrawLiquidity<'info> {
    #[account(mut)]
    pub withdrawer: Signer<'info>,

    pub mint_x: InterfaceAccount<'info, Mint>,
    pub mint_y: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds=[
            b"lp",
            pool.key().as_ref()
        ],
        bump = pool.mint_lp_bump,
        mint::decimals = 6,
        mint::authority = pool,
    )]
    pub mint_lp: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = pool,
        associated_token::token_program = token_program,
        mint::token_program = token_program,
    )]
    pub pool_vault_x_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = pool,
        associated_token::token_program = token_program,
        mint::token_program = token_program,
    )]
    pub pool_vault_y_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_lp,
        associated_token::authority = pool,
        associated_token::token_program = token_program,
        mint::token_program = token_program,
    )]
    pub pool_mint_lp_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = withdrawer,
        associated_token::mint = mint_lp,
        associated_token::authority = withdrawer,
        associated_token::token_program = token_program,
        mint::token_program = token_program,
    )]
    pub withdrawer_mint_lp_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = withdrawer,
        associated_token::mint = mint_x,
        associated_token::authority = withdrawer,
        associated_token::token_program = token_program,
        mint::token_program = token_program,
    )]
    pub withdrawer_mint_x_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = withdrawer,
        associated_token::mint = mint_y,
        associated_token::authority = withdrawer,
        associated_token::token_program = token_program,
        mint::token_program = token_program,
    )]
    pub withdrawer_mint_y_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        seeds = [
            b"pool",
            mint_x.key().as_ref(),
            mint_y.key().as_ref()
        ],
        bump = pool.pool_bump
    )]
    pub pool: Account<'info, Pool>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl WithdrawLiquidity<'_> {
    pub fn withdraw_liquidity(&self, amount: u64, min_x: u64, min_y: u64) -> Result<()> {
        require!(amount > 0, ErrorCode::InvalidAmount);

        let amounts = ConstantProduct::xy_withdraw_amounts_from_l(
            self.pool_vault_x_ata.amount,
            self.pool_vault_y_ata.amount,
            self.mint_lp.supply,
            amount,
            6,
        )
        .unwrap();

        require!(amounts.x >= min_x, ErrorCode::InsufficientTokenX);
        require!(amounts.y >= min_y, ErrorCode::InsufficientTokenY);

        self.withdraw_tokens(true, amounts.x)?;
        self.withdraw_tokens(false, amounts.y)?;

        self.burn_lp_tokens(amount)?;

        Ok(())
    }

    fn withdraw_tokens(&self, is_x: bool, amount: u64) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();
        let (from, mint, to, authority, decimals) = match is_x {
            true => (
                self.pool_vault_x_ata.to_account_info(),
                self.mint_x.to_account_info(),
                self.withdrawer_mint_x_ata.to_account_info(),
                self.pool.to_account_info(),
                self.mint_x.decimals,
            ),
            false => (
                self.pool_vault_y_ata.to_account_info(),
                self.mint_y.to_account_info(),
                self.withdrawer_mint_y_ata.to_account_info(),
                self.pool.to_account_info(),
                self.mint_y.decimals,
            ),
        };

        let cpi_accounts = TransferChecked {
            from,
            mint,
            to,
            authority,
        };

        let mint_x_bytes = self.mint_x.key().to_bytes();
        let mint_y_bytes = self.mint_y.key().to_bytes();

        let seeds = [
            b"pool",
            mint_x_bytes.as_ref(),
            mint_y_bytes.as_ref(),
            &[self.pool.pool_bump],
        ];

        let signer_seeds = &[&seeds[..]];

        let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        transfer_checked(cpi_context, amount, decimals)
    }

    fn burn_lp_tokens(&self, amount: u64) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = Burn {
            mint: self.mint_lp.to_account_info(),
            from: self.withdrawer_mint_lp_ata.to_account_info(),
            authority: self.withdrawer.to_account_info(),
        };

        let cpi_context = CpiContext::new(cpi_program, cpi_accounts);

        burn(cpi_context, amount)
    }
}
