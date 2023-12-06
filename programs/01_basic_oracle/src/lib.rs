#![allow(clippy::result_large_err)]
// Program: Solana Basic Oracle
// This Solana program will allow you to peridoically relay information from Binance to your
// program and store in an account. When a user interacts with our program they will reference
// the price from the previous push.
// - initialize:        Initializes the program and creates the accounts.
// - set_function:      Sets the Switchboard Function for our program. This is the only function
//                      allowed to push data to our program.
// - update_price:      This is the instruction our Switchboard Function will emit to update
//                      our oracle prices.
// - trigger_function:  Our Switchboard Function will be configured to push data on a pre-defined
//                      schedule. This instruction will allow us to manually request a new price
//                      from the off-chain oracles.

pub use switchboard_solana::prelude::*;

declare_id!("6cAUwwbUEYS5g3HBFc9UUMU63xsSU22KqQ3NKyhKfwJV");

pub const PROGRAM_SEED: &[u8] = b"BASICORACLE";

pub const ORACLE_SEED: &[u8] = b"ORACLE_V1_SEED";

#[account(zero_copy(unsafe))]
pub struct MyProgramState {
    pub bump: u8,
    pub authority: Pubkey,
    pub switchboard_function: Pubkey,
    pub btc_price: f64,
}

#[program]
pub mod basic_oracle {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, bump: u8) -> anchor_lang::Result<()> {
        let program = &mut ctx.accounts.program.load_init()?;
        program.bump =  bump;
        program.authority = ctx.accounts.authority.key();

        // Optionally set the switchboard_function if provided
        if let Some(switchboard_function) = ctx.accounts.switchboard_function.as_ref() {
            program.switchboard_function = switchboard_function.key();
        }

        Ok(())
    }

    pub fn update_price(ctx: Context<UpdatePrice>, price: f64) -> anchor_lang::Result<()> {
        let program = &mut ctx.accounts.program.load_mut()?;

        msg!("saving oracle data");
        program.btc_price = price;

        Ok(())
    }

    pub fn set_function(ctx: Context<SetFunction>) -> anchor_lang::Result<()> {
        let program = &mut ctx.accounts.program.load_init()?;
        program.switchboard_function = ctx.accounts.switchboard_function.key();

        Ok(())
    }

    pub fn trigger_function(ctx: Context<TriggerFunction>) -> anchor_lang::Result<()> {
        FunctionTrigger {
            function: ctx.accounts.switchboard_function.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
            attestation_queue: ctx.accounts.attestation_queue.to_account_info(),
        }
        .invoke(ctx.accounts.attestation_program.clone())?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        space = 8 + std::mem::size_of::<MyProgramState>(),
        payer = payer,
        seeds = [PROGRAM_SEED],
        bump
    )]
    pub program: AccountLoader<'info, MyProgramState>,

    pub authority: Signer<'info>,

    pub switchboard_function: Option<AccountLoader<'info, FunctionAccountData>>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(params: f64)] // rpc parameters hint
pub struct UpdatePrice<'info> {
    // We need this to validate that the Switchboard Function passed to our program
    // is the expected one.
    #[account(mut, 
        seeds = [PROGRAM_SEED],
        bump = program.load()?.bump,
        has_one = switchboard_function
    )]
    pub program: AccountLoader<'info, MyProgramState>,

    // We use this to verify the functions enclave state was verified successfully
   #[account(
        constraint =
                switchboard_function.load()?.validate(
                &enclave_signer.to_account_info()
            )? @ BasicOracleError::FunctionValidationFailed
    )] 
    pub switchboard_function: AccountLoader<'info, FunctionAccountData>,
    pub enclave_signer: Signer<'info>,
}

#[derive(Accounts)]
pub struct SetFunction<'info> {
    #[account(
        mut,
        seeds = [PROGRAM_SEED],
        bump = program.load()?.bump,
        has_one = authority
    )]
    pub program: AccountLoader<'info, MyProgramState>,
    pub authority: Signer<'info>,

    pub switchboard_function: AccountLoader<'info, FunctionAccountData>,
}

#[derive(Accounts)]
pub struct TriggerFunction<'info> {
    // We need this to validate that the Switchboard Function passed to our program
    // is the expected one.
    #[account(
        seeds = [PROGRAM_SEED],
        bump = program.load()?.bump,
        has_one = switchboard_function
    )]
    pub program: AccountLoader<'info, MyProgramState>,

    #[account(
        has_one = authority,
        has_one = attestation_queue,
        owner = attestation_program.key()
    )]
    pub switchboard_function: AccountLoader<'info, FunctionAccountData>,
    pub authority: Signer<'info>,

    pub attestation_queue: AccountLoader<'info, AttestationQueueAccountData>,

    /// CHECK: address is explicit
    #[account(address = SWITCHBOARD_ATTESTATION_PROGRAM_ID)]
    pub attestation_program: AccountInfo<'info>,
}

#[error_code]
#[derive(Eq, PartialEq)]
pub enum BasicOracleError {
    #[msg("Invalid authority account")]
    InvalidAuthority,
    #[msg("Array overflow")]
    ArrayOverflow,
    #[msg("Stale data")]
    StaleData,
    #[msg("Invalid trusted signer")]
    InvalidTrustedSigner,
    #[msg("Invalid MRENCLAVE")]
    InvalidMrEnclave,
    #[msg("Failed to find a valid trading symbol for this price")]
    InvalidSymbol,
    #[msg("FunctionAccount pubkey did not match program_state.function")]
    IncorrectSwitchboardFunction,
    #[msg("FunctionAccount pubkey did not match program_state.function")]
    InvalidSwitchboardFunction,
    #[msg("FunctionAccount was not validated successfully")]
    FunctionValidationFailed,
}
