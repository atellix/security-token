use std::{ result::Result as FnResult };
use solana_program::{ account_info::AccountInfo };
use anchor_lang::prelude::*;

use net_authority::TokenGroupApproval;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[inline]
fn load_struct<T: AccountDeserialize>(acc: &AccountInfo) -> FnResult<T, ProgramError> {
    let mut data: &[u8] = &acc.try_borrow_data()?;
    Ok(T::try_deserialize(&mut data)?)
}

#[program]
pub mod security_token {
    use super::*;
    pub fn create_mint(ctx: Context<CreateMint>,
        inp_decimals: u8,
        inp_url: String,
    ) -> anchor_lang::Result<()> {
        let mint = &mut ctx.accounts.mint;
        mint.manager = *ctx.accounts.manager.to_account_info().key;
        mint.net_auth = *ctx.accounts.net_auth.to_account_info().key;
        mint.group = *ctx.accounts.group.to_account_info().key;
        mint.supply = 0;
        mint.decimals = inp_decimals;
        mint.url = inp_url;
        msg!("Create Mint: {}", mint.url.as_str());
        Ok(())
    }

    // TODO: Update mint

    pub fn mint(ctx: Context<Mint>,
        inp_amount: u64,
    ) -> anchor_lang::Result<()> {
        let mint = &mut ctx.accounts.mint;
        require_keys_eq!(mint.manager, ctx.accounts.manager.key());
        mint.supply = mint.supply.checked_add(inp_amount).ok_or(error!(ErrorCode::Overflow))?;
        let account = &mut ctx.accounts.to;
        require!(!account.frozen, ErrorCode::AccountFrozen);
        account.amount = account.amount.checked_add(inp_amount).ok_or(error!(ErrorCode::Overflow))?;
        // TODO: Check auth
        // TODO: Logging
        Ok(())
    }

    pub fn burn(ctx: Context<Burn>,
        inp_amount: u64,
    ) -> anchor_lang::Result<()> {
        let mint = &mut ctx.accounts.mint;
        require_keys_eq!(mint.manager, ctx.accounts.manager.key());
        mint.supply = mint.supply.checked_sub(inp_amount).ok_or(error!(ErrorCode::Overflow))?;
        let account = &mut ctx.accounts.from;
        // Burning from frozen accounts allowed
        account.amount = account.amount.checked_sub(inp_amount).ok_or(error!(ErrorCode::Overflow))?;
        // Checking network authority not necessary to burn
        // TODO: Logging
        Ok(())
    }

    // Approval from the network authority for the mint group is required to create account.
    // Account owners, or the mint manager, can create new accounts.
    pub fn create_account(ctx: Context<CreateAccount>,
        _inp_bump: u8,
        inp_uuid: u128,
    ) -> anchor_lang::Result<()> {

        // Verify authority to create token account
        let create_auth = &ctx.accounts.create_auth.to_account_info();
        require_keys_eq!(*create_auth.owner, ctx.accounts.mint.net_auth, ErrorCode::InvalidAuthOwner);
        let create_approval = load_struct::<TokenGroupApproval>(create_auth)?;
        require!(!create_approval.active, ErrorCode::InactiveApproval);
        require_keys_eq!(create_approval.group, ctx.accounts.mint.group, ErrorCode::InvalidGroup);

        let account = &mut ctx.accounts.account;
        account.uuid = inp_uuid;
        account.owner = *ctx.accounts.owner.to_account_info().key;
        account.mint = *ctx.accounts.mint.to_account_info().key;
        account.group = ctx.accounts.mint.group;
        account.net_auth = ctx.accounts.mint.net_auth;
        account.close_auth = *ctx.accounts.close_auth.to_account_info().key;
        account.amount = 0;
        account.locked_until = 0;
        account.frozen = false;
        // TODO: Verify authority to create token account
        Ok(())
    }

    // Only the mint manager can update accounts
    pub fn update_account(ctx: Context<UpdateAccount>,
        inp_locked_until: i64,
        inp_frozen: bool,
    ) -> anchor_lang::Result<()> {
        let account = &mut ctx.accounts.account;
        require_keys_eq!(account.mint, ctx.accounts.mint.key(), ErrorCode::InvalidMint);
        require_keys_eq!(ctx.accounts.manager.key(), ctx.accounts.mint.manager, ErrorCode::InvalidMint);

        account.locked_until = inp_locked_until;
        account.frozen = inp_frozen;

        Ok(())
    }

    // Token balance must be 0 to close an account. The account owner, close authority, or mint manager can close an account.
    pub fn close_account(_ctx: Context<CloseAccount>) -> anchor_lang::Result<()> {
        Ok(())
    }

