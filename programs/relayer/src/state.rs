use anchor_lang::prelude::*;
use std::mem::size_of;

// chain id must != 0
pub const CHAIN_ID: u32 = 10;
pub const DIFFICULTY_ADJUSTMENT_INTERVAL: u32 = 2016;
pub const DIFF1_TARGET: &str = "ffff0000000000000000000000000000000000000000000000000000";
pub const RETARGET_PERIOD: u32 = 1209600; // 2 weeks in seconds
pub const CONFIRMATIONS: u32 = 6;
pub const MAIN_CHAIN_ID: u32 = 1;

#[derive(Accounts)]
#[instruction(genesis_height: u32, genesis_block_hash: [u8; 32])]
pub struct Initialize<'info> {
    #[account(init, payer = user, space = size_of::<RelayState>() + 8 + 32 * 2, seeds = [b"relay_state"], bump)]
    pub relay_state: Account<'info, RelayState>,
    #[account(init, payer = user, space = size_of::<Fork>() + 8 + 32 * 8, seeds = [b"fork", CHAIN_ID.to_le_bytes().as_ref()], bump)]
    pub fork: Account<'info, Fork>,
    #[account(init, payer = user, space = size_of::<BlockHash>() + 8, seeds = [b"chain", genesis_height.to_le_bytes().as_ref()], bump)]
    pub chain: Account<'info, BlockHash>,
    #[account(init, payer = user, space = size_of::<Header>() + 8, seeds = [b"header", genesis_block_hash.as_ref()], bump)]
    pub header: Account<'info, Header>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(block_hash: [u8; 32], prev_block_hash: [u8; 32], prev_block_hash_chain_id: u32, block_height: u32, next_counter: u32)] //, 
pub struct SubmitBlockHeader<'info> {
    #[account(mut)]
    pub relay_state: Account<'info, RelayState>,
    #[account(seeds = [b"header", prev_block_hash.as_ref()], bump)]
    pub prev_header: Account<'info, Header>, 
    #[account(seeds = [b"fork", prev_block_hash_chain_id.to_le_bytes().as_ref()], bump)]
    pub prev_fork: Account<'info, Fork>,
    #[account(init_if_needed, payer = user, space = size_of::<Fork>() + 8 + 32 * 8, seeds = [b"fork", next_counter.to_le_bytes().as_ref()], bump)]
    pub fork: Account<'info, Fork>,
    #[account(init_if_needed, payer = user, space = size_of::<BlockHash>() + 8, seeds = [b"chain", block_height.to_le_bytes().as_ref()], bump)]
    pub chain: Account<'info, BlockHash>,
    #[account(init_if_needed, payer = user, space = size_of::<Header>() + 8, seeds = [b"header", block_hash.as_ref()], bump)]
    pub header: Account<'info, Header>, 
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
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
    pub epoch_start_time: u32,
    pub epoch_end_time: u32,
    pub chain_counter: u32,
}

// You might need to create custom types for some of the complex structures
#[account]
pub struct Header {
    pub height: u32,
    pub chain_id: u32,
}

#[account]
pub struct BlockHash {
    pub block_hash: [u8; 32],
}

#[account]
pub struct Fork {
    pub height: u32,
    pub ancestor: [u8; 32],
    pub descendants: Vec<[u8; 32]>,
}

#[event]
pub struct ChainReorg {
    pub from: [u8; 32],
    pub to: [u8; 32],
    pub chain_id: u64,
}

// Constants for the maximum number of headers and forks
pub const MAX_HEADERS: usize = 1000;
pub const MAX_FORKS: usize = 100;