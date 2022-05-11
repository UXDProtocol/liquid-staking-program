#![cfg_attr(not(debug_assertions), deny(warnings))]

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount, Token};
use error::CommonError;
use stake_wrapper::StakeWrapper;
use std::{
    convert::{TryFrom, TryInto},
    fmt::Display,
    ops::{Deref, DerefMut},
    str::FromStr,
};
use ticket_account::TicketAccountData;

pub mod calc;
pub mod checks;
pub mod error;
pub mod liq_pool;
pub mod list;
pub mod located;
pub mod stake_system;
pub mod stake_wrapper;
pub mod state;
pub mod ticket_account;
pub mod validator_system;

pub use state::State;

/// The static program ID
pub static ID: Pubkey = Pubkey::new_from_array([
    5, 69, 227, 101, 190, 242, 113, 173, 117, 53, 3, 103, 86, 93, 164, 13, 163, 54, 220, 28, 135,
    155, 177, 84, 138, 122, 252, 197, 90, 169, 57, 30,
]); // "MarBmsSgKXdrN1egZf5sqe1TMai9K1rChYNDJgjq7aD"

/// Confirms that a given pubkey is equivalent to the program ID
pub fn check_id(id: &Pubkey) -> bool {
    id == &ID
}

/// Returns the program ID
pub fn id() -> Pubkey {
    ID
}

#[cfg(test)]
#[test]
fn test_id() {
    assert_eq!(
        ID,
        Pubkey::from_str("MarBmsSgKXdrN1egZf5sqe1TMai9K1rChYNDJgjq7aD").unwrap()
    );
    assert!(check_id(&id()));
}

pub const MAX_REWARD_FEE: u32 = 1_000; //basis points, 10% max reward fee

fn check_context<T>(ctx: &Context<T>) -> Result<()> {
    if !check_id(ctx.program_id) {
        return Err(CommonError::InvalidProgramId.into());
    }
    //make sure there are no extra accounts
    if !ctx.remaining_accounts.is_empty() {
        return Err(CommonError::UnexpectedAccount.into());
    }

    Ok(())
}

//-----------------------------------------------------
#[program]
pub mod marinade_finance {

    use super::*;

    //----------------------------------------------------------------------------
    // Stable Instructions, part of devnet-MVP-1 beta-test at marinade.finance
    //----------------------------------------------------------------------------
    // Includes: initialization, contract parameters
    // basic user functions: (liquid)stake, liquid-unstake
    // liq-pool: add-liquidity, remove-liquidity
    // Validator list management
    //----------------------------------------------------------------------------

    pub fn initialize(ctx: Context<Initialize>, data: InitializeData) -> Result<()> {
        check_context(&ctx)?;
        ctx.accounts.process(data)?;
        Ok(())
    }

    pub fn change_authority(
        ctx: Context<ChangeAuthority>,
        data: ChangeAuthorityData,
    ) -> Result<()> {
        check_context(&ctx)?;
        ctx.accounts.process(data)
    }

    pub fn add_validator(ctx: Context<AddValidator>, score: u32) -> Result<()> {
        check_context(&ctx)?;
        ctx.accounts.process(score)
    }

    pub fn remove_validator(
        ctx: Context<RemoveValidator>,
        index: u32,
        validator_vote: Pubkey,
    ) -> Result<()> {
        check_context(&ctx)?;
        ctx.accounts.process(index, validator_vote)
    }

    pub fn set_validator_score(
        ctx: Context<SetValidatorScore>,
        index: u32,
        validator_vote: Pubkey,
        score: u32,
    ) -> Result<()> {
        check_context(&ctx)?;
        ctx.accounts.process(index, validator_vote, score)
    }

    pub fn config_validator_system(
        ctx: Context<ConfigValidatorSystem>,
        extra_runs: u32,
    ) -> Result<()> {
        check_context(&ctx)?;
        ctx.accounts.process(extra_runs)
    }

    // deposit AKA stake, AKA deposit_sol
    pub fn deposit(ctx: Context<Deposit>, lamports: u64) -> Result<()> {
        check_context(&ctx)?;
        ctx.accounts.process(lamports)
    }

