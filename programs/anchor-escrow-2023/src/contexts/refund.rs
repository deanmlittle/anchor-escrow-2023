use anchor_lang::prelude::*;
use anchor_spl::{token::{TokenAccount, Mint, Token, Transfer, transfer, CloseAccount, close_account}, associated_token::AssociatedToken};

use crate::structs::Escrow;

#[derive(Accounts)]
pub struct Refund<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,
    #[account(
        mut,
        associated_token::mint = maker_token,
        associated_token::authority = maker
    )]
    pub maker_ata: Account<'info, TokenAccount>,
    pub maker_token: Account<'info, Mint>,
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
        has_one = maker_token,
        seeds = [b"escrow", maker.key.as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.escrow_bump,
        close = maker
    )]
    pub escrow: Box<Account<'info, Escrow>>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>   
}

impl<'info> Refund<'info> {
    pub fn empty_vault(&self) -> Result<()> {
        let signer_seeds = &[
            &b"auth"[..],
            &[self.escrow.auth_bump],
        ];
        // &[&[&[b"auth"[..], &self.escrow.auth_bump.to_le_bytes()]]];
        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.maker_ata.to_account_info(),
            authority: self.auth.to_account_info(),
        };
        let binding = [&signer_seeds[..]];
        let ctx = CpiContext::new_with_signer(self.token_program.to_account_info(), cpi_accounts, &binding);
        transfer(ctx, self.vault.amount)
    }

    pub fn close_vault(&self) -> Result<()> {
        let signer_seeds = &[
            &b"auth"[..],
            &[self.escrow.auth_bump],
        ];
        let cpi_accounts = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.maker.to_account_info(),
            authority: self.auth.to_account_info(),
        };
        let binding = [&signer_seeds[..]];
        let ctx = CpiContext::new_with_signer(self.token_program.to_account_info(), cpi_accounts, &binding);
        close_account(ctx)
    }
}