#![allow(clippy::result_large_err)]
// Program: Solana TWAP Oracle
// This Solana program will allow you to peridoically relay information from EtherPrices to your
// program and store in an account. When a user interacts with our program they will reference
// the price from the previous push.
// - initialize:        Initializes the program and creates the accounts.
// - set_function:      Sets the Switchboard Function for our program. This is the only function
//                      allowed to push data to our program.
// - refresh_oracle:    This is the instruction our Switchboard Function will emit to update
//                      our oracle prices.
// - trigger_function:  Our Switchboard Function will be configured to push data on a pre-defined
//                      schedule. This instruction will allow us to manually request a new price
//                      from the off-chain oracles.

pub use switchboard_solana::prelude::*;

pub mod models;
pub use models::*;



declare_id!("2LuPhyrumCFRXjeDuYp1bLNYp7EbzUraZcvrzN9ZBUkN");

pub const PROGRAM_SEED: &[u8] = b"USDY_USDC_ORACLE_V2";

pub const ORACLE_SEED: &[u8] = b"ORACLE_USDY_SEED_V2";

#[program]
pub mod usdy_usd_oracle {

    use super::*;

    pub fn initialize(ctx: Context<Initialize>, bump: u8, bump2: u8) -> anchor_lang::Result<()> {
        let program = &mut ctx.accounts.program.load_init()?;
        program.bump = bump;
        program.authority = ctx.accounts.authority.key();

        // Optionally set the switchboard_function if provided
        program.switchboard_function = ctx.accounts.switchboard_function.key();

        let oracle = &mut ctx.accounts.oracle.load_init()?;
        oracle.bump = bump2;

        let ondo_price_feed = &mut ctx.accounts.ondo_price_feed.load_init()?;
        ondo_price_feed.authority = ctx.accounts.authority.key();

        let ondo_traded_feed = &mut ctx.accounts.ondo_traded_feed.load_init()?;
        ondo_traded_feed.authority = ctx.accounts.authority.key();



        Ok(())
    }


    pub fn update(ctx: Context<Initialize>, bump: u8, bump2: u8) -> anchor_lang::Result<()> {
        let program = &mut ctx.accounts.program.load_mut()?;
        program.bump = bump;
        program.authority = ctx.accounts.authority.key();

        // Optionally set the switchboard_function if provided
        program.switchboard_function = ctx.accounts.switchboard_function.key();
        let ondo_price_feed = &mut ctx.accounts.ondo_price_feed.load_mut()?;
        ondo_price_feed.authority = ctx.accounts.authority.key();
        let ondo_traded_feed = &mut ctx.accounts.ondo_traded_feed.load_mut()?;
        ondo_traded_feed.authority = ctx.accounts.authority.key();
        let oracle = &mut ctx.accounts.oracle.load_mut()?;
        oracle.bump = bump2;
        
        Ok(())
    }
    pub fn refresh_oracles(
        ctx: Context<RefreshOracles>,
        params: RefreshOraclesParams,
    ) -> anchor_lang::Result<()> {
        let oracle = &mut ctx.accounts.oracle.load_mut()?;
        msg!("saving oracle data");
        oracle.save_rows(&params.rows)?;
        msg!("${}", {oracle.usdy_usd.ondo_price});
        msg!("${}", {oracle.usdy_usd.traded_price});
        let ondo_price_feed = &mut ctx.accounts.ondo_price_feed.load_mut()?;
        
        let mut result = models::AggregatorRound::default();
        result.num_success = 1;
        result.num_error = 0;
        result.result = models::SwitchboardDecimal::from_f64(oracle.usdy_usd.ondo_price as f64);
        result.result.scale = 9;
        result.round_open_timestamp = Clock::get()?.unix_timestamp;
        result.round_open_slot = Clock::get()?.slot;
        ondo_price_feed.latest_confirmed_round = result;
        
        let ondo_traded_feed = &mut ctx.accounts.ondo_traded_feed.load_mut()?;

        let mut result = models::AggregatorRound::default();
        result.num_success = 1;
        result.num_error = 0;
        result.result = models::SwitchboardDecimal::from_f64(oracle.usdy_usd.traded_price as f64);
        result.result.scale = 9;
        result.round_open_timestamp =  Clock::get()?.unix_timestamp;
        result.round_open_slot = Clock::get()?.slot;

        ondo_traded_feed.latest_confirmed_round = result;

        // save the price


        
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
        init_if_needed,
        space = 8 + std::mem::size_of::<MyProgramState>(),
        payer = payer,
        seeds = [PROGRAM_SEED],
        bump
    )]
    pub program: AccountLoader<'info, MyProgramState>,

    #[account(
        init_if_needed,
        space = 8 + std::mem::size_of::<MyOracleState>(),
        payer = payer,
        seeds = [ORACLE_SEED],
        bump
    )]
    pub oracle: AccountLoader<'info, MyOracleState>,

    pub authority: Signer<'info>,

    pub switchboard_function: AccountLoader<'info, FunctionAccountData>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
    
    #[account(init_if_needed, 
        seeds = [ORACLE_SEED, switchboard_function.key().as_ref(),  b"ondo_price_feed"],
        bump,
        payer = payer,
        space = 8 + std::mem::size_of::<AggregatorAccountData>(),
    )]
    pub ondo_price_feed: AccountLoader<'info, models::AggregatorAccountData>,
    
    #[account(init_if_needed, 
        seeds = [ORACLE_SEED, switchboard_function.key().as_ref(),  b"ondo_traded_feed"],
        bump,
        payer = payer,
        space = 8 + std::mem::size_of::<AggregatorAccountData>(),
    )]

    pub ondo_traded_feed: AccountLoader<'info, models::AggregatorAccountData>
}

#[derive(Accounts)]
#[instruction(params: RefreshOraclesParams)] // rpc parameters hint
pub struct RefreshOracles<'info> {
    // We need this to validate that the Switchboard Function passed to our program
    // is the expected one.
    #[account(
        seeds = [PROGRAM_SEED],
        bump = program.load()?.bump,
       //has_one = switchboard_function
    )]
    pub program: AccountLoader<'info, MyProgramState>,

    #[account(
        mut,
        seeds = [ORACLE_SEED],
        bump = oracle.load()?.bump
    )]
    pub oracle: AccountLoader<'info, MyOracleState>,

    // We use this to verify the functions enclave state was verified successfully
   #[account(
    constraint =
                switchboard_function.load()?.validate(
                &enclave_signer.to_account_info()
            )? @ USDY_USDC_ORACLEError::FunctionValidationFailed     
    )]
    pub switchboard_function: AccountLoader<'info, FunctionAccountData>,
    pub enclave_signer: Signer<'info>,
    #[account(mut, 
        constraint = ondo_price_feed.load()?.authority == program.load()?.authority,

        seeds = [ORACLE_SEED, program.load()?.switchboard_function.as_ref(),  b"ondo_price_feed"],
        bump
    )]
    pub ondo_price_feed: AccountLoader<'info, models::AggregatorAccountData>,
    #[account(mut,        
        constraint = ondo_price_feed.load()?.authority == program.load()?.authority,

        seeds = [ORACLE_SEED, program.load()?.switchboard_function.as_ref(), b"ondo_traded_feed"],
        bump
    )]
    pub ondo_traded_feed: AccountLoader<'info, models::AggregatorAccountData>
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct RefreshOraclesParams {
    pub rows: Vec<OracleDataWithTradingSymbol>,
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

    #[account(mut,
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
pub enum USDY_USDC_ORACLEError {
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
