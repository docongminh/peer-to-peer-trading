use crate::constant::{ MintAddressType, TokenAccountType, STATE_PDA_SEED, VAULT_PDA_SEED };
use crate::error::EscrowError;
use crate::state::{ EscrowAccount, Stage, TradeType };
use crate::utils::{
    create_account,
    initialize_token_account,
    transfer_native_to_account,
    transfer_token_to_account,
};

use anchor_lang::prelude::*;
use anchor_spl::token::Token;

#[derive(Accounts)]
#[instruction(params: CreateParams)]
pub struct Create<'info> {
    #[account(
        init,
        payer = creator,
        seeds = [STATE_PDA_SEED, creator.key().as_ref(), params.order_id.to_le_bytes().as_ref()],
        bump,
        space = EscrowAccount::LEN
    )]
    pub escrow_state: Account<'info, EscrowAccount>,
    /// CHECK: This is init account state, this account will be create when identity trade type (SPL-SPL, SOL-SPL, SPL-SOL)
    #[account(mut,
    seeds=[VAULT_PDA_SEED, creator.key().as_ref(), params.order_id.to_le_bytes().as_ref()],
    bump = params.vault_bump
  )]
    pub escrow_vault: AccountInfo<'info>,
    #[account(mut, constraint = creator.lamports() > 0 && creator.data_is_empty())]
    pub creator: Signer<'info>,
    /// CHECK: This account use to send `Token` to swap (Token can be SOL or SPL Token)
    #[account(mut)]
    pub creator_send_account: AccountInfo<'info>,
    /// CHECK: This account use to receive `Token` swapped (Token can be SOL or SPL Token)
    #[account(mut)]
    pub creator_receive_account: AccountInfo<'info>,
    /// CHECK: receive fee for each deal
    #[account(mut)]
    pub fee_account: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Eq, PartialEq, Clone, Copy, Debug)]
pub struct CreateParams {
    // each trade deal has specify order id to identity
    pub order_id: u64,
    // in this case, user want to trade with specify partner
    pub specify_partner: Option<Pubkey>,
    // value of token user want to trade
    pub trade_value: u64,
    // value of token user expect to receive
    pub receive_value: u64,
    pub timestamp: u64,
    pub vault_bump: u8,
}

