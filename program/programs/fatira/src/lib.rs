use anchor_lang::prelude::*;
use anchor_lang::solana_program::program_pack::Pack;
use spl_token::state::{Account as SplAccount, Mint as SplMint};
mod state;
mod error;
mod constants;
use crate::state::{Group};
use crate::error::{ErrorCode};
use crate::constants::{GROUP_SIZE, MAX_GROUP_USERS};


declare_id!("ftra545Ysk9H9HjvhfqXh5xP5PTQTC1KV3rk4AADXeC");

#[program]
pub mod fatira {
    use super::*;

    pub fn create_group(ctx: Context<CreateGroup>) -> Result<()> {
        let group = &mut ctx.accounts.group;
        let payer = &ctx.accounts.payer;
        let currency = &ctx.accounts.currency;
        let escrow = &ctx.accounts.escrow;
        let (escrow_authority, _) = Pubkey::find_program_address(&[b"escrow", group.key().as_ref()], ctx.program_id);

        let currency_data = &currency.try_borrow_data()?;
        let _currency_token_account = SplMint::unpack(currency_data).map_err(|_| error!(ErrorCode::InvalidCurrencyAccount))?;
        let escrow_data = &escrow.try_borrow_data()?;
        let escrow_token_account = SplAccount::unpack(escrow_data).map_err(|_| error!(ErrorCode::InvalidEscrowAccount))?;

        require_eq!(currency.owner, escrow.owner, ErrorCode::InconsistentTokenPrograms);
        require_eq!(escrow_token_account.owner, escrow_authority, ErrorCode::InconsistentEscrowOwner);
        require_eq!(escrow_token_account.mint, currency.key(), ErrorCode::InconsistentEscrowMint);
        require!(escrow_token_account.delegate.is_none(), ErrorCode::EscrowHasDelegate);
        require!(!escrow_token_account.is_frozen(), ErrorCode::EscrowIsFrozen);

        group.currency = currency.key();
        group.escrow = escrow.key();
        group.balances = Vec::with_capacity(MAX_GROUP_USERS);
        group.add_balance(payer.key(), 0)?;
        Ok(())
    }

    pub fn update_balances(
        ctx: Context<UpdateBalances>,
        total_cost: i64,
        users: Vec<Pubkey>,
        amounts: Vec<i64>
    ) -> Result<()> {
        let group = &mut ctx.accounts.group;
        let payer = &ctx.accounts.payer;

        require_eq!(users.len(), amounts.len(), ErrorCode::InconsistentBalanceLengths);
        require!(total_cost > 0, ErrorCode::AmountIsNegative);

        group.change_balance(payer.key(), -1 * total_cost)?;
        for (i, amount) in amounts.iter().enumerate() {
            require!(*amount > 0, ErrorCode::AmountIsNegative);
            group.change_balance(users[i], *amount)?;
        }

        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateGroup<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = GROUP_SIZE,
    )]
    pub group: Account<'info, Group>,

    /// CHECK: verify that currency is a valid mint account
    pub currency: AccountInfo<'info>,

    /// CHECK: verify that escrow is a valid token account with the correct owner, mint, delegate, and frozen status
    pub escrow: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateBalances<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut)]
    pub group: Account<'info, Group>,

    pub system_program: Program<'info, System>,
}