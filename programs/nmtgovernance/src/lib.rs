// File: lib.rs
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint};

declare_id!("3GVzN1BXzsY4sKkzwDZPkAufWvVNGQKDGr8bWfpuHUY6");

// Tokenomics : 
const MIN_PROPOSAL_DEPOSIT: u64 = 111_111 * 1_000_000_000; // 111,111 NMT (considering 9 decimals)
const REVIEW_PERIOD: i64 = 3 * 24 * 60 * 60; // 3 days in seconds
const DEFAULT_VOTING_PERIOD: i64 = 7 * 24 * 60 * 60; // 7 days in seconds
const DEFAULT_TIMELOCK: i64 = 2 * 24 * 60 * 60; // 2 days in seconds
const QUORUM_PERCENTAGE: u8 = 10; // 10% quorum
const TOTAL_SUPPLY: u64 = 1_111_111_111 * 1_000_000_000; // 1,111,111,111 NMT (considering 9 decimals)


#[program]
pub mod contRideHailingGovernance {
    use super::*;

    // Governance Contract
    pub fn initialize_governance(
        ctx: Context<InitializeGovernance>,
        max_ride_distance: u32,
        cancellation_policy: String,
    ) -> Result<()> {
        let governance = &mut ctx.accounts.governance;
        governance.max_ride_distance = max_ride_distance;
        governance.cancellation_policy = cancellation_policy;
        governance.authority = *ctx.accounts.authority.key;
        Ok(())
    }

    pub fn update_governance(
        ctx: Context<UpdateGovernance>,
        max_ride_distance: u32,
        cancellation_policy: String,
    ) -> Result<()> {
        let governance = &mut ctx.accounts.governance;
        governance.max_ride_distance = max_ride_distance;
        governance.cancellation_policy = cancellation_policy;
        Ok(())
    }