    // SPL stake pool like
    pub fn deposit_stake_account(
        ctx: Context<DepositStakeAccount>,
        validator_index: u32,
    ) -> Result<()> {
        check_context(&ctx)?;
        ctx.accounts.process(validator_index)
    }

    pub fn liquid_unstake(ctx: Context<LiquidUnstake>, msol_amount: u64) -> Result<()> {
        check_context(&ctx)?;
        ctx.accounts.process(msol_amount)
    }

    pub fn add_liquidity(ctx: Context<AddLiquidity>, lamports: u64) -> Result<()> {
        check_context(&ctx)?;
        ctx.accounts.process(lamports)
    }

    pub fn remove_liquidity(ctx: Context<RemoveLiquidity>, tokens: u64) -> Result<()> {
        check_context(&ctx)?;
        ctx.accounts.process(tokens)
    }

    pub fn set_lp_params(
        ctx: Context<SetLpParams>,
        min_fee: Fee,
        max_fee: Fee,
        liquidity_target: u64,
    ) -> Result<()> {
        check_context(&ctx)?;
        ctx.accounts.process(min_fee, max_fee, liquidity_target)
    }

    pub fn config_marinade(
        ctx: Context<ConfigMarinade>,
        params: ConfigMarinadeParams,
    ) -> Result<()> {
        check_context(&ctx)?;
        ctx.accounts.process(params)
    }

    //-------------------------------------------------------------------------------------
    // WIP Instructions, wil be part of devnet-MVP-2 beta-test release at marinade.finance
    //-------------------------------------------------------------------------------------
    // Includes advanced user options: deposit-stake-account, Delayed-Unstake
    // backend/bot "crank" related functions:
    // * order_unstake (starts stake-account deactivation)
    // * withdraw (delete & withdraw from a deactivated stake-account)
    // * update (compute stake-account rewards & update mSOL price)
    //-------------------------------------------------------------------------------------

    pub fn order_unstake(ctx: Context<OrderUnstake>, msol_amount: u64) -> Result<()> {
        check_context(&ctx)?;
        ctx.accounts.process(msol_amount)
    }

    pub fn claim(ctx: Context<Claim>) -> Result<()> {
        check_context(&ctx)?;
        ctx.accounts.process()
    }

    pub fn stake_reserve(ctx: Context<StakeReserve>, validator_index: u32) -> Result<()> {
        check_context(&ctx)?;
        ctx.accounts.process(validator_index)
    }

    pub fn update_active(
        ctx: Context<UpdateActive>,
        stake_index: u32,
        validator_index: u32,
    ) -> Result<()> {
        check_context(&ctx)?;
        ctx.accounts.process(stake_index, validator_index)
    }
    pub fn update_deactivated(ctx: Context<UpdateDeactivated>, stake_index: u32) -> Result<()> {
        check_context(&ctx)?;
        ctx.accounts.process(stake_index)
    }

    pub fn deactivate_stake(
        ctx: Context<DeactivateStake>,
        stake_index: u32,
        validator_index: u32,
    ) -> Result<()> {
        check_context(&ctx)?;
        ctx.accounts.process(stake_index, validator_index)
    }

    pub fn emergency_unstake(
        ctx: Context<EmergencyUnstake>,
        stake_index: u32,
        validator_index: u32,
    ) -> Result<()> {
        check_context(&ctx)?;
        ctx.accounts.process(stake_index, validator_index)
    }

    pub fn merge_stakes(
        ctx: Context<MergeStakes>,
        destination_stake_index: u32,
        source_stake_index: u32,
        validator_index: u32,
    ) -> Result<()> {
        check_context(&ctx)?;
        ctx.accounts
            .process(destination_stake_index, source_stake_index, validator_index)
    }
}

#[cfg(not(feature = "no-entrypoint"))]
pub fn test_entry(program_id: &Pubkey, accounts: &[AccountInfo], ix_data: &[u8]) -> Result<()> {
    entry(program_id, accounts, ix_data).map_err(|e| e.into())
}

