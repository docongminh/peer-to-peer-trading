use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};

pub const VAULT_PDA_SEED: &[u8] = b"vault";
pub const STATE_PDA_SEED: &[u8] = b"state";

pub type TokenAccountType<'info> = std::result::Result<Account<'info, TokenAccount>, Error>;
pub type MintAddressType<'info> = std::result::Result<Account<'info, Mint>, Error>;
