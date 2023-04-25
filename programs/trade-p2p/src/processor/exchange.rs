use crate::constant::{ TokenAccountType, STATE_PDA_SEED, VAULT_PDA_SEED };
use crate::error::EscrowError;

use crate::state::{ EscrowAccount, Stage, TradeType };
use crate::utils::{
    close_native_account,
    close_token_account,
    transfer_native_to_account,
    transfer_token_to_account,
};

use anchor_lang::prelude::*;
use anchor_spl::token::Token;

#[derive(Accounts)]
#[instruction(order_id: u64, state_bump: u8, vault_bump: u8)]
pub struct Exchange<'info> {
    #[account(
        mut,
        has_one=creator,
        has_one=escrow_vault,
        seeds=[STATE_PDA_SEED, creator.key().as_ref(), order_id.to_le_bytes().as_ref()],
        bump = state_bump,
        constraint = escrow_state.stage == Stage::ReadyExchange.to_code() @ EscrowError::InvalidStage
    )]
    pub escrow_state: Account<'info, EscrowAccount>,
    /// CHECK: this account use to transfer token to receiverF
    #[account(mut,
    seeds=[VAULT_PDA_SEED, creator.key().as_ref(), escrow_state.order_id.to_le_bytes().as_ref()],
    bump = vault_bump
  )]
    pub escrow_vault: AccountInfo<'info>,
    /// CHECK: This account use to receive `Token` (Token can be SOL or SPL Token)
    #[account(mut,
    constraint = creator_receive_account.key() == escrow_state.creator_receive_account @ EscrowError::InvalidAccount
  )]
    pub creator_receive_account: AccountInfo<'info>,
    /// CHECK: This account use to send token to creator
    #[account(mut)]
    pub partner_send_account: AccountInfo<'info>,
    /// CHECK: This account use to receive token from escrow vault
    #[account(mut)]
    pub partner_receive_account: AccountInfo<'info>,
    /// CHECK: TODO
    #[account(mut)]
    pub creator: AccountInfo<'info>,
    #[account(mut, constraint = partner.lamports() > 0 && partner.data_is_empty())]
    pub partner: Signer<'info>,
    // system
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
}

pub fn handler_exchange(ctx: Context<Exchange>) -> Result<()> {
    // extract in case have specify partner to exchange
    let has_specify_partner = ctx.accounts.escrow_state.specify_partner;
    match has_specify_partner {
        Some(has_specify_partner) => {
            require_eq!(
                has_specify_partner,
                ctx.accounts.partner.key(),
                EscrowError::InvalidPartner
            );
        }
        None => {}
    }

    let trade_value = ctx.accounts.escrow_state.trade_value;
    let receive_value = ctx.accounts.escrow_state.receive_value;
    //
    let trade_type = TradeType::from(ctx.accounts.escrow_state.trade_type);
    match trade_type {
        // Case SPL - SPL
        Ok(TradeType::TokenToken) => {
            // convert `partner_send_account` to TokenAccount
            let partner_token_accounts: (TokenAccountType, TokenAccountType) = (
                Account::try_from(&ctx.accounts.partner_send_account),
                Account::try_from(&ctx.accounts.partner_receive_account),
            );
            match partner_token_accounts {
                (Ok(partner_send_token_account), Ok(partner_receive_token_account)) => {
                    // make sure token balance of partner enought for trade
                    let partner_token_balance = partner_send_token_account.amount;
                    require_gte!(
                        partner_token_balance,
                        receive_value,
                        EscrowError::InsufficientFunds
                    );
                    // make sure `partner_send_account` is associated token account with `creator_receive_token_mint`
                    let creator_receive_token_mint =
                        ctx.accounts.escrow_state.creator_receive_token_mint.unwrap();
                    require_eq!(
                        creator_receive_token_mint,
                        partner_send_token_account.mint,
                        EscrowError::InvalidAccount
                    );
                    // make sure `partner_receive_token_account` is associated token account with `creator_send_token_mint`
                    let creator_send_token_mint =
                        ctx.accounts.escrow_state.creator_send_token_mint.unwrap();
                    require_eq!(
                        creator_send_token_mint,
                        partner_receive_token_account.mint,
                        EscrowError::InvalidAccount
                    );
                    //
                    require_eq!(
                        partner_receive_token_account.owner,
                        ctx.accounts.partner.key(),
                        EscrowError::InvalidOwner
                    );
                    //
                    require_eq!(
                        partner_send_token_account.owner,
                        ctx.accounts.partner.key(),
                        EscrowError::InvalidOwner
                    );
                }
                _ => {
                    return Err(EscrowError::InvalidAccount.into());
                }
            }
            // transfer TOKEN: escrow vault -> partner
            ctx.accounts.transfer_to_partner_token(trade_value)?;
            // transfer TOKEN: partner -> Creator
            ctx.accounts.transfer_to_creator_token(receive_value)?;
            // close vault token account
            ctx.accounts.close_vault_token()?;
        }

        // Case SPL - SOL
        Ok(TradeType::TokenSol) => {
            let partner_receive_token_account: TokenAccountType = Account::try_from(
                &ctx.accounts.partner_receive_account
            );
            match partner_receive_token_account {
                Ok(partner_receive_token_account) => {
                    //
                    require_eq!(
                        partner_receive_token_account.mint,
                        ctx.accounts.escrow_state.creator_send_token_mint.unwrap(),
                        EscrowError::InvalidAccount
                    );
                    //
                    require_eq!(
                        partner_receive_token_account.owner,
                        ctx.accounts.partner.key(),
                        EscrowError::InvalidOwner
                    );
                }
                _ => {
                    return Err(EscrowError::InvalidAccount.into());
                }
            }
            // sol balance of partner enough for trade
            require_gte!(
                ctx.accounts.partner.lamports(),
                receive_value,
                EscrowError::InsufficientFunds
            );

            // transfer TOKEN: escrow vault -> partner
            ctx.accounts.transfer_to_partner_token(trade_value)?;
            // transfer SOL: partner -> Creator
            ctx.accounts.transfer_to_creator_native(receive_value)?;
            ctx.accounts.close_vault_token()?;
        }
        // Case SOL - SPL
        Ok(TradeType::SolToken) => {
            let partner_token_account: TokenAccountType = Account::try_from(
                &ctx.accounts.partner_send_account
            );
            match partner_token_account {
                Ok(partner_token_account) => {
                    // check mint
                    let creator_receive_token_mint =
                        ctx.accounts.escrow_state.creator_receive_token_mint.unwrap();
                    require_eq!(
                        creator_receive_token_mint,
                        partner_token_account.mint,
                        EscrowError::InvalidMint
                    );
                    // check owner
                    require_eq!(
                        partner_token_account.owner,
                        ctx.accounts.partner.key(),
                        EscrowError::InvalidOwner
                    );
                    // check balance
                    require_gte!(
                        partner_token_account.amount,
                        receive_value,
                        EscrowError::InsufficientFunds
                    );
                }
                _ => {
                    return Err(EscrowError::InvalidAccount.into());
                }
            }
            //
            require_eq!(
                ctx.accounts.creator_receive_account.key(),
                ctx.accounts.escrow_state.creator_receive_account,
                EscrowError::InvalidAccount
            );
            // Transfer SOL: Vault -> partner
            ctx.accounts.transfer_to_partner_native(trade_value)?;

            // Transfer SPL: partner -> creator
            ctx.accounts.transfer_to_creator_token(receive_value)?;

            // Close SPL Vault
            ctx.accounts.close_vault_native()?;
        }
        // error case
        _ => {
            return Err(EscrowError::InvalidTradeType.into());
        }
    }
    ctx.accounts.escrow_state.specify_partner = Some(ctx.accounts.partner.key());
    ctx.accounts.escrow_state.stage = Stage::Exchanged.to_code();
    Ok(())
}