pub fn handler_create_trade<'info>(
    ctx: Context<'_, '_, '_, 'info, Create<'info>>,
    params: CreateParams
) -> Result<()> {
    // both trade value and receive value must be larger than zero
    require_gt!(params.trade_value, 0, EscrowError::ZeroValue);
    require_gt!(params.receive_value, 0, EscrowError::ZeroValue);

    // extract account
    // its work if it account is TokenAccount
    let creator_accounts: (TokenAccountType, TokenAccountType) = (
        Account::try_from(&ctx.accounts.creator_send_account),
        Account::try_from(&ctx.accounts.creator_receive_account),
    );
    // extract mint address account
    // its work if it account is Mint
    // use remaining_account to pass mint_address
    // index 0: creator trade token mint address
    // index 1: creator receive token token mint address
    let mints_address: (Option<MintAddressType>, Option<MintAddressType>) = (
        ctx.remaining_accounts.get(0).map(Account::try_from),
        ctx.remaining_accounts.get(1).map(Account::try_from),
    );
    let state_bump = *ctx.bumps.get("escrow_state").unwrap();
    let vault_bump = params.vault_bump;

    // init by trade type
    let trade_type: TradeType = match creator_accounts {
        /////// CASE 1: SPL <-> SPL
        // `creator_send_account` & `creator_receive_account` are Associate-Token-Account corresponding with mint addresses
        (Ok(creator_send_account), Ok(creator_receive_account)) =>
            match mints_address {
                (Some(Ok(mint_token_creator_trade)), Some(Ok(mint_token_creator_receive))) => {
                    // make sure valid `creator_send_account` token account with mint
                    require_eq!(
                        creator_send_account.mint,
                        mint_token_creator_trade.key(),
                        EscrowError::InvalidMint
                    );
                    // make sure valid `creator_receive_account` token account with mint
                    require_eq!(
                        creator_receive_account.mint,
                        mint_token_creator_receive.key(),
                        EscrowError::InvalidMint
                    );
                    // make sure creator is owner of `creator_send_account`
                    require_eq!(
                        creator_send_account.owner,
                        ctx.accounts.creator.key(),
                        EscrowError::InvalidOwner
                    );
                    // make sure creator is owner of `creator_receive_account`
                    require_eq!(
                        creator_receive_account.owner,
                        ctx.accounts.creator.key(),
                        EscrowError::InvalidOwner
                    );
                    // make sure never duplicate trade p2p between same token
                    require_neq!(
                        mint_token_creator_trade.key(),
                        mint_token_creator_receive.key(),
                        EscrowError::DuplicateMint
                    );
                    // make sure token balance of creator greater than or equal value_trade
                    require_gte!(
                        creator_send_account.amount,
                        params.trade_value,
                        EscrowError::InsufficientFunds
                    );

                    // create account associated token account
                    ctx.accounts.create_token_account_vault(
                        mint_token_creator_trade.to_account_info(),
                        vault_bump,
                        params.order_id
                    )?;

                    // transfer token to escrow account
                    ctx.accounts.transfer_token_to_vault(params.trade_value)?;

                    msg!("Created trading P2P between SPL <-> SPL. Now ready for trade");

                    //
                    ctx.accounts.escrow_state.creator_send_token_mint = Some(
                        mint_token_creator_trade.key()
                    );
                    ctx.accounts.escrow_state.creator_receive_token_mint = Some(
                        mint_token_creator_receive.key()
                    );
                    //
                    TradeType::TokenToken
                }
                (None, _) | (_, None) => {
                    return Err(EscrowError::MissingMint.into());
                }
                _ => {
                    return Err(EscrowError::InvalidMint.into());
                }
            }


        // CASE 2: SPL <-> SOL
        // Only `creator_send_account` provided. And Its corresponding with `mint_token_creator_trade`
        (Ok(creator_send_account), Err(_)) =>
            match mints_address {
                (Some(Ok(mint_token_creator_trade)), None) => {
                    // `creator_send_account` is Associate-Token-Account corresponding with mint address
                    require_eq!(
                        creator_send_account.mint,
                        mint_token_creator_trade.key(),
                        EscrowError::InvalidMint
                    );
                    // make sure creator is owner of `creator_send_account`
                    require_eq!(
                        creator_send_account.owner,
                        ctx.accounts.creator.key(),
                        EscrowError::InvalidOwner
                    );
                    // make sure token balance of creator greater than or equal value_trade
                    require_gte!(
                        creator_send_account.amount,
                        params.trade_value,
                        EscrowError::InsufficientFunds
                    );

                    // create token account
                    ctx.accounts.create_token_account_vault(
                        mint_token_creator_trade.to_account_info(),
                        vault_bump,
                        params.order_id
                    )?;

                    // transfer token to escrow vault token account
                    ctx.accounts.transfer_token_to_vault(params.trade_value)?;
                    msg!("Created trading P2P between SPL <-> SOL. Now ready for trade");
                    //
                    ctx.accounts.escrow_state.creator_send_token_mint = Some(
                        mint_token_creator_trade.key()
                    );
                    //
                    TradeType::TokenSol
                }
                (None, _) | (_, None) => {
                    return Err(EscrowError::MissingMint.into());
                }
                _ => {
                    return Err(EscrowError::InvalidMint.into());
                }
            }
        // CASE 3: SOL <-> SPL
        // Only `creator_receive_token_account` provided. And Its corresponding with `mint_token_creator_receive`
        (Err(_), Ok(creator_receive_account)) =>
            match mints_address {
                (Some(Ok(mint_token_creator_receive)), None) => {
                    // make sure mint of `creator_receive_account` is `mint_token_creator_receive`
                    require_eq!(
                        creator_receive_account.mint,
                        mint_token_creator_receive.key(),
                        EscrowError::InvalidMint
                    );
                    // make sure owner of `creator_receive_account` is creator
                    require_eq!(
                        creator_receive_account.owner,
                        ctx.accounts.creator.key(),
                        EscrowError::InvalidOwner
                    );
                    // make sure enough SOL for trade
                    require_gte!(
                        ctx.accounts.creator.lamports(),
                        params.trade_value,
                        EscrowError::InsufficientFunds
                    );

                    // create escrow vault account to hold lamports
                    ctx.accounts.create_native_account_vault(vault_bump, params.order_id)?;
                    // transfer SOL -> Vault Escrow
                    ctx.accounts.transfer_native_vault(params.trade_value)?;

                    msg!("Created trading P2P between SOL <-> SPL. Now ready for trade");
                    //
                    ctx.accounts.escrow_state.creator_receive_token_mint = Some(
                        mint_token_creator_receive.key()
                    );
                    // // trade type
                    TradeType::SolToken
                }
                (None, _) | (_, None) => {
                    return Err(EscrowError::MissingMint.into());
                }
                _ => {
                    return Err(EscrowError::InvalidMint.into());
                }
            }
        (Err(_), Err(_)) => {
            return Err(EscrowError::InvalidTradeType.into());
        }
    };

    // fill escrow config account data
    ctx.accounts.escrow_state.specify_partner = params.specify_partner;
    ctx.accounts.escrow_state.creator = ctx.accounts.creator.key();
    ctx.accounts.escrow_state.trade_type = trade_type.to_code();
    ctx.accounts.escrow_state.escrow_vault = ctx.accounts.escrow_vault.key();
    ctx.accounts.escrow_state.creator_send_account = ctx.accounts.creator_send_account.key();
    ctx.accounts.escrow_state.creator_receive_account = ctx.accounts.creator_receive_account.key();
    ctx.accounts.escrow_state.trade_value = params.trade_value;
    ctx.accounts.escrow_state.receive_value = params.receive_value;
    ctx.accounts.escrow_state.fee_account = ctx.accounts.fee_account.key();
    ctx.accounts.escrow_state.order_id = params.order_id;
    ctx.accounts.escrow_state.timestamp = params.timestamp;
    ctx.accounts.escrow_state.vault_bump = vault_bump;
    ctx.accounts.escrow_state.state_bump = state_bump;
    ctx.accounts.escrow_state.stage = Stage::ReadyExchange.to_code();
    Ok(())
}