//-----------------------------------------------------
#[derive(
    Clone, Copy, Debug, Default, AnchorSerialize, AnchorDeserialize, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct Fee {
    pub basis_points: u32,
}

impl Display for Fee {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}%", self.basis_points as f32 / 100.0)
    }
}

impl Fee {
    pub fn from_basis_points(basis_points: u32) -> Self {
        Self { basis_points }
    }

    /// generic check, capped Fee
    pub fn check_max(&self, max_basis_points: u32) -> Result<()> {
        if self.basis_points > max_basis_points {
            Err(CommonError::FeeTooHigh.into())
        } else {
            Ok(())
        }
    }
    /// base check, Fee <= 100%
    pub fn check(&self) -> Result<()> {
        self.check_max(10_000)
    }

    pub fn apply(&self, lamports: u64) -> u64 {
        // LMT no error possible
        (lamports as u128 * self.basis_points as u128 / 10_000_u128) as u64
    }
}

impl TryFrom<f64> for Fee {
    type Error = anchor_lang::error::Error;

    fn try_from(n: f64) -> Result<Self> {
        let basis_points_i = (n * 100.0).floor() as i64; // 4.5% => 450 basis_points
        let basis_points =
            u32::try_from(basis_points_i).map_err(|_| CommonError::CalculationFailure)?;
        let fee = Fee::from_basis_points(basis_points);
        fee.check()?;
        Ok(fee)
    }
}

impl FromStr for Fee {
    type Err = anchor_lang::error::Error; // TODO: better error

    fn from_str(s: &str) -> Result<Self> {
        f64::try_into(s.parse().map_err(|_| CommonError::CalculationFailure)?)
    }
}
//-----------------------------------------------------
#[derive(Accounts)]
pub struct Initialize<'info> {
    /// CHECK : Marinade CPI
    #[account(signer)]
    pub creator_authority: AccountInfo<'info>,
    #[account(zero, rent_exempt = enforce)]
    pub state: Box<Account<'info, State>>,

    /// CHECK : Marinade CPI
    pub reserve_pda: AccountInfo<'info>,
    /// CHECK : Marinade CPI
    #[account(mut, rent_exempt = enforce)]
    pub stake_list: AccountInfo<'info>,
    /// CHECK : Marinade CPI
    #[account(mut, rent_exempt = enforce)]
    pub validator_list: AccountInfo<'info>,

    pub msol_mint: Account<'info, Mint>,
    /// CHECK : Marinade CPI
    pub operational_sol_account: AccountInfo<'info>,

    pub liq_pool: LiqPoolInitialize<'info>,

    // treasury_sol_account: AccountInfo<'info>,
    treasury_msol_account: Account<'info, TokenAccount>,

    pub clock: Sysvar<'info, Clock>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub struct InitializeData {
    pub admin_authority: Pubkey,
    pub validator_manager_authority: Pubkey,
    pub min_stake: u64,
    pub reward_fee: Fee,

    pub liq_pool: LiqPoolInitializeData,
    pub additional_stake_record_space: u32,
    pub additional_validator_record_space: u32,
    pub slots_for_stake_delta: u64,
}

//-----------------------------------------------------
#[derive(Accounts)]
pub struct LiqPoolInitialize<'info> {
    pub lp_mint: Account<'info, Mint>,
    /// CHECK : Marinade CPI
    pub sol_leg_pda: AccountInfo<'info>,
    pub msol_leg: Account<'info, TokenAccount>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub struct LiqPoolInitializeData {
    pub lp_liquidity_target: u64,
    pub lp_max_fee: Fee,
    pub lp_min_fee: Fee,
    pub lp_treasury_cut: Fee,
}

//-----------------------------------------------------
#[derive(Accounts)]
pub struct ChangeAuthority<'info> {
    #[account(mut)]
    pub state: Account<'info, State>,
    /// CHECK : Marinade CPI
    #[account(signer)]
    pub admin_authority: AccountInfo<'info>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub struct ChangeAuthorityData {
    pub admin: Option<Pubkey>,
    pub validator_manager: Option<Pubkey>,
    pub operational_sol_account: Option<Pubkey>,
    pub treasury_msol_account: Option<Pubkey>,
}

