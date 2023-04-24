#[macro_use]
pub mod seed;

pub mod constant;
pub mod error;
pub mod processor;
pub mod state;
pub mod utils;

use crate::processor::*;
use anchor_lang::prelude::*;

declare_id!("EJV62xsWEZ5Kbzy7QNR8ogvDDQYqMkdN31UyCqkeaHDe");

#[program]
pub mod trade_p2p {
    use super::*;

    pub fn create_trade<'info>(
        ctx: Context<'_, '_, '_, 'info, Create<'info>>,
        params: CreateParams,
    ) -> Result<()> {
        handler_create_trade(ctx, params)?;
        Ok(())
    }

    pub fn exchange(
        ctx: Context<Exchange>,
        _order_id: u64,
        _state_bump: u8,
        _vault_bump: u8,
    ) -> Result<()> {
        handler_exchange(ctx)?;
        Ok(())
    }
    //
    pub fn cancel(
        ctx: Context<Cancel>,
        _order_id: u64,
        _state_bump: u8,
        _vault_bump: u8,
    ) -> Result<()> {
        handler_cancel(ctx)?;
        Ok(())
    }
}
