use anchor_lang::prelude::*;
use anchor_spl::{token::{TokenAccount, Mint, Transfer, Token, transfer, close_account, CloseAccount}, associated_token::AssociatedToken};

use crate::structs::Escrow;

#[derive(Accounts)]
pub struct Take<'info> {
    #[account(mut)]
    pub maker: SystemAccount<'info>,
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = taker_token,
        associated_token::authority = maker
    )]
    pub maker_receive_ata: Account<'info, TokenAccount>,
    pub maker_token: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub taker: Signer<'info>,
    #[account(
        mut,
        associated_token::mint = taker_token,
        associated_token::authority = taker
    )]
    pub taker_ata: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = maker_token,
        associated_token::authority = taker
    )]
    pub taker_receive_ata: Account<'info, TokenAccount>,
    pub taker_token: Box<Account<'info, Mint>>,
    #[account(
        seeds = [b"auth"],
        bump = escrow.auth_bump
    )]
    /// CHECK: This is not dangerous because this account doesn't exist
    pub auth: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [b"vault", escrow.key().as_ref()],
        bump = escrow.vault_bump,
        token::mint = maker_token,
        token::authority = auth
    )]
    pub vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        has_one = maker,
        has_one = taker_token,
        has_one = maker_token,
        seeds = [b"escrow", maker.key.as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.escrow_bump,
        close = taker
    )]
    pub escrow: Box<Account<'info, Escrow>>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>
}

impl<'info> Take<'info> {
    pub fn deposit_to_maker(&self) -> Result<()> {
        let cpi_accounts = Transfer {
            from: self.taker_ata.to_account_info(),
            to: self.maker_receive_ata.to_account_info(),
            authority: self.taker.to_account_info(),
        };
        let ctx = CpiContext::new(self.token_program.to_account_info(), cpi_accounts);
        transfer(ctx, self.escrow.offer_amount)
    }

    pub fn empty_vault_to_taker(&self) -> Result<()> {
        
        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.taker_receive_ata.to_account_info(),
            authority: self.auth.to_account_info(),
        };
        let signer_seeds = &[
            &b"auth"[..],
            &[self.escrow.auth_bump],
        ];
        let binding = [&signer_seeds[..]];
        let ctx = CpiContext::new_with_signer(self.token_program.to_account_info(), cpi_accounts, &binding);
        transfer(ctx, self.vault.amount)
    }

    pub fn close_vault(&self) -> Result<()> {
        let cpi_accounts = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.taker.to_account_info(),
            authority: self.auth.to_account_info(),
        };
        let signer_seeds = &[
            &b"auth"[..],
            &[self.escrow.auth_bump],
        ];
        let binding = [&signer_seeds[..]];
        let ctx = CpiContext::new_with_signer(self.token_program.to_account_info(), cpi_accounts, &binding);
        close_account(ctx)
    }
}