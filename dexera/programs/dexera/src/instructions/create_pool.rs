use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint,MintTo, mint_to, TokenAccount, TokenInterface, TransferChecked},
};
use constant_product_curve::ConstantProduct;

use crate::{errors::ErrorCode, Pool};

#[derive(Accounts)]
pub struct CreatePool<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    pub mint_x: InterfaceAccount<'info, Mint>,
    pub mint_y: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = creator,
        seeds = [b"lp", pool.key().as_ref()],
        bump, 
        mint::decimals = 6,
        mint::authority = pool,
    )]
    pub mint_lp: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = creator,
        associated_token::mint = mint_x,
        associated_token::authority = pool,
        associated_token::token_program = token_program,
        mint::token_program = token_program,
    )]
    pub pool_vault_x_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = creator,
        associated_token::mint = mint_y,
        associated_token::authority = pool,
        associated_token::token_program = token_program,
        mint::token_program = token_program,
    )]
    pub pool_vault_y_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = creator,
        associated_token::mint = mint_lp,
        associated_token::authority = pool,
        associated_token::token_program = token_program,
        mint::token_program = token_program,
    )]
    pub pool_mint_lp_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub creator_mint_x_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub creator_mint_y_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = creator,
        associated_token::mint = mint_lp,
        associated_token::authority = creator,
        associated_token::token_program = token_program,
        mint::token_program = token_program,
    )]
    pub creator_mint_lp_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = creator,
        seeds = [b"pool", mint_x.key().as_ref(), mint_y.key().as_ref()],
        space = Pool::INIT_SPACE,
        bump
    )]
    pub pool: Account<'info, Pool>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl CreatePool<'_> {
    pub fn create_pool_state(
        &mut self,
        bumps: CreatePoolBumps,
    ) -> Result<()> {
        self.pool.set_inner(Pool {
            creator: self.creator.key(),
            mint_x: self.mint_x.key(),
            mint_y: self.mint_y.key(),
            mint_lp: self.mint_lp.key(),
            pool_bump: bumps.pool,
            mint_lp_bump: bumps.mint_lp,
            locked: false
        });

        // Add initial liquidity
        self.deposit_tokens(1_000_000, 1_000_000, 1_000_000)?;

        Ok(())
    }

    pub fn deposit_tokens(&self, lp_tokens_amount:u64, max_x_tokens:u64, max_y_tokens:u64)->Result<()>{
        require!(lp_tokens_amount > 0, ErrorCode::InvalidAmount);

        let (x,y) = match self.pool_vault_x_ata.amount == 0 && self.pool_vault_y_ata.amount ==0 && self.mint_lp.supply == 0{
            true => (max_x_tokens, max_y_tokens),
            false => {
                let tokens_amount = ConstantProduct::xy_deposit_amounts_from_l(self.pool_vault_x_ata.amount, self.pool_vault_y_ata.amount, self.mint_lp.supply, lp_tokens_amount, 6).unwrap();

                (tokens_amount.x, tokens_amount.y)
            },
        };

        require!(max_x_tokens>=x, ErrorCode::InsufficientTokenX);
        require!(max_y_tokens>=y, ErrorCode::InsufficientTokenY);

        self.deposit_token(true, x)?;
        self.deposit_token(false, y)?;
        self.mint_lp_tokens(lp_tokens_amount)?;

        Ok(())
    }

    fn deposit_token(&self,is_x:bool, amount:u64)->Result<()>{
        let cpi_program = self.token_program.to_account_info();


        let (from, mint, to,authority, decimals) = match is_x{
            true => (self.creator_mint_x_ata.to_account_info(), self.mint_x.to_account_info(),  self.pool_vault_x_ata.to_account_info(),  self.creator.to_account_info(), self.mint_x.decimals),
            false => (self.creator_mint_y_ata.to_account_info(), self.mint_y.to_account_info(),  self.pool_vault_y_ata.to_account_info(),  self.creator.to_account_info(), self.mint_y.decimals),
        };

        let cpi_accounts = TransferChecked { from, mint, to, authority };

        let cpi_context = CpiContext::new(cpi_program, cpi_accounts);

        transfer_checked(cpi_context, amount, decimals)
    }

    fn mint_lp_tokens(&self, amount:u64)->Result<()>{
        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = MintTo{ mint: self.mint_lp.to_account_info(), to: self.creator_mint_lp_ata.to_account_info(), authority: self.pool.to_account_info() };

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
