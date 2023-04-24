use crate::constant::{STATE_PDA_SEED, VAULT_PDA_SEED};
use crate::error::EscrowError;

use crate::state::{EscrowAccount, Stage, TradeType};
use crate::utils::{
  close_native_account, close_token_account, transfer_native_to_account, transfer_token_to_account,
};

use anchor_lang::prelude::*;
use anchor_spl::token::Token;

#[derive(Accounts)]
#[instruction(order_id: u64, state_bump: u8, vault_bump: u8)]
pub struct Cancel<'info> {
  #[account(
        mut,
        has_one=creator,
        has_one=escrow_vault @ EscrowError::InvalidAccount,
        has_one=creator_send_account @ EscrowError::InvalidOwner,
        seeds=[STATE_PDA_SEED, creator.key().as_ref(), order_id.to_le_bytes().as_ref()],
        bump = state_bump,
        constraint = escrow_state.stage == Stage::ReadyExchange.to_code() @ EscrowError::InvalidStage
    )]
  pub escrow_state: Account<'info, EscrowAccount>,
  /// CHECK: TODO
  #[account(mut,
    seeds=[VAULT_PDA_SEED, creator.key().as_ref(), order_id.to_le_bytes().as_ref()],
    bump = vault_bump
  )]
  pub escrow_vault: AccountInfo<'info>,
  /// CHECK: This account use to receive `Token` (Token can be SOL or SPL Token)
  #[account(mut)]
  pub creator_send_account: AccountInfo<'info>,
  #[account(mut, constraint = creator.lamports() > 0 && creator.data_is_empty())]
  pub creator: Signer<'info>,
  // system
  system_program: Program<'info, System>,
  token_program: Program<'info, Token>,
}

pub fn handler_cancel(ctx: Context<Cancel>, ) -> Result<()> {
  let trade_type = TradeType::from(ctx.accounts.escrow_state.trade_type);
  let creator_send_account = ctx.accounts.escrow_state.creator_send_account;
  match trade_type {
    Ok(TradeType::SolToken) => {
      require_eq!(
        creator_send_account,
        ctx.accounts.creator.key(),
        EscrowError::InvalidOwner
      );
      ctx.accounts.with_draw_native()?;
      // close vault native account
      ctx.accounts.close_vault_native()?;
    }
    //
    Ok(TradeType::TokenToken) | Ok(TradeType::TokenSol) => {
      // make sure account withdraw token to invalid with account of creator send token to vault
      require_eq!(
        creator_send_account,
        ctx.accounts.creator_send_account.key(),
        EscrowError::InvalidOwner
      );
      // Transfer SPL from Vault to Creator
      ctx.accounts.with_draw_token()?;
      // Close SPL Vault
      ctx.accounts.close_vault_token()?;
    }
    _ => return Err(EscrowError::InvalidTradeType.into()),
  }
  ctx.accounts.escrow_state.stage = Stage::CancelTrade.to_code();
  Ok(())
}

impl<'info> Cancel<'info> {
  fn with_draw_native(&self) -> Result<()> {
    // withdraw SOL escrow_vault -> creator
    let amount = self.escrow_state.trade_value;
    let creator_key = self.creator.key();
    let order_id_bytes = self.escrow_state.order_id.to_le_bytes();
    let vault_bump = self.escrow_state.vault_bump;
    let vault_signers_seeds = &[&[
      VAULT_PDA_SEED,
      creator_key.as_ref(),
      order_id_bytes.as_ref(),
      bytemuck::bytes_of(&vault_bump),
    ][..]];
    transfer_native_to_account(
      self.escrow_vault.to_account_info(),
      self.creator.to_account_info(),
      amount,
      self.system_program.to_account_info(),
      Some(vault_signers_seeds),
    )?;
    Ok(())
  }

  fn with_draw_token(&self) -> Result<()> {
    let creator = self.creator.key();
    let state_bump = self.escrow_state.state_bump;
    let order_id = self.escrow_state.order_id.to_le_bytes();
    let seeds = &[&[
      STATE_PDA_SEED,
      creator.as_ref(),
      order_id.as_ref(),
      bytemuck::bytes_of(&state_bump),
    ][..]];
    // transfer Token escrow_vault -> creator_send_account.
    // Withdraw to exactly account transfer token escrow vault
    // Why not `creator_receive_account` ?
    // Because `creator_receive_account` && `creator_send_account` are different.
    // We can send a token to exchange and receive another token
    transfer_token_to_account(
      self.escrow_vault.to_account_info(),
      self.creator_send_account.to_account_info(),
      self.escrow_state.to_account_info(),
      self.escrow_state.trade_value,
      self.token_program.to_account_info(),
      Some(seeds),
    )?;
    Ok(())
  }

  fn close_vault_token(&self) -> Result<()> {
    let creator = self.creator.key();
    let state_bump = self.escrow_state.state_bump;
    let order_id = self.escrow_state.order_id.to_le_bytes();
    let signers_seeds = &[&[
      STATE_PDA_SEED,
      creator.as_ref(),
      order_id.as_ref(),
      bytemuck::bytes_of(&state_bump),
    ][..]];
    close_token_account(
      self.escrow_vault.to_account_info(),
      self.creator.to_account_info(),
      self.escrow_state.to_account_info(),
      signers_seeds,
      self.token_program.to_account_info(),
    )?;
    Ok(())
  }

  fn close_vault_native(&self) -> Result<()> {
    let vault_bump = self.escrow_state.vault_bump;
    let creator_pubkey = self.creator.key();
    let order_id = self.escrow_state.order_id.to_le_bytes();
    let vault_signers_seeds = &[&[
      VAULT_PDA_SEED,
      creator_pubkey.as_ref(),
      order_id.as_ref(),
      bytemuck::bytes_of(&vault_bump),
    ][..]];
    close_native_account(
      self.escrow_vault.to_account_info(),
      self.creator.to_account_info(),
      vault_signers_seeds,
      self.system_program.to_account_info(),
    )?;
    Ok(())
  }
}