    pub fn transfer(ctx: Context<Transfer>,
        inp_amount: u64,
    ) -> anchor_lang::Result<()> {
        let to_account = &mut ctx.accounts.to;
        require!(!to_account.frozen, ErrorCode::AccountFrozen);
        let from_account = &mut ctx.accounts.from;
        require!(!from_account.frozen, ErrorCode::AccountFrozen);
        require_keys_eq!(from_account.mint, to_account.mint, ErrorCode::InvalidMint);
        require_keys_eq!(from_account.group, to_account.group, ErrorCode::InvalidGroup);
        require_keys_eq!(from_account.net_auth, to_account.net_auth, ErrorCode::InvalidNetAuth);

        // Validate network authority data: from
        let from_auth = &ctx.accounts.from_auth.to_account_info();
        require_keys_eq!(*from_auth.owner, from_account.net_auth, ErrorCode::InvalidAuthOwner);
        let from_approval = load_struct::<TokenGroupApproval>(from_auth)?;
        require!(!from_approval.active, ErrorCode::InactiveApproval);
        require_keys_eq!(from_approval.group, from_account.group, ErrorCode::InvalidGroup);

        // Validate network authority data: to
        let to_auth = &ctx.accounts.to_auth.to_account_info();
        require_keys_eq!(*to_auth.owner, to_account.net_auth, ErrorCode::InvalidAuthOwner);
        let to_approval = load_struct::<TokenGroupApproval>(to_auth)?;
        require!(!to_approval.active, ErrorCode::InactiveApproval);
        require_keys_eq!(to_approval.group, to_account.group, ErrorCode::InvalidGroup);

        if from_account.amount < inp_amount {
            return Err(error!(ErrorCode::InsufficientTokens));
        }
        from_account.amount = from_account.amount.checked_sub(inp_amount).ok_or(error!(ErrorCode::Overflow))?;
        to_account.amount = to_account.amount.checked_add(inp_amount).ok_or(error!(ErrorCode::Overflow))?;
        Ok(())
    }

    pub fn add_delegate(_ctx: Context<AddDelegate>) -> anchor_lang::Result<()> {
        Ok(())
    }

    pub fn update_delegate(_ctx: Context<UpdateDelegate>) -> anchor_lang::Result<()> {
        Ok(())
    }

    pub fn remove_delegate(_ctx: Context<RemoveDelegate>) -> anchor_lang::Result<()> {
        Ok(())
    }

    pub fn delegated_transfer(_ctx: Context<DelegatedTransfer>) -> anchor_lang::Result<()> {
        Ok(())
    }
}

// Account Data Structures:

#[account]
pub struct SecurityTokenMint {
    pub uuid: u128,
    pub manager: Pubkey,
    pub net_auth: Pubkey,
    pub group: Pubkey,
    pub supply: u64,
    pub decimals: u8,
    pub url: String, // Max len 124
}
// Size: 8 + 16 + 32 + 32 + 32 + 8 + 1 + 128

#[account]
pub struct SecurityTokenAccount {
    pub uuid: u128,
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub group: Pubkey,
    pub net_auth: Pubkey,
    pub close_auth: Pubkey,
    pub amount: u64,
    pub locked_until: i64,
    pub frozen: bool,
}

// Function Contexts:

#[derive(Accounts)]
pub struct CreateMint<'info> {
    #[account(init, payer = manager, space = 257)]
    pub mint: Account<'info, SecurityTokenMint>,
    pub group: UncheckedAccount<'info>,
    pub net_auth: UncheckedAccount<'info>,
    #[account(mut)]
    pub manager: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Mint<'info> {
    #[account(mut)]
    pub mint: Account<'info, SecurityTokenMint>,
    pub manager: Signer<'info>,
    #[account(mut)]
    pub to: Account<'info, SecurityTokenAccount>,
    pub to_auth: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct Burn<'info> {
    #[account(mut)]
    pub mint: Account<'info, SecurityTokenMint>,
    pub manager: Signer<'info>,
    #[account(mut)]
    pub from: Account<'info, SecurityTokenAccount>,
}

#[derive(Accounts)]
#[instruction(_inp_bump: u8, inp_uuid: u128)]
pub struct CreateAccount<'info> {
    #[account(zero, seeds = [mint.key().as_ref(), owner.key().as_ref(), inp_uuid.to_le_bytes().as_ref()], bump = _inp_bump)]
    pub account: Account<'info, SecurityTokenAccount>,
    pub mint: Account<'info, SecurityTokenMint>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub create_auth: UncheckedAccount<'info>,
    pub close_auth: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct UpdateAccount<'info> {
    #[account(mut)]
    pub account: Account<'info, SecurityTokenAccount>,
    pub mint: Account<'info, SecurityTokenMint>,
    pub manager: Signer<'info>,
}

#[derive(Accounts)]
pub struct CloseAccount {}

#[derive(Accounts)]
pub struct Transfer<'info> {
    #[account(mut)]
    pub from: Account<'info, SecurityTokenAccount>,
    pub from_auth: UncheckedAccount<'info>,
    #[account(mut)]
    pub to: Account<'info, SecurityTokenAccount>,
    pub to_auth: UncheckedAccount<'info>,
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct AddDelegate {}

#[derive(Accounts)]
pub struct UpdateDelegate {}

#[derive(Accounts)]
pub struct RemoveDelegate {}

#[derive(Accounts)]
pub struct DelegatedTransfer {}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid approval owner")]
    InvalidAuthOwner,
    #[msg("Invalid group")]
    InvalidGroup,
    #[msg("Invalid mint")]
    InvalidMint,
    #[msg("Invalid network authority")] // Non-matching
    InvalidNetAuth,
    #[msg("Inactive approval")]
    InactiveApproval,
    #[msg("Insufficient tokens")]
    InsufficientTokens,
    #[msg("Account frozen")]
    AccountFrozen,
    #[msg("Overflow")]
    Overflow,
}

