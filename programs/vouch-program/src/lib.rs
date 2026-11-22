use anchor_lang::prelude::*;

declare_id!("k2MpXasbGoR6KLex2CHQMhATbExaNeZTZpaH58BJJ4x");

#[program]
pub mod vouch_program {
    use super::*;

    pub fn register_agent(
        ctx: Context<RegisterAgent>,
        name: String,
        description: String,
        capabilities: String,
    ) -> Result<()> {
        require!(name.len() <= 32, VouchError::NameTooLong);
        require!(description.len() <= 256, VouchError::DescriptionTooLong);

        let agent = &mut ctx.accounts.agent;
        agent.authority = ctx.accounts.authority.key();
        agent.wallet = ctx.accounts.wallet.key();
        agent.name = name;
        agent.description = description;
        agent.capabilities = capabilities;
        agent.reputation_score = 0;
        agent.onchain_score = 0;
        agent.vouch_score = 0;
        agent.total_vouches = 0;
        agent.anomaly_flags = 0;
        agent.anomaly_penalty = 0;
        agent.registered_at = Clock::get()?.unix_timestamp;
        agent.last_scored = 0;
        agent.bump = ctx.bumps.agent;

        emit!(AgentRegistered {
            wallet: agent.wallet,
            name: agent.name.clone(),
            timestamp: agent.registered_at,
        });

        Ok(())
    }

    pub fn update_score(
        ctx: Context<UpdateScore>,
        reputation_score: u8,
        onchain_score: u8,
        vouch_score: u8,
        anomaly_flags: u8,
        anomaly_penalty: u8,
    ) -> Result<()> {
        let agent = &mut ctx.accounts.agent;
        agent.reputation_score = reputation_score;
        agent.onchain_score = onchain_score;
        agent.vouch_score = vouch_score;
        agent.anomaly_flags = anomaly_flags;
        agent.anomaly_penalty = anomaly_penalty;
        agent.last_scored = Clock::get()?.unix_timestamp;

        emit!(ScoreUpdated {
            wallet: agent.wallet,
            reputation_score,
            onchain_score,
            anomaly_flags,
            timestamp: agent.last_scored,
        });

        Ok(())
    }

    pub fn vouch_for(
        ctx: Context<VouchFor>,
        score: u8,
        comment: String,
    ) -> Result<()> {
        require!(score >= 1 && score <= 5, VouchError::InvalidScore);
        require!(comment.len() <= 128, VouchError::CommentTooLong);
        require!(
            ctx.accounts.voucher.key() != ctx.accounts.agent.wallet,
            VouchError::CannotVouchSelf
        );

        let vouch = &mut ctx.accounts.vouch;
        vouch.voucher = ctx.accounts.voucher.key();
        vouch.agent = ctx.accounts.agent.key();
        vouch.score = score;
        vouch.comment = comment;
        vouch.timestamp = Clock::get()?.unix_timestamp;
        vouch.bump = ctx.bumps.vouch;

        let agent = &mut ctx.accounts.agent;
        agent.total_vouches += 1;

        emit!(VouchSubmitted {
            voucher: vouch.voucher,
            agent_wallet: agent.wallet,
            score,
            timestamp: vouch.timestamp,
        });

        Ok(())
    }

    pub fn check_reputation(ctx: Context<CheckReputation>) -> Result<()> {
        let agent = &ctx.accounts.agent;

        emit!(ReputationChecked {
            wallet: agent.wallet,
            reputation_score: agent.reputation_score,
            anomaly_flags: agent.anomaly_flags,
            checker: ctx.accounts.checker.key(),
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }
}

// === ACCOUNTS ===

#[derive(Accounts)]
#[instruction(name: String)]
pub struct RegisterAgent<'info> {
    #[account(
        init,
        payer = authority,
        space = AgentAccount::SIZE,
        seeds = [b"agent", wallet.key().as_ref()],
        bump
    )]
    pub agent: Account<'info, AgentAccount>,
    /// CHECK: The wallet being registered
    pub wallet: UncheckedAccount<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateScore<'info> {
    #[account(
        mut,
        seeds = [b"agent", agent.wallet.as_ref()],
        bump = agent.bump,
    )]
    pub agent: Account<'info, AgentAccount>,
    #[account(
        constraint = scorer.key() == agent.authority @ VouchError::Unauthorized
    )]
    pub scorer: Signer<'info>,
}