    // Treasury Contract
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        token::transfer(ctx.accounts.into(), amount)
    }

    // Voting Contract
    pub fn create_proposal(
        ctx: Context<CreateProposal>,
        title: String,
        description: String,
        options: Vec<String>,
        voting_period: Option<i64>,
        timelock: Option<i64>,
    ) -> Result<()> {
        let governance = &mut ctx.accounts.governance;
        let proposal = &mut ctx.accounts.proposal;
        let treasury = &ctx.accounts.treasury;
        let clock = Clock::get()?;

        // Validate proposal creator's NMT balance
        let creator_balance = ctx.accounts.creator_token_account.amount;
        require!(creator_balance >= MIN_PROPOSAL_DEPOSIT, ErrorCode::InsufficientBalance);

        // Lock proposal deposit in the treasury
        let cpi_accounts = Transfer {
            from: ctx.accounts.creator_token_account.to_account_info(),
            to: ctx.accounts.treasury_token_account.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, MIN_PROPOSAL_DEPOSIT)?;

        // Update treasury state
        let treasury = &mut ctx.accounts.treasury;
        treasury.total_locked += MIN_PROPOSAL_DEPOSIT;

        // Initialize proposal
        proposal.description = description;
        proposal.vote_yes = 0;
        proposal.vote_no = 0;
        proposal.end_time = Clock::get()?.unix_timestamp + voting_period;
        proposal.is_active = true;
        Ok(())
    }

    pub fn vote(ctx: Context<Vote>, vote: bool) -> Result<()> {
        let proposal = &mut ctx.accounts.proposal;
        require!(proposal.is_active, ErrorCode::ProposalNotActive);
        require!(
            Clock::get()?.unix_timestamp < proposal.end_time,
            ErrorCode::VotingPeriodEnded
        );

        if vote {
            proposal.vote_yes += 1;
        } else {
            proposal.vote_no += 1;
        }
        Ok(())
    }

    pub fn end_proposal(ctx: Context<EndProposal>) -> Result<()> {
        let proposal = &mut ctx.accounts.proposal;
        require!(proposal.is_active, ErrorCode::ProposalNotActive);
        require!(
            Clock::get()?.unix_timestamp >= proposal.end_time,
            ErrorCode::VotingPeriodNotEnded
        );

        proposal.is_active = false;
        Ok(())
    }

    // current governing Params : 
    pub fn initialize_ride_hailing_params(ctx: Context<InitializeRideHailingParams>) -> Result<()> {
        let params = &mut ctx.accounts.ride_hailing_params;
        params.authority = *ctx.accounts.authority.key;
        params.min_cancellation_charge = 100; // Example: 1 NMT (assuming 2 decimal places)
        params.rider_cancellation_percentage = 50; // 50%
        params.driver_cancellation_percentage = 30; // 30%
        params.platform_cancellation_percentage = 20; // 20%
        params.platform_fee_percentage = 10; // 10%
        params.daily_subscription_fee = 500; // Example: 5 NMT
        params.min_ride_distance = 1000; // Example: 1 km (in meters)
        Ok(())
    }

    pub fn update_ride_hailing_params(
        ctx: Context<UpdateRideHailingParams>,
        new_params: RideHailingParams,
    ) -> Result<()> {
        require!(
            ctx.accounts.proposal.is_approved && ctx.accounts.proposal.is_executed,
            ErrorCode::ProposalNotApprovedOrExecuted
        );

        let params = &mut ctx.accounts.ride_hailing_params;
        *params = new_params;
        params.authority = ctx.accounts.governance.key();

        Ok(())
    }

    pub fn create_param_update_proposal(
        ctx: Context<CreateParamUpdateProposal>,
        title: String,
        description: String,
        new_params: RideHailingParams,
    ) -> Result<()> {
        // ... [Similar to create_proposal, but with specific handling for RideHailingParams]
        let governance = &mut ctx.accounts.governance;
        let proposal = &mut ctx.accounts.proposal;
        let clock = Clock::get()?;

        // Validate proposal creator's NMT balance
        let creator_balance = ctx.accounts.creator_token_account.amount;
        require!(creator_balance >= MIN_PROPOSAL_DEPOSIT, ErrorCode::InsufficientBalance);

        // Lock proposal deposit
        let cpi_accounts = Transfer {
            from: ctx.accounts.creator_token_account.to_account_info(),
            to: ctx.accounts.governance_token_account.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, MIN_PROPOSAL_DEPOSIT)?;

        // Initialize proposal
        proposal.id = governance.proposal_count;
        proposal.creator = *ctx.accounts.authority.key;
        proposal.title = title;
        proposal.description = description;
        proposal.created_at = clock.unix_timestamp;
        proposal.review_end_time = clock.unix_timestamp + REVIEW_PERIOD;
        proposal.voting_end_time = proposal.review_end_time + DEFAULT_VOTING_PERIOD;
        proposal.execution_time = proposal.voting_end_time + DEFAULT_TIMELOCK;
        proposal.is_active = true;
        proposal.is_executed = false;
        proposal.total_votes = 0;
        proposal.votes = vec![0, 0]; // Yes and No votes
        proposal.proposed_params = new_params;

        governance.proposal_count += 1;

        Ok(())
    }

    // Treasury : 
    pub fn execute_param_update_proposal(ctx: Context<ExecuteParamUpdateProposal>, proposal_id: u64) -> Result<()> {
        let proposal = &mut ctx.accounts.proposal;
        let params = &mut ctx.accounts.ride_hailing_params;
        let clock = Clock::get()?;
        let treasury = &mut ctx.accounts.treasury;
    
        require!(!proposal.is_active, ErrorCode::ProposalStillActive);
        require!(proposal.is_approved, ErrorCode::ProposalNotApproved);
        require!(!proposal.is_executed, ErrorCode::ProposalAlreadyExecuted);
        require!(
            clock.unix_timestamp >= proposal.execution_time,
            ErrorCode::TimelockNotExpired
        );

        // Execute proposal logic here
        // This would typically involve calling other functions to implement the proposal's changes

        // If the proposal involves transferring funds, you would do it here
        // For example:
        // let cpi_accounts = Transfer {
        //     from: ctx.accounts.treasury_token_account.to_account_info(),
        //     to: ctx.accounts.recipient_token_account.to_account_info(),
        //     authority: treasury.to_account_info(),
        // };
        // let cpi_program = ctx.accounts.token_program.to_account_info();
        // let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        // token::transfer(cpi_ctx, amount)?;


        // Update the parameters
        *params = proposal.proposed_params.clone();
        proposal.is_executed = true;

        Ok(())
    }

    pub fn initialize_treasury(ctx: Context<InitializeTreasury>) -> Result<()> {
        let treasury = &mut ctx.accounts.treasury;
        treasury.authority = *ctx.accounts.authority.key;
        treasury.total_locked = 0;
        Ok(())
    }

    pub fn lock_tokens(ctx: Context<LockTokens>, amount: u64) -> Result<()> {
        let treasury = &mut ctx.accounts.treasury;
        let token_account = &mut ctx.accounts.token_account;

        // Transfer tokens from user to treasury
        let cpi_accounts = Transfer {
            from: token_account.to_account_info(),
            to: ctx.accounts.treasury_token_account.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        // Update treasury state
        treasury.total_locked += amount;

        Ok(())
    }

}