impl<'info> Exchange<'info> {
    fn transfer_to_partner_native(&self, amount: u64) -> Result<()> {
        // transfer SOL escrow_vault -> partner
        let creator_key = self.creator.key();
        let order_id_bytes = self.escrow_state.order_id.to_le_bytes();
        let vault_bump = self.escrow_state.vault_bump;
        let vault_seed = &[
            &[
                VAULT_PDA_SEED,
                creator_key.as_ref(),
                order_id_bytes.as_ref(),
                bytemuck::bytes_of(&vault_bump),
            ][..],
        ];
        transfer_native_to_account(
            self.escrow_vault.to_account_info(),
            self.partner.to_account_info(),
            amount,
            self.system_program.to_account_info(),
            Some(vault_seed)
        )?;
        Ok(())
    }

    fn transfer_to_partner_token(&self, amount: u64) -> Result<()> {
        let creator_key = self.creator.key();
        let order_id_bytes = self.escrow_state.order_id.to_le_bytes();
        let state_bump = self.escrow_state.state_bump;
        let state_signers_seeds = &[
            &[
                STATE_PDA_SEED,
                creator_key.as_ref(),
                order_id_bytes.as_ref(),
                bytemuck::bytes_of(&state_bump),
            ][..],
        ];
        // transfer token escrow_vault -> partner
        transfer_token_to_account(
            self.escrow_vault.to_account_info(),
            self.partner_receive_account.to_account_info(),
            self.escrow_state.to_account_info(),
            amount,
            self.token_program.to_account_info(),
            Some(state_signers_seeds)
        )?;
        Ok(())
    }

    fn transfer_to_creator_native(&self, amount: u64) -> Result<()> {
        // transfer SOL partner -> creator
        transfer_native_to_account(
            self.partner.to_account_info(),
            self.creator.to_account_info(),
            amount,
            self.system_program.to_account_info(),
            None
        )?;
        Ok(())
    }

    fn transfer_to_creator_token(&self, amount: u64) -> Result<()> {
        // transfer token partner -> creator
        transfer_token_to_account(
            self.partner_send_account.to_account_info(),
            self.creator_receive_account.to_account_info(),
            self.partner.to_account_info(),
            amount,
            self.token_program.to_account_info(),
            None
        )?;
        Ok(())
    }

    fn close_vault_token(&self) -> Result<()> {
        let creator = self.creator.key();
        let state_bump = self.escrow_state.state_bump;
        let order_id = self.escrow_state.order_id.to_le_bytes();
        let signers_seeds = &[
            &[STATE_PDA_SEED, creator.as_ref(), order_id.as_ref(), bytemuck::bytes_of(&state_bump)]
                [..],
        ];

        close_token_account(
            self.escrow_vault.to_account_info(),
            self.creator.to_account_info(),
            self.escrow_state.to_account_info(),
            signers_seeds,
            self.token_program.to_account_info()
        )?;
        Ok(())
    }

    fn close_vault_native(&self) -> Result<()> {
        let creator = self.creator.key();
        let vault_bump = self.escrow_state.vault_bump;
        let order_id = self.escrow_state.order_id.to_le_bytes();
        let vault_signers_seeds = &[
            &[VAULT_PDA_SEED, creator.as_ref(), order_id.as_ref(), bytemuck::bytes_of(&vault_bump)]
                [..],
        ];
        close_native_account(
            self.escrow_vault.to_account_info(),
            self.creator.to_account_info(),
            vault_signers_seeds,
            self.system_program.to_account_info()
        )?;
        Ok(())
    }
}