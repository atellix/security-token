use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[repr(u8)]
#[derive(PartialEq, Debug, Eq, Copy, Clone, TryFromPrimitive)]
pub enum TokenState {
    Uninitialized,
    Active,
    Frozen,
}


#[program]
pub mod security_token {
    use super::*;
    pub fn create_mint(ctx: Context<CreateMint>) -> ProgramResult {
        Ok(())
    }

    pub fn mint(ctx: Context<Mint>) -> ProgramResult {
        Ok(())
    }

    pub fn burn(ctx: Context<Burn>) -> ProgramResult {
        Ok(())
    }

    pub fn create_account(ctx: Context<CreateAccount>) -> ProgramResult {
        Ok(())
    }

    pub fn update_account(ctx: Context<UpdateAccount>) -> ProgramResult {
        Ok(())
    }

    pub fn close_account(ctx: Context<CloseAccount>) -> ProgramResult {
        Ok(())
    }

    pub fn transfer(ctx: Context<Transfer>) -> ProgramResult {
        Ok(())
    }

    pub fn create_allowance(ctx: Context<CreateAllowance>) -> ProgramResult {
        Ok(())
    }

    pub fn update_allowance(ctx: Context<UpdateAllowance>) -> ProgramResult {
        Ok(())
    }

    pub fn close_allowance(ctx: Context<CloseAllowance>) -> ProgramResult {
        Ok(())
    }

    pub fn delegated_transfer(ctx: Context<DelegatedTransfer>) -> ProgramResult {
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
    pub url: String, // Max len 128
}

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
    //pub state: TokenState,
}

// Function Contexts:

#[derive(Accounts)]
pub struct CreateMint {}

#[derive(Accounts)]
pub struct Mint {}

#[derive(Accounts)]
pub struct Burn {}

#[derive(Accounts)]
pub struct CreateAccount {}

#[derive(Accounts)]
pub struct UpdateAccount {}

#[derive(Accounts)]
pub struct CloseAccount {}

#[derive(Accounts)]
pub struct Transfer {}

#[derive(Accounts)]
pub struct CreateAllowance {}

#[derive(Accounts)]
pub struct UpdateAllowance {}

#[derive(Accounts)]
pub struct CloseAllowance {}

#[derive(Accounts)]
pub struct DelegatedTransfer {}

