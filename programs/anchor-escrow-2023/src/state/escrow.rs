use anchor_lang::prelude::*;

use crate::constants::{ANCHOR_DISCRIMINATOR_BYTES, PUBKEY_BYTES, U64_BYTES, U8_BYTES};

#[account]
pub struct Escrow {
    pub maker: Pubkey,
    pub maker_token: Pubkey,
    pub taker_token: Pubkey,
    pub offer_amount: u64,
    pub seed: u64,
    pub auth_bump: u8,
    pub vault_bump: u8,
    pub escrow_bump: u8
}

impl Escrow {
    pub const LEN: usize = ANCHOR_DISCRIMINATOR_BYTES + 3 * PUBKEY_BYTES + 2 * U64_BYTES + 3 * U8_BYTES;
}