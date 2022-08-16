use std::{ result::Result as FnResult, convert::TryFrom };
use solana_program::{ account_info::AccountInfo };
use anchor_lang::prelude::*;

use net_authority::{ TokenApproval, TokenApprovalStatus };

declare_id!("8JxtmFxuhmgoEFmBeZAqBVouj6DDQBwybpJnpqcYUU8M");

#[inline]
fn load_struct<T: AccountDeserialize>(acc: &AccountInfo) -> FnResult<T, ProgramError> {
    let mut data: &[u8] = &acc.try_borrow_data()?;
    Ok(T::try_deserialize(&mut data)?)
}

#[inline]
fn verify_approval(approval: &TokenApproval, owner: &Pubkey, mint: &Pubkey, group: &Pubkey) -> anchor_lang::Result<()> {
    require_keys_eq!(approval.owner, *owner, ErrorCode::AccessDenied);
    let status = TokenApprovalStatus::try_from(approval.status).unwrap();
    if status == TokenApprovalStatus::PerGroup {
        require_keys_eq!(approval.context, *group, ErrorCode::InvalidGroup);
    } else if status == TokenApprovalStatus::PerMint {
        require_keys_eq!(approval.context, *mint, ErrorCode::InvalidMint);
    } else {
        return Err(error!(ErrorCode::AccessDenied));
    }
    Ok(())
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

        let clock = Clock::get()?;
        msg!("atellix-log");
        emit!(CreateMintEvent {
            event_hash: 242845757374072834824834923033958114258, // solana/program/security-token/create_mint
            slot: clock.slot,
            mint: mint.key(),
            manager: *ctx.accounts.manager.to_account_info().key,
            net_auth: *ctx.accounts.net_auth.to_account_info().key,
            group: *ctx.accounts.group.to_account_info().key,
        });

        Ok(())
    }

    pub fn mint(ctx: Context<Mint>,
        inp_amount: u64,
    ) -> anchor_lang::Result<()> {
        let mint = &mut ctx.accounts.mint;
        let account = &mut ctx.accounts.to;

        // Check auth
        require_keys_eq!(mint.manager, ctx.accounts.manager.key());
        let auth = &ctx.accounts.to_auth.to_account_info();
        require_keys_eq!(*auth.owner, mint.net_auth, ErrorCode::InvalidAuthOwner);
        let approval = load_struct::<TokenApproval>(auth)?;
        verify_approval(&approval, &account.owner, &mint.key(), &mint.group)?;

        require!(!account.frozen, ErrorCode::AccountFrozen);
        mint.supply = mint.supply.checked_add(inp_amount).ok_or(error!(ErrorCode::Overflow))?;
        account.amount = account.amount.checked_add(inp_amount).ok_or(error!(ErrorCode::Overflow))?;

        // Increment action count
        account.action_count = account.action_count.checked_add(1).ok_or(error!(ErrorCode::Overflow))?;

        // Logging
        let clock = Clock::get()?;
        msg!("atellix-log");
        emit!(MintEvent {
            event_hash: 280177221030518891079652254311266574141, // solana/program/security-token/mint
            slot: clock.slot,
            mint: mint.key(),
            manager: *ctx.accounts.manager.to_account_info().key,
            account: account.key(),
            owner: account.owner,
            amount: inp_amount,
            new_supply: mint.supply,
            new_balance: account.amount,
            action_id: account.action_count,
        });

        Ok(())
    }

    pub fn burn(ctx: Context<Burn>,
        inp_amount: u64,
    ) -> anchor_lang::Result<()> {
        let mint = &mut ctx.accounts.mint;
        require_keys_eq!(mint.manager, ctx.accounts.manager.key());

        // Checking network authority not necessary to burn
        mint.supply = mint.supply.checked_sub(inp_amount).ok_or(error!(ErrorCode::Overflow))?;
        let account = &mut ctx.accounts.from;

        // Burning from frozen accounts allowed
        account.amount = account.amount.checked_sub(inp_amount).ok_or(error!(ErrorCode::Overflow))?;

        // Increment action count
        account.action_count = account.action_count.checked_add(1).ok_or(error!(ErrorCode::Overflow))?;

        // Logging
        let clock = Clock::get()?;
        msg!("atellix-log");
        emit!(BurnEvent {
            event_hash: 82379720673132384763647910321069443842, // solana/program/security-token/burn
            slot: clock.slot,
            mint: mint.key(),
            manager: *ctx.accounts.manager.to_account_info().key,
            account: account.key(),
            owner: account.owner,
            amount: inp_amount,
            new_supply: mint.supply,
            new_balance: account.amount,
            action_id: account.action_count,
        });

        Ok(())
    }

    // Approval from the network authority for the mint group is required to create account.
    // Account owners, or the mint manager, can create new accounts.
    pub fn create_account(ctx: Context<CreateAccount>,
        inp_uuid: u128,
    ) -> anchor_lang::Result<()> {

        // Verify authority to create token account
        let auth = &ctx.accounts.create_auth.to_account_info();
        require_keys_eq!(*auth.owner, ctx.accounts.mint.net_auth, ErrorCode::InvalidAuthOwner);
        let approval = load_struct::<TokenApproval>(auth)?;
        verify_approval(&approval, &ctx.accounts.owner.key(), &ctx.accounts.mint.key(), &ctx.accounts.mint.group)?;

        let account = &mut ctx.accounts.account;
        account.uuid = inp_uuid;
        account.owner = *ctx.accounts.owner.to_account_info().key;
        account.mint = *ctx.accounts.mint.to_account_info().key;
        account.group = ctx.accounts.mint.group;
        account.net_auth = ctx.accounts.mint.net_auth;
        account.close_auth = *ctx.accounts.close_auth.to_account_info().key;
        account.amount = 0;
        account.locked_until = 0;
        account.action_count = 0;
        account.frozen = false;

        // Logging
        let clock = Clock::get()?;
        msg!("atellix-log");
        emit!(AccountEvent {
            event_hash: 283176560959297984528699322955846433642, // solana/program/security-token/create_account
            slot: clock.slot,
            mint: account.mint,
            owner: account.owner,
            account: account.key(),
            manager: Pubkey::default(),
            locked_until: 0,
            frozen: false,
            is_manager: false,
            is_update: false,
        });

        Ok(())
    }

    // Token balance must be 0 to close an account. The account owner, close authority, or mint manager can close an account.
    pub fn close_account(ctx: Context<CloseAccount>) -> anchor_lang::Result<()> {
        let account = &ctx.accounts.account;
        require!(account.amount == 0, ErrorCode::NonZeroAccountBalance);
        require_keys_eq!(account.mint, ctx.accounts.mint.key(), ErrorCode::InvalidMint);
        let user_key = ctx.accounts.user.key();
        let close_auth = account.close_auth;
        let manager_key = ctx.accounts.mint.manager;
        if !(user_key == close_auth || user_key == manager_key || user_key == account.owner) {
            return Err(error!(ErrorCode::AccessDenied));
        }

        let clock = Clock::get()?;
        msg!("atellix-log");
        emit!(CloseEvent {
            event_hash: 291921020161207202589209186455769654935, // solana/program/security-token/close_account
            slot: clock.slot,
            user: user_key,
            owner: account.owner,
            account: account.key(),
            allowance: Pubkey::default(),
            is_allowance: false,
        });

        Ok(())
    }

    pub fn transfer(ctx: Context<Transfer>,
        inp_amount: u64,
    ) -> anchor_lang::Result<()> {
        let to_account = &mut ctx.accounts.to;
        let from_account = &mut ctx.accounts.from;

        require!(!to_account.frozen, ErrorCode::AccountFrozen);
        require!(!from_account.frozen, ErrorCode::AccountFrozen);
        require_keys_eq!(from_account.owner, ctx.accounts.user.key(), ErrorCode::AccessDenied);
        require_keys_eq!(from_account.mint, to_account.mint, ErrorCode::InvalidMint);
        require_keys_eq!(from_account.group, to_account.group, ErrorCode::InvalidGroup);
        require_keys_eq!(from_account.net_auth, to_account.net_auth, ErrorCode::InvalidNetAuth);

        // Validate network authority data: from
        let from_auth = &ctx.accounts.from_auth.to_account_info();
        require_keys_eq!(*from_auth.owner, from_account.net_auth, ErrorCode::InvalidAuthOwner);
        let from_approval = load_struct::<TokenApproval>(from_auth)?;
        verify_approval(&from_approval, &from_account.owner, &from_account.mint.key(), &from_account.group)?;

        // Validate network authority data: to
        let to_auth = &ctx.accounts.to_auth.to_account_info();
        require_keys_eq!(*to_auth.owner, to_account.net_auth, ErrorCode::InvalidAuthOwner);
        let to_approval = load_struct::<TokenApproval>(to_auth)?;
        verify_approval(&to_approval, &to_account.owner, &to_account.mint.key(), &to_account.group)?;

        // TODO: Check timelock

        if from_account.amount < inp_amount {
            return Err(error!(ErrorCode::InsufficientTokens));
        }
        from_account.amount = from_account.amount.checked_sub(inp_amount).ok_or(error!(ErrorCode::Overflow))?;
        to_account.amount = to_account.amount.checked_add(inp_amount).ok_or(error!(ErrorCode::Overflow))?;

        // Increment action count
        from_account.action_count = from_account.action_count.checked_add(1).ok_or(error!(ErrorCode::Overflow))?;
        to_account.action_count = to_account.action_count.checked_add(1).ok_or(error!(ErrorCode::Overflow))?;

        let clock = Clock::get()?;
        msg!("atellix-log");
        emit!(TransferEvent {
            event_hash: 148603774903118292415252491548477030903, // solana/program/security-token/transfer
            slot: clock.slot,
            from_account: from_account.key(),
            to_account: to_account.key(),
            from_owner: from_account.owner,
            to_owner: to_account.owner,
            user: ctx.accounts.user.key(),
            mint: from_account.mint,
            group: from_account.group,
            amount: inp_amount,
            new_from_balance: from_account.amount,
            new_to_balance: to_account.amount,
            from_action_id: from_account.action_count,
            to_action_id: to_account.action_count,
            is_delegate: false,
        });

        Ok(())
    }

    pub fn manager_create_account(ctx: Context<ManagerCreateAccount>,
        inp_uuid: u128,
    ) -> anchor_lang::Result<()> {
        let account = &mut ctx.accounts.account;
        require_keys_eq!(account.mint, ctx.accounts.mint.key(), ErrorCode::InvalidMint);
        require_keys_eq!(ctx.accounts.manager.key(), ctx.accounts.mint.manager, ErrorCode::AccessDenied);

        // Verify authority to create token account
        let auth = &ctx.accounts.create_auth.to_account_info();
        require_keys_eq!(*auth.owner, ctx.accounts.mint.net_auth, ErrorCode::InvalidAuthOwner);
        let approval = load_struct::<TokenApproval>(auth)?;
        verify_approval(&approval, &ctx.accounts.owner.key(), &ctx.accounts.mint.key(), &ctx.accounts.mint.group)?;

        account.uuid = inp_uuid;
        account.owner = *ctx.accounts.owner.to_account_info().key;
        account.mint = *ctx.accounts.mint.to_account_info().key;
        account.group = ctx.accounts.mint.group;
        account.net_auth = ctx.accounts.mint.net_auth;
        account.close_auth = *ctx.accounts.close_auth.to_account_info().key;
        account.amount = 0;
        account.locked_until = 0;
        account.frozen = false;
        Ok(())
    }

    // Only the mint manager can update accounts
    pub fn manager_update_account(ctx: Context<ManagerUpdateAccount>,
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

    pub fn delegate_approve(ctx: Context<DelegateApprove>,
        inp_allowance_amount: u64,
        inp_all: bool,
    ) -> anchor_lang::Result<()> {

        let account = &mut ctx.accounts.account;
        require_keys_eq!(account.owner, ctx.accounts.owner.key(), ErrorCode::AccessDenied);
        
        let allowance = &mut ctx.accounts.allowance;
        allowance.owner = *ctx.accounts.owner.to_account_info().key;
        allowance.account = *ctx.accounts.account.to_account_info().key;
        allowance.delegate = *ctx.accounts.delegate.to_account_info().key;
        if inp_all {
            allowance.amount = 0;
            allowance.all = true;
        } else {
            allowance.amount = inp_allowance_amount;
            allowance.all = false;
        }
        Ok(())
    }

    pub fn delegate_transfer(ctx: Context<DelegateTransfer>,
        inp_amount: u64,
    ) -> anchor_lang::Result<()> {
        let allowance = &mut ctx.accounts.allowance;
        let to_account = &mut ctx.accounts.to;
        let from_account = &mut ctx.accounts.from;

        // Validate allowance
        require_keys_eq!(from_account.key(), allowance.account, ErrorCode::AccessDenied);
        require_keys_eq!(ctx.accounts.delegate.key(), allowance.delegate, ErrorCode::AccessDenied);
        if !allowance.all {
            require!(allowance.amount >= inp_amount, ErrorCode::InsufficientAllowance);
        }

        require!(!to_account.frozen, ErrorCode::AccountFrozen);
        require!(!from_account.frozen, ErrorCode::AccountFrozen);
        require_keys_eq!(from_account.mint, to_account.mint, ErrorCode::InvalidMint);
        require_keys_eq!(from_account.group, to_account.group, ErrorCode::InvalidGroup);
        require_keys_eq!(from_account.net_auth, to_account.net_auth, ErrorCode::InvalidNetAuth);

        // Validate network authority data: from
        let from_auth = &ctx.accounts.from_auth.to_account_info();
        require_keys_eq!(*from_auth.owner, from_account.net_auth, ErrorCode::InvalidAuthOwner);
        let from_approval = load_struct::<TokenApproval>(from_auth)?;
        verify_approval(&from_approval, &from_account.owner, &from_account.mint.key(), &from_account.group)?;

        // Validate network authority data: to
        let to_auth = &ctx.accounts.to_auth.to_account_info();
        require_keys_eq!(*to_auth.owner, to_account.net_auth, ErrorCode::InvalidAuthOwner);
        let to_approval = load_struct::<TokenApproval>(to_auth)?;
        verify_approval(&to_approval, &to_account.owner, &to_account.mint.key(), &to_account.group)?;

        // TODO: Check timelock

        if from_account.amount < inp_amount {
            return Err(error!(ErrorCode::InsufficientTokens));
        }
        from_account.amount = from_account.amount.checked_sub(inp_amount).ok_or(error!(ErrorCode::Overflow))?;
        to_account.amount = to_account.amount.checked_add(inp_amount).ok_or(error!(ErrorCode::Overflow))?;
        if !allowance.all {
            allowance.amount = allowance.amount.checked_sub(inp_amount).ok_or(error!(ErrorCode::Overflow))?;
        }
        Ok(())
    }

    pub fn delegate_update(ctx: Context<DelegateUpdate>,
        inp_allowance_amount: u64,
        inp_all: bool,
    ) -> anchor_lang::Result<()> {
        let allowance = &mut ctx.accounts.allowance;

        require_keys_eq!(ctx.accounts.owner.key(), allowance.owner, ErrorCode::AccessDenied);

        if inp_all {
            allowance.amount = 0;
            allowance.all = true;
        } else {
            allowance.amount = inp_allowance_amount;
            allowance.all = false;
        }

        Ok(())
    }

    pub fn delegate_close(ctx: Context<DelegateClose>) -> anchor_lang::Result<()> {
        let allowance = &ctx.accounts.allowance;
        require_keys_eq!(allowance.owner, ctx.accounts.owner.key(), ErrorCode::AccessDenied);
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
    pub action_count: u64,
    pub locked_until: i64,
    pub frozen: bool,
}
// Size: 8 + 16 + 32 + 32 + 32 + 32 + 32 + 8 + 8 + 8 + 1 = 209

#[account]
#[derive(Default)]
pub struct DelegateAllowance {
    pub owner: Pubkey,                  // The owner of the allowance (must be same as the owner of the token account)
    pub account: Pubkey,                // The security token account for the allowance
    pub delegate: Pubkey,               // The delegate granted an allowance of tokens to transfer (typically the root PDA of another program)
    pub amount: u64,                    // The amount of tokens for the allowance (same decimals as underlying token)
    pub all: bool,                      // Ignore amount field, delegate all tokens (used to allow other programs to transfer tokens without needing to periodically reset the amount field)
}
// LEN: 8 + 32 + 32 + 32 + 8 + 1 = 113

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
#[instruction(inp_uuid: u128)]
pub struct CreateAccount<'info> {
    #[account(init_if_needed, seeds = [mint.key().as_ref(), owner.key().as_ref(), inp_uuid.to_le_bytes().as_ref()], bump, payer = owner, space = 209)]
    pub account: Account<'info, SecurityTokenAccount>,
    pub mint: Account<'info, SecurityTokenMint>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub create_auth: UncheckedAccount<'info>,
    pub close_auth: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CloseAccount<'info> {
    pub user: Signer<'info>,
    #[account(mut)]
    pub fee_receiver: Signer<'info>,
    #[account(mut, close = fee_receiver)]
    pub account: Account<'info, SecurityTokenAccount>,
    pub mint: Account<'info, SecurityTokenMint>,
}

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
#[instruction(inp_uuid: u128)]
pub struct ManagerCreateAccount<'info> {
    #[account(init_if_needed, seeds = [mint.key().as_ref(), owner.key().as_ref(), inp_uuid.to_le_bytes().as_ref()], bump, payer = manager, space = 209)]
    pub account: Account<'info, SecurityTokenAccount>,
    pub mint: Account<'info, SecurityTokenMint>,
    #[account(mut)]
    pub manager: Signer<'info>,
    pub owner: UncheckedAccount<'info>,
    pub create_auth: UncheckedAccount<'info>,
    pub close_auth: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ManagerUpdateAccount<'info> {
    #[account(mut)]
    pub account: Account<'info, SecurityTokenAccount>,
    pub mint: Account<'info, SecurityTokenMint>,
    pub manager: Signer<'info>,
}