//-----------------------------------------------------
#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    #[account(mut)]
    pub state: Box<Account<'info, State>>,

    #[account(mut)]
    pub lp_mint: Account<'info, Mint>,

    /// CHECK : Marinade CPI
    pub lp_mint_authority: AccountInfo<'info>,

    // msol_mint to be able to compute current msol value in liq_pool
    // not needed because we use memorized value
    // pub msol_mint: CpiAccount<'info, Mint>,
    // liq_pool_msol_leg to be able to compute current msol value in liq_pool
    pub liq_pool_msol_leg: Account<'info, TokenAccount>,

    /// CHECK : Marinade CPI
    #[account(mut)]
    // seeds = [&state.to_account_info().key.to_bytes()[..32], LiqPool::SOL_ACCOUNT_SEED], bump = state.liq_pool.sol_account_bump_seed)]
    // #[account(owner = "11111111111111111111111111111111")]
    pub liq_pool_sol_leg_pda: AccountInfo<'info>,

    /// CHECK : Marinade CPI
    // #[check_owner_program("11111111111111111111111111111111")]
    #[account(mut, signer)] //, owner = "11111111111111111111111111111111")]
    pub transfer_from: AccountInfo<'info>,

    // #[check_owner_program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")]
    #[account(mut)] // , owner = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")]
    pub mint_to: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,
}
//-----------------------------------------------------
#[derive(Accounts)]
pub struct RemoveLiquidity<'info> {
    #[account(mut)]
    pub state: Box<Account<'info, State>>,

    #[account(mut)]
    pub lp_mint: Account<'info, Mint>,

    // pub msol_mint: CpiAccount<'info, Mint>, // not needed anymore
    #[account(mut)]
    pub burn_from: Account<'info, TokenAccount>,
    /// CHECK : Marinade CPI
    #[account(signer)]
    pub burn_from_authority: AccountInfo<'info>,

    /// CHECK : Marinade CPI
    #[account(mut)]
    pub transfer_sol_to: AccountInfo<'info>,

    #[account(mut)]
    pub transfer_msol_to: Account<'info, TokenAccount>,

    // legs
    /// CHECK : Marinade CPI
    #[account(mut)]
    pub liq_pool_sol_leg_pda: AccountInfo<'info>,
    #[account(mut)]
    pub liq_pool_msol_leg: Account<'info, TokenAccount>,
    /// CHECK : Marinade CPI
    pub liq_pool_msol_leg_authority: AccountInfo<'info>,

    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,
}
//-----------------------------------------------------
#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub state: Box<Account<'info, State>>,

    #[account(mut)]
    pub msol_mint: Account<'info, Mint>,

    /// CHECK : Marinade CPI
    #[account(mut)]
    pub liq_pool_sol_leg_pda: AccountInfo<'info>,

    #[account(mut)]
    pub liq_pool_msol_leg: Account<'info, TokenAccount>,
    /// CHECK : Marinade CPI
    pub liq_pool_msol_leg_authority: AccountInfo<'info>,

    /// CHECK : Marinade CPI
    #[account(mut)]
    pub reserve_pda: AccountInfo<'info>,

    /// CHECK : Marinade CPI
    #[account(mut, signer)]
    pub transfer_from: AccountInfo<'info>,

    #[account(mut)]
    pub mint_to: Account<'info, TokenAccount>,

    /// CHECK : Marinade CPI
    pub msol_mint_authority: AccountInfo<'info>,

    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,
}

