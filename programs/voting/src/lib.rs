use anchor_lang::prelude::*;

declare_id!("5vxREM6UabKcfjo7RcwL2ECQ5mJsfRcUa9vnisDAH3Z6");

#[program]
pub mod voting {
    use super::*;

    pub fn initialize_poll(
        ctx: Context<InitializePoll>,
        _poll_id: u64,
        start_time: u64,
        end_time: u64,
        name: String,
        description: String
    ) -> Result<()> {
        let poll_account = &mut ctx.accounts.poll_account;
        poll_account.poll_name = name;
        poll_account.poll_description = description;
        poll_account.poll_voting_start = start_time;
        poll_account.poll_voting_end = end_time;

        Ok(())
    }

    pub fn initialize_candidate(
        ctx: Context<InitializeCandidate>,
        _poll_id: u64,
        candidate: String
    ) -> Result<()> {
        let mut candidate_account = &mut ctx.accounts.candidate_account;
        candidate_account.candidate_id = _poll_id;
        candidate_account.candidate_name = candidate;
        ctx.accounts.poll_account.poll_option_index += 1;

        // Add the candidate's pubkey to the poll
        ctx.accounts.poll_account.poll_candidate.push(candidate_account.key());

        msg!("successfully");

        Ok(())
    }

    pub fn vote(ctx: Context<Vote>, _poll_id: u64, _candidate: String) -> Result<()> {
        let candidate_account = &mut ctx.accounts.candidate_account;
        let current_time = Clock::get()?.unix_timestamp;

        if current_time > (ctx.accounts.poll_account.poll_voting_end as i64) {
            return Err(ErrorCode::VotingEnded.into());
        }

        if current_time <= (ctx.accounts.poll_account.poll_voting_start as i64) {
            return Err(ErrorCode::VotingNotStarted.into());
        }

        candidate_account.candidate_votes += 1;

        let vote_record = &mut ctx.accounts.vote_record_account;
        vote_record.poll_id = _poll_id;
        vote_record.user_public_key = ctx.accounts.signer.key();

        Ok(())
    }

    pub fn get_winner(ctx: Context<Win>, _poll_id: u64) -> Result<()> {
        let poll_account = &ctx.accounts.poll_account;
        let current_time = Clock::get()?.unix_timestamp;

        // if current_time <= (poll_account.poll_voting_end as i64) {
        //     return Err(ErrorCode::NoVotingEnded.into());
        // }

        let mut max_votes = 0u64;
        let mut winner: Option<(Pubkey, String)> = None;

        for account_info in ctx.remaining_accounts.iter() {
            let mut data: &[u8] = &account_info.data.borrow();
            let candidate_account = CandidateAccount::try_deserialize_unchecked(&mut data)?;

            if candidate_account.candidate_votes > max_votes {
                max_votes = candidate_account.candidate_votes;
                winner = Some((*account_info.key, candidate_account.candidate_name));
            }
        }

        match winner {
            Some((winner_pubkey, name)) => {
                msg!("🎉 Winning candidate: {} ({}) with {} votes", name, winner_pubkey, max_votes);
                Ok(())
            }
            None => Err(ErrorCode::NoCandidates.into()),
        }
    }
    #[derive(Accounts)]
    #[instruction(poll_id: u64)]
    pub struct InitializePoll<'info> {
        #[account(mut)]
        pub signer: Signer<'info>,

        #[account(
            init,
            payer = signer,
            space = 8 + PollAccount::INIT_SPACE + CandidateAccount::INIT_SPACE,
            seeds = [b"poll".as_ref(), poll_id.to_le_bytes().as_ref()],
            bump
        )]
        pub poll_account: Account<'info, PollAccount>,

        pub system_program: Program<'info, System>,
    }

    #[derive(Accounts)]
    #[instruction(poll_id: u64, candidate: String)]
    pub struct InitializeCandidate<'info> {
        #[account(mut)]
        pub signer: Signer<'info>,

        #[account(mut)]
        pub poll_account: Account<'info, PollAccount>,

        #[account(
            init,
            payer = signer,
            space = 8 + CandidateAccount::INIT_SPACE,
            seeds = [poll_id.to_le_bytes().as_ref(), candidate.as_ref()],
            bump
        )]
        pub candidate_account: Account<'info, CandidateAccount>,

        pub system_program: Program<'info, System>,
    }

    #[derive(Accounts)]
    #[instruction(poll_id: u64, candidate: String)]
    pub struct Vote<'info> {
        #[account(mut)]
        pub signer: Signer<'info>,

        #[account(
        mut,
        seeds = [b"poll".as_ref(), poll_id.to_le_bytes().as_ref()],
        bump,
    )]
        pub poll_account: Account<'info, PollAccount>,

        #[account(
        mut,
        seeds = [poll_id.to_le_bytes().as_ref(), candidate.as_ref()],
        bump)]
        pub candidate_account: Account<'info, CandidateAccount>,

        #[account(
            init_if_needed,
            payer = signer,
            space = 8 + VoteRecord::INIT_SPACE,
            seeds = [poll_id.to_le_bytes().as_ref(), signer.key().as_ref()],
            bump
        )]
        pub vote_record_account: Account<'info, VoteRecord>,
        pub system_program: Program<'info, System>,
    }

    #[derive(Accounts)]
    #[instruction(poll_id:u64)]
    pub struct Win<'info> {
        #[account(mut)]
        pub signer: Signer<'info>,

        #[account(
        mut,
        seeds = [b"poll".as_ref(), poll_id.to_le_bytes().as_ref()],
        bump,
    )]
        pub poll_account: Account<'info, PollAccount>,
    }

    #[account]
    #[derive(InitSpace)]
    pub struct VoteRecord {
        pub poll_id: u64,
        pub user_public_key: Pubkey,
    }

    #[account]
    #[derive(InitSpace)]
    pub struct CandidateAccount {
        pub candidate_id: u64,
        #[max_len(64)] // You can choose any reasonable size
        pub candidate_name: String,
        pub candidate_votes: u64,
    }

    #[account]
    #[derive(InitSpace)]
    pub struct PollAccount {
        #[max_len(32)]
        pub poll_name: String,
        #[max_len(280)]
        pub poll_description: String,
        pub poll_voting_start: u64,
        pub poll_voting_end: u64,
        pub poll_option_index: u64,
        #[max_len(10)] // for example: max 10 candidates
        pub poll_candidate: Vec<Pubkey>,
    }

    #[error_code]
    pub enum ErrorCode {
        #[msg("Voting has not started yet")]
        VotingNotStarted,
        #[msg("Voting has ended")]
        VotingEnded,
        #[msg("candidate is not exists")]
        NoCandidates,

        #[msg("voting is not ended")]
        NoVotingEnded,
    }
}
