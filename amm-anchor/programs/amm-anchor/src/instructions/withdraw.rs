use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{burn, transfer_checked, Burn, Mint, Token, TokenAccount, TransferChecked},
};
use constant_product_curve::ConstantProduct;

use crate::error::AmmError;
use crate::Pool;

#[derive(Accounts)]
#[instruction(pool_id: u64)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    pub mint_x: Account<'info, Mint>,
    pub mint_y: Account<'info, Mint>,
    #[account(
        mut,
        seeds = [
            b"lp",
            pool.key().to_bytes().as_ref()
        ],
        bump = pool.lp_bump,
        mint::decimals = 6,
        mint::authority = pool,
    )]
    pub mint_lp: Account<'info, Mint>,

    #[account(
        mut, 
        associated_token::mint = mint_lp,
        associated_token::authority = pool,
        associated_token::token_program = token_program
    )]
    pub mint_lp_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = pool,
        associated_token::token_program = token_program
    )]
    pub vault_mint_x_ata: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = pool,
        associated_token::token_program = token_program
    )]
    pub vault_mint_y_ata: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = mint_x,
        associated_token::authority = owner,
        associated_token::token_program = token_program
    )]
    pub owner_mint_x_ata: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = mint_y,
        associated_token::authority = owner,
        associated_token::token_program = token_program
    )]
    pub owner_mint_y_ata: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint_lp,
        associated_token::authority = owner,
        associated_token::token_program = token_program
    )]
    pub owner_mint_lp_ata: Account<'info, TokenAccount>,
    #[account(
        seeds = [
            b"pool",
            mint_x.key().as_ref(),
            mint_y.key().as_ref(),
            pool_id.to_le_bytes().as_ref()
        ],
        bump = pool.pool_bump,
        has_one = owner,
    )]
    pub pool: Account<'info, Pool>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl Withdraw<'_> {
    pub fn withdraw_liquidity_from_pool(&self, amount: u64, min_x:u64, min_y:u64) -> Result<()> {
        require!(amount > 0, AmmError::InvalidAmount);
        require!(!self.pool.locked, AmmError::AMMLocked);

        let xy_amount = ConstantProduct::xy_withdraw_amounts_from_l(
            self.vault_mint_x_ata.amount,
            self.vault_mint_y_ata.amount,
            self.mint_lp.supply,
            amount,
            self.mint_lp.decimals as u32
        ).unwrap();

        require!(xy_amount.x >= min_x, AmmError::InsufficientTokenX );
        require!(xy_amount.y >= min_y, AmmError::InsufficientTokenY );

        self.withdraw_tokens_from_pool(true, xy_amount.x)?;
        self.withdraw_tokens_from_pool(false, xy_amount.y)?;
        self.burn_lp_tokens(amount)
    }

    fn withdraw_tokens_from_pool(&self, is_x: bool, amount: u64) -> Result<()> {
        let (from, to, mint, decimals) = match is_x {
            true => (
                self.vault_mint_x_ata.to_account_info(),
                self.owner_mint_x_ata.to_account_info(),
                self.mint_x.to_account_info(),
                self.mint_x.decimals,
            ),
            false => (
                self.vault_mint_y_ata.to_account_info(),
                self.owner_mint_y_ata.to_account_info(),
                self.mint_y.to_account_info(),
                self.mint_y.decimals,
            ),
        };

        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = TransferChecked {
            from,
            mint,
            to,
            authority: self.pool.to_account_info(),
        };

        let mint_x_bytes = self.mint_x.key().to_bytes();
        let mint_y_bytes = self.mint_y.key().to_bytes();

        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"pool",
            mint_x_bytes.as_ref(),
            mint_y_bytes.as_ref(),
            &self.pool.id.to_le_bytes(),
            &[self.pool.pool_bump],
        ]];

        let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, &signer_seeds);

        transfer_checked(cpi_context, amount, decimals)
    }

    fn burn_lp_tokens(&self, amount: u64) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = Burn {
            mint: self.mint_lp.to_account_info(),
            from: self.owner_mint_lp_ata.to_account_info(),
            authority: self.owner.to_account_info(),
        };

        let cpi_context = CpiContext::new(cpi_program, cpi_accounts);

        burn(cpi_context, amount)
    }
}
