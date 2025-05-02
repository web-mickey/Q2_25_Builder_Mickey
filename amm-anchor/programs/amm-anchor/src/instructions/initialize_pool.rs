use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token::{mint_to, transfer_checked, Mint, MintTo, Token, TokenAccount, TransferChecked},
};
use constant_product_curve::ConstantProduct;

use crate::{error::AmmError, Pool};

#[derive(Accounts)]
#[instruction(pool_id:u64)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    pub mint_x: Account<'info, Mint>,
    pub mint_y: Account<'info, Mint>,

    #[account(
        init,
        payer=owner,
        seeds = [
            b"lp",
            pool_state.key().to_bytes().as_ref(),
        ],
        bump,
        mint::decimals = 6,
        mint::authority = pool_state,
    )]
    pub mint_lp: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = mint_lp,
        associated_token::authority = pool_state,
        associated_token::token_program = token_program
    )]
    pub mint_lp_ata: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = mint_lp,
        associated_token::authority = owner,
        associated_token::token_program = token_program
    )]
    pub owner_mint_lp_ata: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = mint_x,
        associated_token::authority = pool_state,
        associated_token::token_program = token_program
    )]
    pub vault_x_ata: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = mint_y,
        associated_token::authority = pool_state,
        associated_token::token_program = token_program
    )]
    pub vault_y_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = owner,
        associated_token::token_program = token_program
    )]
    pub owner_x_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = owner,
        associated_token::token_program = token_program
    )]
    pub owner_y_ata: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = owner,
        seeds = [
            b"pool",
            mint_x.key().to_bytes().as_ref(),
            mint_y.key().to_bytes().as_ref(),
            pool_id.to_le_bytes().as_ref(),
        ],
        bump,
        space = 8 + Pool::INIT_SPACE,
    )]
    pub pool_state: Account<'info, Pool>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl InitializePool<'_> {
    pub fn initialize(&mut self, pool_id: u64, fee: u16, bumps: InitializePoolBumps) -> Result<()> {
        self.pool_state.set_inner(Pool {
            id: pool_id,
            owner: self.owner.key(),
            mint_x: self.mint_x.key(),
            mint_y: self.mint_y.key(),
            mint_lp: self.mint_lp.key(),
            fee,
            pool_bump: bumps.pool_state,
            lp_bump: bumps.mint_lp,
            locked: false,
        });

        Ok(())
    }

    pub fn deposit(&mut self, amount: u64, max_x: u64, max_y: u64) -> Result<()> {
        require!(amount > 0, AmmError::InvalidAmount);
        require!(!self.pool_state.locked, AmmError::AMMLocked);

        let (x, y) = match self.vault_x_ata.amount == 0
            && self.vault_y_ata.amount == 0
            && self.mint_lp.supply == 0
        {
            true => (max_x, max_y),
            false => {
                let amount = ConstantProduct::xy_deposit_amounts_from_l(
                    self.vault_x_ata.amount,
                    self.vault_y_ata.amount,
                    self.mint_lp.supply,
                    amount,
                    6,
                )
                .unwrap();

                (amount.x, amount.y)
            }
        };

        require!(max_x >= x, AmmError::InsufficientTokenX);
        require!(max_y >= y, AmmError::InsufficientTokenY);

        self.deposit_token(true, max_x)?;
        self.deposit_token(false, max_y)?;

        self.mint_lp_tokens(amount)
    }

    pub fn deposit_token(&self, is_x: bool, amount: u64) -> Result<()> {
        let (from, to, mint, decimals) = match is_x {
            true => (
                self.owner_x_ata.to_account_info(),
                self.vault_x_ata.to_account_info(),
                self.mint_x.to_account_info(),
                self.mint_x.decimals,
            ),
            false => (
                self.owner_y_ata.to_account_info(),
                self.vault_y_ata.to_account_info(),
                self.mint_y.to_account_info(),
                self.mint_y.decimals,
            ),
        };

        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = TransferChecked {
            from,
            mint,
            to,
            authority: self.owner.to_account_info(),
        };

        let cpi_context = CpiContext::new(cpi_program, cpi_accounts);

        transfer_checked(cpi_context, amount, decimals)
    }

    pub fn mint_lp_tokens(&self, amount: u64) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = MintTo {
            mint: self.mint_lp.to_account_info(),
            to: self.owner_mint_lp_ata.to_account_info(),
            authority: self.pool_state.to_account_info(),
        };

        let mint_x_bytes = self.mint_x.key().to_bytes();
        let mint_y_bytes = self.mint_y.key().to_bytes();

        let seeds = [
            b"pool",
            mint_x_bytes.as_ref(),
            mint_y_bytes.as_ref(),
            &self.pool_state.id.to_le_bytes(),
            &[self.pool_state.pool_bump],
        ];

        let signer_seeds = &[&seeds[..]];

        let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        mint_to(cpi_context, amount)
    }
}