#[derive(Accounts)]
pub struct VouchFor<'info> {
    #[account(
        init,
        payer = voucher,
        space = VouchAccount::SIZE,
        seeds = [b"vouch", voucher.key().as_ref(), agent.key().as_ref()],
        bump
    )]
    pub vouch: Account<'info, VouchAccount>,
    #[account(mut)]
    pub agent: Account<'info, AgentAccount>,
    #[account(mut)]
    pub voucher: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CheckReputation<'info> {
    #[account(
        seeds = [b"agent", agent.wallet.as_ref()],
        bump = agent.bump,
    )]
    pub agent: Account<'info, AgentAccount>,
    pub checker: Signer<'info>,
}

// === STATE ===

#[account]
pub struct AgentAccount {
    pub authority: Pubkey,
    pub wallet: Pubkey,
    pub name: String,
    pub description: String,
    pub capabilities: String,
    pub reputation_score: u8,
    pub onchain_score: u8,
    pub vouch_score: u8,
    pub total_vouches: u32,
    pub anomaly_flags: u8,
    pub anomaly_penalty: u8,
    pub registered_at: i64,
    pub last_scored: i64,
    pub bump: u8,
}

impl AgentAccount {
    pub const SIZE: usize = 8  // discriminator
        + 32  // authority
        + 32  // wallet
        + 4 + 32  // name
        + 4 + 256  // description
        + 4 + 128  // capabilities
        + 1  // reputation_score
        + 1  // onchain_score
        + 1  // vouch_score
        + 4  // total_vouches
        + 1  // anomaly_flags
        + 1  // anomaly_penalty
        + 8  // registered_at
        + 8  // last_scored
        + 1; // bump
}

#[account]
pub struct VouchAccount {
    pub voucher: Pubkey,
    pub agent: Pubkey,
    pub score: u8,
    pub comment: String,
    pub timestamp: i64,
    pub bump: u8,
}

impl VouchAccount {
    pub const SIZE: usize = 8  // discriminator
        + 32  // voucher
        + 32  // agent
        + 1   // score
        + 4 + 128  // comment
        + 8   // timestamp
        + 1;  // bump
}

// === EVENTS ===

#[event]
pub struct AgentRegistered {
    pub wallet: Pubkey,
    pub name: String,
    pub timestamp: i64,
}

#[event]
pub struct ScoreUpdated {
    pub wallet: Pubkey,
    pub reputation_score: u8,
    pub onchain_score: u8,
    pub anomaly_flags: u8,
    pub timestamp: i64,
}

#[event]
pub struct VouchSubmitted {
    pub voucher: Pubkey,
    pub agent_wallet: Pubkey,
    pub score: u8,
    pub timestamp: i64,
}

#[event]
pub struct ReputationChecked {
    pub wallet: Pubkey,
    pub reputation_score: u8,
    pub anomaly_flags: u8,
    pub checker: Pubkey,
    pub timestamp: i64,
}

// === ERRORS ===

#[error_code]
pub enum VouchError {
    #[msg("Name too long (max 32 chars)")]
    NameTooLong,
    #[msg("Description too long (max 256 chars)")]
    DescriptionTooLong,
    #[msg("Comment too long (max 128 chars)")]
    CommentTooLong,
    #[msg("Score must be 1-5")]
    InvalidScore,
    #[msg("Cannot vouch for yourself")]
    CannotVouchSelf,
    #[msg("Unauthorized")]
    Unauthorized,
}