#[derive(Accounts)]
pub struct InitializeGovernance<'info> {
    #[account(init, payer = authority, space = 8 + 32 + 4 + 200)]
    pub governance: Account<'info, Governance>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateGovernance<'info> {
    #[account(mut, has_one = authority)]
    pub governance: Account<'info, Governance>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct InitializeTreasury<'info> {
    #[account(init, payer = authority, space = 8 + 32 + 8)]
    pub treasury: Account<'info, Treasury>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub treasury: Account<'info, Treasury>,
    #[account(mut)]
    pub from: Account<'info, TokenAccount>,
    #[account(mut)]
    pub to: Account<'info, TokenAccount>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Vote<'info> {
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
    pub voter: Signer<'info>,
}

#[derive(Accounts)]
pub struct EndProposal<'info> {
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
    pub authority: Signer<'info>,
}

#[account]
pub struct Governance {
    pub authority: Pubkey,
    pub max_ride_distance: u32,
    pub cancellation_policy: String,
}

#[account]
pub struct Treasury {
    pub authority: Pubkey,
    pub total_locked: u64,
}

#[account]
pub struct Proposal {
    pub description: String,
    pub vote_yes: u64,
    pub vote_no: u64,
    pub end_time: i64,
    pub is_active: bool,
    pub proposed_params: RideHailingParams,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Proposal not approved or not executed")]
    ProposalNotApprovedOrExecuted,
    #[msg("Insufficient NMT balance for proposal creation")]
    InsufficientBalance,
    #[msg("Proposal is not active")]
    ProposalNotActive,
    #[msg("Review period has not ended yet")]
    ReviewPeriodNotEnded,
    #[msg("Voting period has ended")]
    VotingPeriodEnded,
    #[msg("Invalid vote option")]
    InvalidVoteOption,
    #[msg("Voting period has not ended yet")]
    VotingPeriodNotEnded,
    #[msg("Quorum not reached")]
    QuorumNotReached,
    #[msg("Proposal is still active")]
    ProposalStillActive,
    #[msg("Proposal was not approved")]
    ProposalNotApproved,
    #[msg("Proposal has already been executed")]
    ProposalAlreadyExecuted,
    #[msg("Timelock period has not expired yet")]
    TimelockNotExpired,
}

impl<'info> From<&mut Deposit<'info>> for CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
    fn from(accounts: &mut Deposit<'info>) -> Self {
        let cpi_accounts = Transfer {
            from: accounts.from.to_account_info(),
            to: accounts.to.to_account_info(),
            authority: accounts.authority.to_account_info(),
        };
        let cpi_program = accounts.token_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }
}

// Initialize Ride Hailing Params

#[derive(Accounts)]
pub struct InitializeRideHailingParams<'info> {
    #[account(init, payer = authority, space = 8 + 32 + 8 + 8 + 8 + 8 + 8 + 8 + 8)]
    pub ride_hailing_params: Account<'info, RideHailingParams>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateRideHailingParams<'info> {
    #[account(mut)]
    pub ride_hailing_params: Account<'info, RideHailingParams>,
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
    pub governance: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(title: String, description: String)]
pub struct CreateParamUpdateProposal<'info> {
    #[account(mut)]
    pub governance: Account<'info, Governance>,
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 8 + 100 + 1000 + 8 + 8 + 8 + 8 + 1 + 1 + 8 + (4 + 8 * 2) + (4 + (32 + 8 + 1 + 8) * 1000) + 8 + 8 + 8 + 8 + 8 + 8 + 8
    )]
    pub proposal: Account<'info, Proposal>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub creator_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub governance_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ExecuteParamUpdateProposal<'info> {
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
    #[account(mut)]
    pub ride_hailing_params: Account<'info, RideHailingParams>,
    pub authority: Signer<'info>,
}

#[account]
#[derive(Default)]
pub struct RideHailingParams {
    pub authority: Pubkey,
    pub min_cancellation_charge: u64,
    pub rider_cancellation_percentage: u8,
    pub driver_cancellation_percentage: u8,
    pub platform_cancellation_percentage: u8,
    pub platform_fee_percentage: u8,
    pub daily_subscription_fee: u64,
    pub min_ride_distance: u64,
}




#[derive(Accounts)]
pub struct LockTokens<'info> {
    #[account(mut)]
    pub treasury: Account<'info, Treasury>,
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub treasury_token_account: Account<'info, TokenAccount>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(title: String, description: String, options: Vec<String>)]
pub struct CreateProposal<'info> {
    #[account(mut)]
    pub governance: Account<'info, Governance>,
    #[account(mut)]
    pub treasury: Account<'info, Treasury>,
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 8 + 100 + 1000 + (4 + 50 * options.len()) + 8 + 8 + 8 + 8 + 8 + 8 + 1 + 1 + 8 + (4 + 8 * options.len()) + (4 + (32 + 8 + 1 + 8) * 1000)
    )]
    pub proposal: Account<'info, Proposal>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub creator_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub treasury_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
pub struct ExecuteProposal<'info> {
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
    #[account(mut)]
    pub treasury: Account<'info, Treasury>,
    #[account(mut)]
    pub treasury_token_account: Account<'info, TokenAccount>,
    // Include other accounts that might be needed for execution
    // pub recipient_token_account: Account<'info, TokenAccount>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}