impl<'info> Create<'info> {
    fn create_token_account_vault(
        &self,
        mint: AccountInfo<'info>,
        vault_bump: u8,
        order_id: u64
    ) -> Result<()> {
        //
        let space = anchor_spl::token::TokenAccount::LEN;
        // get signers seeds
        let creator_key = self.creator.key();
        let order_id_bytes = order_id.to_le_bytes();
        let vault_seed = &[
            &[
                VAULT_PDA_SEED,
                creator_key.as_ref(),
                order_id_bytes.as_ref(),
                bytemuck::bytes_of(&vault_bump),
            ][..],
        ];
        // create account
        // with pda vault token account
        // owner of program is Token program ID. Not itself program
        create_account(
            self.creator.to_account_info(),
            self.escrow_vault.to_account_info(),
            space,
            vault_seed,
            self.token_program.to_account_info(),
            self.rent.clone()
        )?;

        // init vault pda token account
        initialize_token_account(
            self.escrow_vault.to_account_info(),
            mint,
            self.escrow_state.to_account_info(),
            self.token_program.to_account_info(),
            self.rent.clone()
        )?;
        Ok(())
    }

    fn create_native_account_vault(&self, vault_bump: u8, order_id: u64) -> Result<()> {
        let creator_key = self.creator.key();
        let order_id_bytes = order_id.to_le_bytes();
        let vault_seed = &[
            &[
                VAULT_PDA_SEED,
                creator_key.as_ref(),
                order_id_bytes.as_ref(),
                bytemuck::bytes_of(&vault_bump),
            ][..],
        ];

        // create account
        // assign vault account for system program rather than itself program id
        create_account(
            self.creator.to_account_info(),
            self.escrow_vault.to_account_info(),
            0,
            vault_seed,
            self.system_program.to_account_info(),
            self.rent.clone()
        )?;
        Ok(())
    }
    //
    fn transfer_native_vault(&self, amount: u64) -> Result<()> {
        // transfer SOL creator -> vault
        transfer_native_to_account(
            self.creator.to_account_info(),
            self.escrow_vault.to_account_info(),
            amount,
            self.system_program.to_account_info(),
            None
        )?;
        Ok(())
    }

    fn transfer_token_to_vault(&self, amount: u64) -> Result<()> {
        // transfer Token creator -> vault
        transfer_token_to_account(
            self.creator_send_account.to_account_info(),
            self.escrow_vault.to_account_info(),
            self.creator.to_account_info(),
            amount,
            self.token_program.to_account_info(),
            None
        )?;
        Ok(())
    }
}