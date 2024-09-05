use anchor_lang::prelude::*;
use std::mem::size_of;


#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = user, space = size_of::<RelayState>() + 8, seeds = [b"relay_state"], bump)]
    pub relay_state: Account<'info, RelayState>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SubmitBlockHeader<'info> {
    #[account(mut)]
    pub relay_state: Account<'info, RelayState>,
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct SubmitBlockHeaderBatch<'info> {
    #[account(mut)]
    pub relay_state: Account<'info, RelayState>,
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct VerifyTx<'info> {
    pub relay_state: Account<'info, RelayState>,
    pub user: Signer<'info>,
}

#[account]
pub struct RelayState {
    pub best_block: [u8; 32],
    pub best_height: u32,
    pub epoch_start_target: String,
    pub epoch_end_target: String,
    pub epoch_start_time: u64,
    pub epoch_end_time: u64,
    pub chain_couter: u64,
}

// You might need to create custom types for some of the complex structures
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct Header {
    pub height: u32,
    pub chain_id: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct Fork {
    pub height: u32,
    pub ancestor: [u8; 32],
    pub descendants: Vec<[u8; 32]>,
}

// Constants for the maximum number of headers and forks
pub const MAX_HEADERS: usize = 1000;
pub const MAX_FORKS: usize = 100;