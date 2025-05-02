use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{get_associated_token_address, AssociatedToken}, token::{transfer_checked, TransferChecked}, token_interface::{Mint, TokenAccount, TokenInterface}
};
use constant_product_curve::{ConstantProduct, LiquidityPair};

use crate::{errors::ErrorCode, Pool, Profile, ProtocolConfig};

#[derive(Accounts)]
pub struct SwapTokens<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    pub mint_x: InterfaceAccount<'info, Mint>,
    pub mint_y: InterfaceAccount<'info, Mint>,

    #[account(
        seeds = [b"lp", pool.key().as_ref()],
        bump = pool.mint_lp_bump, 
        mint::decimals = 6,
        mint::authority = pool,
    )]
    pub mint_lp: InterfaceAccount<'info, Mint>,

    pub profile: Option<Account<'info, Profile>>,

    #[account(
        seeds = [b"config"],
        bump,
    )]
    pub config: Account<'info, ProtocolConfig>,
    
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
    pub user_mint_x_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint_y,
        associated_token::authority = user,
        associated_token::token_program = token_program,
        mint::token_program = token_program,
    )]
    pub user_mint_y_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        seeds = [
            b"pool",
            mint_x.key().as_ref(),
            mint_y.key().as_ref(),
        ],
        bump = pool.pool_bump,
    )]
    pub pool: Account<'info, Pool>,

    /// CHECK: This is the referrer's ATA that will be created if it doesn't exist
    #[account(mut)]
    pub referrer_ata: Option<UncheckedAccount<'info>>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl SwapTokens<'_> {
    pub fn swap_exact_out(&self, max_amount_in:u64, amount_out:u64, program_id: &Pubkey, referrer: Option<Pubkey>) -> Result<()> {
        require!(!self.pool.locked, ErrorCode::AMMLocked);
        require!(amount_out > 0, ErrorCode::InvalidAmount);

        let mut curve = ConstantProduct::init(self.pool_vault_x_ata.amount, self.pool_vault_y_ata.amount, self.mint_lp.supply, self.config.fee,None ).unwrap();

        let swap_result = curve.swap(LiquidityPair::Y, max_amount_in, amount_out).map_err(ErrorCode::from)?;

        require!(swap_result.withdraw >= amount_out, ErrorCode::SlippageExceeded);

        self.deposit_from_user_to_pool(false, swap_result.deposit)?;
        self.withdraw_from_pool_to_user(true, swap_result.withdraw)?;

        if swap_result.fee > 0 {
            self.charge_fee(program_id, swap_result.fee, referrer)?;
        }

        Ok(())
    }

    pub fn swap_exact_in(&self, amount_in: u64, min_amount_out: u64, program_id: &Pubkey, referrer: Option<Pubkey>)-> Result<()> {
        require!(!self.pool.locked, ErrorCode::AMMLocked);
        require!(min_amount_out > 0, ErrorCode::InvalidAmount);

        let mut curve = ConstantProduct::init(self.pool_vault_x_ata.amount, self.pool_vault_y_ata.amount, self.mint_lp.supply, self.config.fee,None ).unwrap();

        let swap_result = curve.swap(LiquidityPair::X, amount_in, min_amount_out).map_err(ErrorCode::from)?;

        require!(swap_result.withdraw >= min_amount_out, ErrorCode::SlippageExceeded);

        self.deposit_from_user_to_pool(true, swap_result.deposit)?;
        self.withdraw_from_pool_to_user(false, swap_result.withdraw)?;

        if swap_result.fee > 0 {
            self.charge_fee(program_id,swap_result.fee, referrer)?;
        }

        Ok(())
    }

    fn deposit_from_user_to_pool(&self, is_token: bool, amount: u64) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();
        let (from, mint, to, authority, decimals) = if is_token {
            (
                self.user_mint_x_ata.to_account_info(),
                self.mint_x.to_account_info(),
                self.pool_vault_x_ata.to_account_info(),
                self.user.to_account_info(),
                self.mint_x.decimals,
            )
        } else {
            (
                self.user_mint_y_ata.to_account_info(),
                self.mint_y.to_account_info(),
                self.pool_vault_y_ata.to_account_info(),
                self.user.to_account_info(),
                self.mint_y.decimals,
            )
        };

        let cpi_accounts = TransferChecked { from, mint, to, authority };
        let cpi_context = CpiContext::new(cpi_program, cpi_accounts);

        transfer_checked(cpi_context, amount, decimals)
    }

    fn withdraw_from_pool_to_user(&self, is_token: bool, amount: u64) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();
        let (from, mint, to, authority, decimals) = if is_token {
            (
                self.pool_vault_x_ata.to_account_info(),
                self.mint_x.to_account_info(),
                self.user_mint_x_ata.to_account_info(),
                self.pool.to_account_info(),
                self.mint_x.decimals,
            )
        } else {
            (
                self.pool_vault_y_ata.to_account_info(),
                self.mint_y.to_account_info(),
                self.user_mint_y_ata.to_account_info(),
                self.pool.to_account_info(),
                self.mint_y.decimals,
            )
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

        let cpi_accounts = TransferChecked { from, mint, to, authority };
        let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        transfer_checked(cpi_context, amount, decimals)
    }

    fn charge_fee(
        &self,
        program_id: &Pubkey,
        total_fee: u64,
        referrer: Option<Pubkey>,
    ) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();
    
        let mint_x_bytes = self.mint_x.key().to_bytes();
        let mint_y_bytes = self.mint_y.key().to_bytes();
    
        let seeds = [
            b"pool",
            mint_x_bytes.as_ref(),
            mint_y_bytes.as_ref(),
            &[self.pool.pool_bump],
        ];
        let signer_seeds = &[&seeds[..]];
    
        let decimals = self.mint_y.decimals;
    
        let (ref_fee, protocol_fee) = if referrer.is_some() {
            (total_fee * 2 / 3, total_fee / 3)
        } else {
            (0, total_fee)
        };
    
        let from = self.pool_vault_y_ata.to_account_info();
        let mint = self.mint_y.to_account_info();
    
        if let Some(referrer_key) = referrer {
            // Derive the profile PDA
            let (profile_pda, _) = Pubkey::find_program_address(
                &[b"profile", referrer_key.as_ref()],
                program_id,
            );
    
            // Check if profile exists and is valid
            if let Some(profile_account) = self.profile.as_ref() {
                if profile_account.key() == profile_pda {
                    // Profile exists and is valid, send referral fee
                    if let Some(referrer_ata) = self.referrer_ata.as_ref() {
                        let cpi_accounts = TransferChecked {
                            from: from.clone(),
                            mint: mint.clone(),
                            to: referrer_ata.to_account_info(),
                            authority: self.pool.to_account_info(),
                        };
                        let cpi_context =
                            CpiContext::new_with_signer(cpi_program.clone(), cpi_accounts, signer_seeds);
                        transfer_checked(cpi_context, ref_fee, decimals)?;
                    }
                }
            }
            // If profile doesn't exist or is invalid, the ref_fee will be added to protocol_fee
        }
    
        // Always transfer protocol fee (including any ref_fee if profile was invalid)
        let protocol_fee = if self.profile.is_none() {
            total_fee // Send full fee to protocol if no profile
        } else {
            protocol_fee
        };

        let cpi_accounts = TransferChecked {
            from,
            mint,
            to: self.user_mint_y_ata.to_account_info(),
            authority: self.pool.to_account_info(),
        };
        let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        transfer_checked(cpi_context, protocol_fee, decimals)
    }    
}