//-----------------------------------------------------
#[derive(Accounts)]
pub struct DepositStakeAccount<'info> {
    #[account(mut)]
    pub state: Box<Account<'info, State>>,

    /// CHECK : Marinade CPI
    #[account(mut)]
    pub validator_list: AccountInfo<'info>,
    /// CHECK : Marinade CPI
    #[account(mut)]
    pub stake_list: AccountInfo<'info>,

    #[account(mut)]
    pub stake_account: Account<'info, StakeWrapper>,
    /// CHECK : Marinade CPI
    #[account(signer)]
    pub stake_authority: AccountInfo<'info>,
    /// CHECK : Marinade CPI
    #[account(mut)]
    pub duplication_flag: AccountInfo<'info>,
    /// CHECK : Marinade CPI
    #[account(mut, signer)]
    pub rent_payer: AccountInfo<'info>,

    #[account(mut)]
    pub msol_mint: Account<'info, Mint>,
    #[account(mut)]
    pub mint_to: Account<'info, TokenAccount>,

    /// CHECK : Marinade CPI
    pub msol_mint_authority: AccountInfo<'info>,

    pub clock: Sysvar<'info, Clock>,
    pub rent: Sysvar<'info, Rent>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    /// CHECK : Marinade CPI
    pub stake_program: AccountInfo<'info>,
}

//-----------------------------------------------------
#[derive(Accounts)]
pub struct LiquidUnstake<'info> {
    #[account(mut)]
    pub state: Box<Account<'info, State>>,

    #[account(mut)]
    pub msol_mint: Account<'info, Mint>,

    /// CHECK : Marinade CPI
    #[account(mut)]
    pub liq_pool_sol_leg_pda: AccountInfo<'info>,

    #[account(mut)]
    pub liq_pool_msol_leg: Account<'info, TokenAccount>,
    /// CHECK : Marinade CPI
    #[account(mut)]
    pub treasury_msol_account: AccountInfo<'info>,

    #[account(mut)]
    pub get_msol_from: Account<'info, TokenAccount>,
    /// CHECK : Marinade CPI
    #[account(signer)]
    pub get_msol_from_authority: AccountInfo<'info>, //burn_msol_from owner or delegate_authority

    /// CHECK : Marinade CPI
    #[account(mut)]
    pub transfer_sol_to: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}
//-----------------------------------------------------
#[derive(Accounts)]
pub struct AddValidator<'info> {
    #[account(mut)]
    pub state: Account<'info, State>,
    /// CHECK : Marinade CPI
    #[account(signer)]
    pub manager_authority: AccountInfo<'info>,
    /// CHECK : Marinade CPI
    #[account(mut)]
    pub validator_list: AccountInfo<'info>,

    /// CHECK : Marinade CPI
    pub validator_vote: AccountInfo<'info>,

    /// CHECK : Marinade CPI
    #[account(mut)]
    pub duplication_flag: AccountInfo<'info>,
    /// CHECK : Marinade CPI
    #[account(mut, signer)]
    pub rent_payer: AccountInfo<'info>,

    pub clock: Sysvar<'info, Clock>,
    pub rent: Sysvar<'info, Rent>,

    pub system_program: Program<'info, System>,
}

