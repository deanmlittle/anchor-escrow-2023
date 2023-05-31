
use anchor_lang::prelude::*;

declare_id!("8RifdHyuiX5yDUYJRGBHHpY1K8tyoQuUhR69TstPh2p1");

mod errors;
mod state;
mod contexts;
mod constants;
use contexts::*;

#[program]
pub mod anchor_escrow_2023_timed {
    use super::*;

    pub fn make(
        ctx: Context<Make>,
        seed: u64,
        deposit_amount: u64,
        offer_amount: u64,
        expiry: u64
    ) -> Result<()> {
        ctx.accounts.init(&ctx.bumps, seed, offer_amount, expiry)?;
        ctx.accounts.transfer_to_vault(deposit_amount)
    }

    // Cancel and refund escrow to the maker
    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        ctx.accounts.empty_vault()?;
        ctx.accounts.close_vault()
    }

    // Allow maker to change the token and offer amount of the escrow
    pub fn update(
        ctx: Context<Update>,
        offer_amount: u64,
        expiry: u64
    ) -> Result<()> {
        ctx.accounts.update(offer_amount, expiry)
    }

    // Allow taker to accept the escrow
    pub fn take(ctx: Context<Take>) -> Result<()> {
        ctx.accounts.check_expiry()?;
        ctx.accounts.deposit_to_maker()?;
        ctx.accounts.empty_vault_to_taker()?;
        ctx.accounts.close_vault()
    }
}