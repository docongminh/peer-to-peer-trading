use anchor_lang::prelude::*;

use crate::error::EscrowError;

// Trading type between users
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum TradeType {
  TokenToken,
  SolToken,
  TokenSol,
}

impl TradeType {
  pub fn from(code: u8) -> Result<TradeType> {
    match code {
      1 => Ok(TradeType::TokenToken),
      2 => Ok(TradeType::TokenSol),
      3 => Ok(TradeType::SolToken),
      unknown_code => {
        msg!("Unknow trade type: {}", unknown_code);
        Err(EscrowError::InvalidStage.into())
      }
    }
  }
  pub fn to_code(&self) -> u8 {
    match self {
      TradeType::TokenToken => 1,
      TradeType::TokenSol => 2,
      TradeType::SolToken => 3,
    }
  }
}

#[account]
pub struct EscrowAccount {
  pub creator: Pubkey,
  pub partner: Pubkey,
  pub specify_partner: Option<Pubkey>,
  pub fee_account: Pubkey,
  pub trade_token_mint: Pubkey,
  pub receive_token_mint: Pubkey,
  pub escrow_vault: Pubkey,
  pub creator_send_account: Pubkey,
  pub creator_receive_account: Pubkey,
  pub creator_send_token_mint: Option<Pubkey>,
  pub creator_receive_token_mint: Option<Pubkey>,
  pub trade_value: u64,
  pub receive_value: u64,
  pub timestamp: u64,
  pub order_id: u64,
  pub state_bump: u8,
  pub vault_bump: u8,
  pub trade_type: u8,
  pub stage: u8,
}

impl EscrowAccount {
  pub const LEN: usize = 8
    + 32 * 8 // PubKey
    + 33 * 3 // Option pubkey
    + 8 * 4 // u64
    + 1 * 4; // u8
}

// define stage of deal
#[derive(Clone, Copy, PartialEq)]
pub enum Stage {
  ReadyExchange,
  Exchanged,
  CancelTrade,
}

impl Stage {

  pub fn from(code: u8) -> Result<Stage> {
    match code {
      1 => Ok(Stage::ReadyExchange),
      2 => Ok(Stage::Exchanged),
      3 => Ok(Stage::CancelTrade),
      unknown_code => {
        msg!("Unknow state: {}", unknown_code);
        Err(EscrowError::InvalidStage.into())
      }
    }
  }

  pub fn to_code(&self) -> u8 {
    match self {
      Stage::ReadyExchange => 1,
      Stage::Exchanged => 2,
      Stage::CancelTrade => 3,
    }
  }
}
