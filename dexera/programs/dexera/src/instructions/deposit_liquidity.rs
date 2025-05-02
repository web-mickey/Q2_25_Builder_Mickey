use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        mint_to, transfer_checked, Mint, MintTo, TokenAccount, TokenInterface, TransferChecked,
    },
};
use constant_product_curve::ConstantProduct;

use crate::{errors::ErrorCode, Pool};

#[derive(Accounts)]
pub struct DepositLiquidity<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,

    pub mint_x: InterfaceAccount<'info, Mint>,
    pub mint_y: InterfaceAccount<'info, Mint>,

    #[account(
        seeds = [b"lp", pool.key().as_ref()],
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

    #[account(mut)]
    pub depositor_mint_x_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub depositor_mint_y_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = depositor,
        associated_token::mint = mint_lp,
        associated_token::authority = depositor,
        associated_token::token_program = token_program,
        mint::token_program = token_program,
    )]
    pub depositor_mint_lp_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        seeds = [
            b"pool",
            mint_x.key().as_ref(),
            mint_y.key().as_ref(),
        ],
        bump = pool.pool_bump,
    )]
    pub pool: Account<'info, Pool>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl DepositLiquidity<'_> {
    pub fn deposit_liquidity(
        &self,
        lp_tokens_amount: u64,
        max_x_tokens: u64,
        max_y_tokens: u64,
    ) -> Result<()> {
        require!(!self.pool.locked, ErrorCode::AMMLocked);
        require!(lp_tokens_amount > 0, ErrorCode::InvalidAmount);

        let (x, y) = if self.pool_vault_x_ata.amount == 0
            && self.pool_vault_y_ata.amount == 0
            && self.mint_lp.supply == 0
        {
            (max_x_tokens, max_y_tokens)
        } else {
            let tokens_amount = ConstantProduct::xy_deposit_amounts_from_l(
                self.pool_vault_x_ata.amount,
                self.pool_vault_y_ata.amount,
                self.mint_lp.supply,
                lp_tokens_amount,
                6,
            )
            .unwrap();
            (tokens_amount.x, tokens_amount.y)
        };

        require!(max_x_tokens >= x, ErrorCode::InsufficientTokenX);
        require!(max_y_tokens >= y, ErrorCode::InsufficientTokenY);

        self.deposit_token(true, x)?;
        self.deposit_token(false, y)?;
        self.mint_lp_tokens(lp_tokens_amount)?;

        Ok(())
    }

    fn deposit_token(&self, is_x: bool, amount: u64) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();

        let (from, mint, to, authority, decimals) = if is_x {
            (
                self.depositor_mint_x_ata.to_account_info(),
                self.mint_x.to_account_info(),
                self.pool_vault_x_ata.to_account_info(),
                self.depositor.to_account_info(),
                self.mint_x.decimals,
            )
        } else {
            (
                self.depositor_mint_y_ata.to_account_info(),
                self.mint_y.to_account_info(),
                self.pool_vault_y_ata.to_account_info(),
                self.depositor.to_account_info(),
                self.mint_y.decimals,
            )
        };

        let cpi_accounts = TransferChecked {
            from,
            mint,
            to,
            authority,
        };
        let cpi_context = CpiContext::new(cpi_program, cpi_accounts);

        transfer_checked(cpi_context, amount, decimals)
    }

    fn mint_lp_tokens(&self, amount: u64) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = MintTo {
            mint: self.mint_lp.to_account_info(),
            to: self.depositor_mint_lp_ata.to_account_info(),
            authority: self.pool.to_account_info(),
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

        mint_to(cpi_context, amount)
    }
}
