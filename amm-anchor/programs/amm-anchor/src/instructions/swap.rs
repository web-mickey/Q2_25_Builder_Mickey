use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount, TransferChecked, transfer_checked},
};
use constant_product_curve::{ConstantProduct, LiquidityPair, SwapResult};

use crate::Pool;
use crate::error::AmmError;


#[derive(Accounts)]
#[instruction(pool_id: u64)]
pub struct Swap<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    pub owner:SystemAccount<'info>,
    
    pub mint_x: Account<'info, Mint>,
    pub mint_y: Account<'info, Mint>,

    #[account(
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
        associated_token::token_program = token_program,
    )]
    pub mint_lp_ata: Account<'info, TokenAccount>,
    #[account(
        mut, 
        associated_token::mint = mint_x,
        associated_token::authority = pool,
        associated_token::token_program = token_program,
    )]
    pub vault_x_ata: Account<'info, TokenAccount>,
    #[account(
        mut, 
        associated_token::mint = mint_y,
        associated_token::authority = pool,
        associated_token::token_program = token_program,
    )]
    pub vault_y_ata: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = signer, 
        associated_token::mint = mint_x,
        associated_token::authority = signer,
        associated_token::token_program = token_program,
    )]
    pub signer_x_ata: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = mint_y,
        associated_token::authority = signer,
        associated_token::token_program = token_program,
    )]
    pub signer_y_ata: Account<'info, TokenAccount>,

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

impl Swap<'_>{
    pub fn swap_tokens(&self, is_x:bool, amount:u64, min:u64) -> Result<()>{
        require!(!self.pool.locked,AmmError::AMMLocked );
        require!(amount>0, AmmError::InvalidAmount);

        let mut curve = ConstantProduct::init(self.vault_x_ata.amount, self.vault_y_ata.amount, amount, self.pool.fee,None).unwrap();


        let p = match is_x {
            true => LiquidityPair::X,
            false => LiquidityPair::Y,
        };

        let res = curve.swap(p, amount, min).map_err(AmmError::from)?;

        require_neq!(res.deposit, 0, AmmError::InvalidAmount);
        require_neq!(res.withdraw, 0, AmmError::InvalidAmount);

        let swap_result = SwapResult{ 
            deposit: res.deposit,
            withdraw: res.withdraw - 1,
            fee: res.fee 
        };

        if is_x {
            // Swap X for Y: deposit X, withdraw Y
            self.deposit_tokens_to_vault(true, swap_result.deposit)?;
            self.withdraw_tokens_from_vault(false, swap_result.withdraw)?;
        } else {
            // Swap Y for X: deposit Y, withdraw X
            self.deposit_tokens_to_vault(false, swap_result.deposit)?;
            self.withdraw_tokens_from_vault(true, swap_result.withdraw)?;
        }

        Ok(())
    }

    fn deposit_tokens_to_vault (&self, is_x:bool, amount: u64) -> Result<()>{
        let cpi_program = self.token_program.to_account_info();

       let (cpi_accounts, mint_decimals) = match is_x{
            true => (
                TransferChecked{
                    from: self.signer_x_ata.to_account_info(),
                    to: self.vault_x_ata.to_account_info(),
                    mint:self.mint_x.to_account_info(),
                    authority: self.signer.to_account_info()
                },
                self.mint_x.decimals
            ),
            false => (
                TransferChecked{
                    from: self.signer_y_ata.to_account_info(),
                    to: self.vault_y_ata.to_account_info(),
                    mint: self.mint_y.to_account_info(),
                    authority: self.signer.to_account_info()
                }, self.mint_y.decimals
            )
       };

       let cpi_context = CpiContext::new(cpi_program, cpi_accounts);

       transfer_checked(cpi_context, amount, mint_decimals)
    }

    fn withdraw_tokens_from_vault (&self, is_x:bool, amount:u64) -> Result<()>{
        let cpi_program = self.token_program.to_account_info();

       let (cpi_accounts, mint_decimals) = match is_x{
            true => (
                TransferChecked{
                    from: self.vault_x_ata.to_account_info(),
                    to: self.signer_x_ata.to_account_info(),
                    mint:self.mint_x.to_account_info(),
                    authority: self.pool.to_account_info()
                },
                self.mint_x.decimals
            ),
            false => (
                TransferChecked{
                    from: self.vault_y_ata.to_account_info(),
                    to: self.signer_y_ata.to_account_info(),
                    mint: self.mint_y.to_account_info(),
                    authority: self.pool.to_account_info()
                }, self.mint_y.decimals
            )
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

       transfer_checked(cpi_context, amount, mint_decimals)
    }
}