// TODO: Implement fn!
#[derive(Accounts)]
pub struct ManagerTransfer<'info> {
    #[account(mut)]
    pub from: Account<'info, SecurityTokenAccount>,
    #[account(mut)]
    pub to: Account<'info, SecurityTokenAccount>,
    pub mint: Account<'info, SecurityTokenMint>,
    pub manager: Signer<'info>,
}

#[derive(Accounts)]
pub struct DelegateApprove<'info> {
    #[account(init_if_needed, seeds = [account.key().as_ref(), delegate.key().as_ref()], bump, payer = owner, space = 113)]
    pub allowance: Account<'info, DelegateAllowance>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub delegate: UncheckedAccount<'info>,
    #[account(mut)]
    pub account: Account<'info, SecurityTokenAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DelegateTransfer<'info> {
    #[account(mut)]
    pub from: Account<'info, SecurityTokenAccount>,
    pub from_auth: UncheckedAccount<'info>,
    #[account(mut)]
    pub to: Account<'info, SecurityTokenAccount>,
    pub to_auth: UncheckedAccount<'info>,
    pub delegate: Signer<'info>,
    #[account(mut)]
    pub allowance: Account<'info, DelegateAllowance>,
}

#[derive(Accounts)]
pub struct DelegateUpdate<'info> {
    #[account(mut)]
    pub allowance: Account<'info, DelegateAllowance>,
    pub account: Account<'info, SecurityTokenAccount>,
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct DelegateClose<'info> {
    #[account(mut, close = fee_recipient)]
    pub allowance: Account<'info, DelegateAllowance>,
    pub owner: Signer<'info>,
    #[account(mut)]
    pub fee_recipient: UncheckedAccount<'info>,
}