//-----------------------------------------------------
#[derive(Accounts)]
pub struct RemoveValidator<'info> {
    #[account(mut)]
    pub state: Account<'info, State>,
    /// CHECK : Marinade CPI
    #[account(signer)]
    pub manager_authority: AccountInfo<'info>,
    /// CHECK : Marinade CPI
    #[account(mut)]
    pub validator_list: AccountInfo<'info>,

    /// CHECK : Marinade CPI
    #[account(mut)]
    pub duplication_flag: AccountInfo<'info>,
    /// CHECK : Marinade CPI
    #[account(mut)]
    pub operational_sol_account: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct SetValidatorScore<'info> {
    #[account(mut)]
    pub state: Account<'info, State>,
    /// CHECK : Marinade CPI
    #[account(signer)]
    pub manager_authority: AccountInfo<'info>,
    /// CHECK : Marinade CPI
    #[account(mut)]
    pub validator_list: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct ConfigValidatorSystem<'info> {
    #[account(mut)]
    pub state: Account<'info, State>,
    /// CHECK : Marinade CPI
    #[account(signer)]
    pub manager_authority: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct OrderUnstake<'info> {
    #[account(mut)]
    pub state: Box<Account<'info, State>>,
    #[account(mut)]
    pub msol_mint: Account<'info, Mint>,

    // Note: Ticket beneficiary is burn_msol_from.owner
    #[account(mut)]
    pub burn_msol_from: Account<'info, TokenAccount>,

    /// CHECK : Marinade CPI
    #[account(signer)]
    pub burn_msol_authority: AccountInfo<'info>, // burn_msol_from acc must be pre-delegated with enough amount to this key or input owner signature here

    #[account(zero, rent_exempt = enforce)]
    pub new_ticket_account: Account<'info, TicketAccountData>,

    pub clock: Sysvar<'info, Clock>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(mut)]
    pub state: Account<'info, State>,
    /// CHECK : Marinade CPI
    #[account(mut)]
    pub reserve_pda: AccountInfo<'info>,

    #[account(mut)]
    pub ticket_account: Account<'info, TicketAccountData>,

    /// CHECK : Marinade CPI
    #[account(mut)]
    pub transfer_sol_to: AccountInfo<'info>,

    pub clock: Sysvar<'info, Clock>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct StakeReserve<'info> {
    #[account(mut)]
    pub state: Box<Account<'info, State>>,
    /// CHECK : Marinade CPI
    #[account(mut)]
    pub validator_list: AccountInfo<'info>,
    /// CHECK : Marinade CPI
    #[account(mut)]
    pub stake_list: AccountInfo<'info>,
    /// CHECK : Marinade CPI
    #[account(mut)]
    pub validator_vote: AccountInfo<'info>,
    /// CHECK : Marinade CPI
    #[account(mut)]
    pub reserve_pda: AccountInfo<'info>,
    #[account(mut)]
    pub stake_account: Account<'info, StakeWrapper>, // must be uninitialized
    /// CHECK : Marinade CPI
    pub stake_deposit_authority: AccountInfo<'info>,

    pub clock: Sysvar<'info, Clock>,
    pub epoch_schedule: Sysvar<'info, EpochSchedule>,
    pub rent: Sysvar<'info, Rent>,
    /// CHECK : Marinade CPI
    pub stake_history: AccountInfo<'info>, // have no CPU budget to parse Sysvar<'info, StakeHistory>,
    /// CHECK : Marinade CPI
    pub stake_config: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    /// CHECK : Marinade CPI
    pub stake_program: AccountInfo<'info>,
}
#[derive(Accounts)]
pub struct UpdateCommon<'info> {
    #[account(mut)]
    pub state: Box<Account<'info, State>>,
    /// CHECK : Marinade CPI
    #[account(mut)]
    pub stake_list: AccountInfo<'info>,
    #[account(mut)]
    pub stake_account: Account<'info, StakeWrapper>,
    /// CHECK : Marinade CPI
    pub stake_withdraw_authority: AccountInfo<'info>, // for getting non delegated SOLs
    /// CHECK : Marinade CPI
    #[account(mut)]
    pub reserve_pda: AccountInfo<'info>, // all non delegated SOLs (if some attacker transfers it to stake) are sent to reserve_pda

    #[account(mut)]
    pub msol_mint: Account<'info, Mint>,
    /// CHECK : Marinade CPI
    pub msol_mint_authority: AccountInfo<'info>,
    /// CHECK : Marinade CPI
    #[account(mut)]
    pub treasury_msol_account: AccountInfo<'info>, //receives 1% from staking rewards protocol fee

    pub clock: Sysvar<'info, Clock>,
    /// CHECK : Marinade CPI
    pub stake_history: AccountInfo<'info>, // have no CPU budget to parse Sysvar<'info, StakeHistory>,

    /// CHECK : Marinade CPI
    pub stake_program: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct UpdateActive<'info> {
    pub common: UpdateCommon<'info>,
    /// CHECK : Marinade CPI
    #[account(mut)]
    pub validator_list: AccountInfo<'info>,
}

impl<'info> Deref for UpdateActive<'info> {
    type Target = UpdateCommon<'info>;

    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

impl<'info> DerefMut for UpdateActive<'info> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.common
    }
}

#[derive(Accounts)]
pub struct UpdateDeactivated<'info> {
    pub common: UpdateCommon<'info>,

    /// CHECK : Marinade CPI
    #[account(mut)]
    pub operational_sol_account: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> Deref for UpdateDeactivated<'info> {
    type Target = UpdateCommon<'info>;

    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

impl<'info> DerefMut for UpdateDeactivated<'info> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.common
    }
}

