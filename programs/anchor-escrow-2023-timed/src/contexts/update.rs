use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

use crate::state::Escrow;

#[derive(Accounts)]
pub struct Update<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,
    pub new_taker_token: Account<'info, Mint>,
    #[account(
        mut,
        has_one = maker,
        seeds = [b"escrow", maker.key.as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.escrow_bump
    )]
    pub escrow: Box<Account<'info, Escrow>>,
    pub system_program: Program<'info, System>   
}

impl<'info> Update<'info> {
    pub fn update(&mut self, offer_amount: u64, expiry: u64 ) -> Result<()> {
        self.escrow.taker_token = *self.new_taker_token.to_account_info().key;
        self.escrow.offer_amount = offer_amount;
        self.escrow.set_expiry_relative(expiry)
    }
}