#[event]
pub struct CreateMintEvent {
    pub event_hash: u128,
    pub slot: u64,
    pub mint: Pubkey,
    pub manager: Pubkey,
    pub net_auth: Pubkey,
    pub group: Pubkey,
}

#[event]
pub struct AccountEvent {
    pub event_hash: u128,
    pub slot: u64,
    pub mint: Pubkey,
    pub owner: Pubkey,
    pub account: Pubkey,
    pub manager: Pubkey,
    pub locked_until: i64,
    pub frozen: bool,
    pub is_manager: bool,
    pub is_update: bool,
}

#[event]
pub struct MintEvent {
    pub event_hash: u128,
    pub slot: u64,
    pub mint: Pubkey,
    pub manager: Pubkey,
    pub account: Pubkey,
    pub owner: Pubkey,
    pub amount: u64,
    pub new_supply: u64,
    pub new_balance: u64,
    pub action_id: u64,
}

#[event]
pub struct BurnEvent {
    pub event_hash: u128,
    pub slot: u64,
    pub mint: Pubkey,
    pub manager: Pubkey,
    pub account: Pubkey,
    pub owner: Pubkey,
    pub amount: u64,
    pub new_supply: u64,
    pub new_balance: u64,
    pub action_id: u64,
}

#[event]
pub struct TransferEvent {
    pub event_hash: u128,
    pub slot: u64,
    pub from_account: Pubkey,
    pub to_account: Pubkey,
    pub from_owner: Pubkey,
    pub to_owner: Pubkey,
    pub user: Pubkey,
    pub mint: Pubkey,
    pub group: Pubkey,
    pub amount: u64,
    pub new_from_balance: u64,
    pub new_to_balance: u64,
    pub from_action_id: u64,
    pub to_action_id: u64,
    pub is_delegate: bool,
}

#[event]
pub struct CloseEvent {
    pub event_hash: u128,
    pub slot: u64,
    pub user: Pubkey,
    pub owner: Pubkey,
    pub account: Pubkey,
    pub allowance: Pubkey,
    pub is_allowance: bool,
}

#[event]
pub struct DelegateEvent {
    pub event_hash: u128,
    pub slot: u64,
    pub owner: Pubkey,
    pub account: Pubkey,
    pub allowance: Pubkey,
    pub amount: u64,
    pub all: bool,
    pub is_close: bool,
}

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
    #[msg("Insufficient allowance")]
    InsufficientAllowance,
    #[msg("Non-zero account balance")]
    NonZeroAccountBalance,
    #[msg("Account frozen")]
    AccountFrozen,
    #[msg("Access denied")]
    AccessDenied,
    #[msg("Overflow")]
    Overflow,
}