#[derive(Accounts)]
pub struct SetLpParams<'info> {
    #[account(mut)]
    pub state: Account<'info, State>,
    /// CHECK : Marinade CPI
    #[account(signer)]
    pub admin_authority: AccountInfo<'info>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub struct ConfigMarinadeParams {
    pub rewards_fee: Option<Fee>,
    pub slots_for_stake_delta: Option<u64>,
    pub min_stake: Option<u64>,
    pub min_deposit: Option<u64>,
    pub min_withdraw: Option<u64>,
    pub staking_sol_cap: Option<u64>,
    pub liquidity_sol_cap: Option<u64>,
    pub auto_add_validator_enabled: Option<bool>,
}

#[derive(Accounts)]
pub struct ConfigMarinade<'info> {
    #[account(mut)]
    pub state: Account<'info, State>,
    /// CHECK : Marinade CPI
    #[account(signer)]
    pub admin_authority: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct DeactivateStake<'info> {
    #[account(mut)]
    pub state: Box<Account<'info, State>>,
    /// CHECK : Marinade CPI
    // Readonly. For stake delta calculation
    pub reserve_pda: AccountInfo<'info>,
    /// CHECK : Marinade CPI
    #[account(mut)]
    pub validator_list: AccountInfo<'info>,
    /// CHECK : Marinade CPI
    #[account(mut)]
    pub stake_list: AccountInfo<'info>,
    #[account(mut)]
    pub stake_account: Account<'info, StakeWrapper>,
    /// CHECK : Marinade CPI
    pub stake_deposit_authority: AccountInfo<'info>,
    /// CHECK : Marinade CPI
    #[account(mut, signer)]
    pub split_stake_account: AccountInfo<'info>,
    /// CHECK : Marinade CPI
    #[account(mut, signer)]
    pub split_stake_rent_payer: AccountInfo<'info>,

    pub clock: Sysvar<'info, Clock>,
    pub rent: Sysvar<'info, Rent>,
    pub epoch_schedule: Sysvar<'info, EpochSchedule>,
    /// CHECK : Marinade CPI
    pub stake_history: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    /// CHECK : Marinade CPI
    pub stake_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct EmergencyUnstake<'info> {
    #[account(mut)]
    pub state: Account<'info, State>,
    /// CHECK : Marinade CPI
    #[account(signer)]
    pub validator_manager_authority: AccountInfo<'info>,
    /// CHECK : Marinade CPI
    #[account(mut)]
    pub validator_list: AccountInfo<'info>,
    /// CHECK : Marinade CPI
    #[account(mut)]
    pub stake_list: AccountInfo<'info>,
    #[account(mut)]
    pub stake_account: Account<'info, StakeWrapper>,
    /// CHECK : Marinade CPI
    pub stake_deposit_authority: AccountInfo<'info>,

    pub clock: Sysvar<'info, Clock>,

    /// CHECK : Marinade CPI
    pub stake_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct MergeStakes<'info> {
    #[account(mut)]
    pub state: Box<Account<'info, State>>,
    /// CHECK : Marinade CPI
    #[account(mut)]
    pub stake_list: AccountInfo<'info>,
    /// CHECK : Marinade CPI
    #[account(mut)]
    pub validator_list: AccountInfo<'info>,
    #[account(mut)]
    pub destination_stake: Account<'info, StakeWrapper>,
    #[account(mut)]
    pub source_stake: Account<'info, StakeWrapper>,
    /// CHECK : Marinade CPI
    pub stake_deposit_authority: AccountInfo<'info>,
    /// CHECK : Marinade CPI
    pub stake_withdraw_authority: AccountInfo<'info>,
    /// CHECK : Marinade CPI
    #[account(mut)]
    pub operational_sol_account: AccountInfo<'info>,

    pub clock: Sysvar<'info, Clock>,
    /// CHECK : Marinade CPI
    pub stake_history: AccountInfo<'info>, // have no CPU budget to parse Sysvar<'info, StakeHistory>,

    /// CHECK : Marinade CPI
    pub stake_program: AccountInfo<'info>,
}
