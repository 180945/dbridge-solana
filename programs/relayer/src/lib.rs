pub mod errors;
pub mod state;

use anchor_lang::prelude::*;
use state::*;
use sha2::{Digest, Sha256};
use errors::RelayError;
use spl_math::uint::U256;

declare_id!("7iY5TvGUTxfPX2vD71k6xkHCTDKDquruKLtikL9Pmtk7");

#[program]
pub mod btc_relay {
    use super::*;
    /// Initializes the BTC relay with the genesis block information
    ///
    /// This function sets up the initial state of the BTC relay by storing the genesis block
    /// information. It performs several checks to ensure the validity of the provided data:
    /// - Verifies the header size is correct (80 bytes)
    /// - Ensures the genesis height is greater than 0
    /// - Validates that the provided block hash matches the hash of the genesis header
    ///
    /// After validation, it initializes the relay state with the genesis block information,
    /// including the best block hash, height, and initial difficulty target. It also sets up
    /// the initial fork and stores the genesis block header.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context for the instruction
    /// * `genesis_header` - The 80-byte Bitcoin genesis block header
    /// * `genesis_height` - The height of the genesis block
    /// * `genesis_block_hash` - The hash of the genesis block
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The header size is invalid
    /// - The genesis height is 0 or negative
    /// - The provided block hash doesn't match the hash of the genesis header
    pub fn initialize(ctx: Context<Initialize>, genesis_header: [u8; 80], genesis_height: u32, genesis_block_hash: [u8; 32]) -> Result<()> {
        require!(genesis_header.len() == 80, RelayError::InvalidHeaderSize);
        require!(genesis_height > 0, RelayError::InvalidGenesisHeight);

        let digest = hash256(&genesis_header);
        require!(digest == genesis_block_hash, RelayError::InvalidBlockHash);

        let target = extract_target_at(&genesis_header, 0);
        let timestamp = extract_timestamp(&genesis_header);

        // store bitcoin header 
        let relay_state = &mut ctx.accounts.relay_state;
        relay_state.best_block = digest;
        relay_state.best_height = genesis_height;
        relay_state.epoch_start_target = target.to_string();
        relay_state.epoch_end_target = target.to_string();
        relay_state.epoch_start_time = timestamp;
        relay_state.epoch_end_time = timestamp;

        let fork = &mut ctx.accounts.fork;
        fork.height = genesis_height;

        _store_block_header(&mut ctx.accounts.header, &mut ctx.accounts.chain, digest, genesis_height, CHAIN_ID)?;
        Ok(())
    }

    /// This function submits a new block header to the relay.
    /// 
    /// It performs several checks to ensure the validity of the submitted header:
    /// - Verifies the header size is correct (80 bytes)
    /// - Ensures the chain counter is incremented correctly
    /// - Validates that the provided block hash matches the hash of the header
    /// - Checks that the block hasn't been submitted before
    /// - Verifies the previous block exists and is at the correct height
    /// - Ensures the chain ID is valid
    /// - Checks that the block meets the required difficulty target
    /// 
    /// If the block is at the start of a new difficulty adjustment period, it also verifies
    /// the new difficulty target. At the end of a period, it updates the epoch end information.
    /// 
    /// The function handles creating new forks when necessary and updates the relay state
    /// with the new best block if applicable.
    /// 
    /// # Arguments
    /// 
    /// * `ctx` - The context for the instruction
    /// * `header` - The 80-byte Bitcoin block header
    /// * `block_hash` - The hash of the block
    /// * `prev_block_hash` - The hash of the previous block
    /// * `prev_block_hash_chain_id` - The chain ID of the previous block
    /// * `block_height` - The height of the new block
    /// * `next_counter` - The next chain counter value
    /// 
    /// # Errors
    /// 
    /// This function will return an error if any of the validity checks fail.
    pub fn submit_block_header(
        ctx: Context<SubmitBlockHeader>, 
        header: [u8; 80], 
        block_hash: [u8; 32], 
        _prev_block_hash: [u8; 32], 
        prev_block_hash_chain_id: u32, 
        block_height: u32,
        next_counter: u32
    ) -> Result<()> {
        require!(header.len() == 80, RelayError::InvalidHeaderSize);
        require!(ctx.accounts.relay_state.chain_counter + 1 == next_counter, RelayError::InvalidCounter);

        let hash_curr_block: [u8; 32] = hash256(&header);
        require!(hash_curr_block == block_hash, RelayError::InvalidBlockHash);       
        require!(ctx.accounts.header.chain_id == 0, RelayError::DuplicateBlock);
        let prv_height = ctx.accounts.prev_header.height;
        require!(prv_height > 0 && prv_height == block_height - 1, RelayError::PreviousBlockNotFound);
        require!(ctx.accounts.prev_header.chain_id == prev_block_hash_chain_id, RelayError::InvalidChainId);

        let target = extract_target_at(&header, 0);
        require!(U256::from_little_endian(&hash_curr_block) <= target, RelayError::LowDifficulty);

        let prv_target = U256::from_dec_str(&ctx.accounts.relay_state.epoch_start_target).unwrap();
        let prv_end_target = U256::from_dec_str(&ctx.accounts.relay_state.epoch_end_target).unwrap();
        
        if is_period_start(block_height) {
            require!(
                is_correct_difficulty_target(
                    prv_target,
                    ctx.accounts.relay_state.epoch_start_time,
                    prv_end_target,
                    ctx.accounts.relay_state.epoch_end_time,
                    target,
                ).unwrap_or_default(),
                RelayError::IncorrectDifficultyTarget
            );

            ctx.accounts.relay_state.epoch_start_target = target.to_string();
            ctx.accounts.relay_state.epoch_start_time = extract_timestamp(&header);
            ctx.accounts.relay_state.epoch_end_target = String::new();
            ctx.accounts.relay_state.epoch_end_time = 0;
        } else if is_period_end(block_height) {
            ctx.accounts.relay_state.epoch_end_target = target.to_string();
            ctx.accounts.relay_state.epoch_end_time = extract_timestamp(&header);
        }

        let is_new_fork = ctx.accounts.prev_fork.height != ctx.accounts.prev_header.height;
        if is_new_fork {
            ctx.accounts.relay_state.chain_counter = next_counter;
            _initialize_fork(&mut ctx.accounts.fork, hash_curr_block, ctx.accounts.relay_state.best_block, next_counter, block_height)?;
            _store_block_header(&mut ctx.accounts.header, &mut ctx.accounts.chain, hash_curr_block, block_height, next_counter)?;
        } else {
            _store_block_header(&mut ctx.accounts.header, &mut ctx.accounts.chain, hash_curr_block, block_height, prev_block_hash_chain_id)?;
            if prev_block_hash_chain_id == MAIN_CHAIN_ID {
                ctx.accounts.relay_state.best_block = hash_curr_block;
                ctx.accounts.relay_state.best_height = block_height;
                ctx.accounts.fork.height = block_height;
                ctx.accounts.fork.descendants.push(hash_curr_block);
            } else if block_height >= ctx.accounts.relay_state.best_height + CONFIRMATIONS {
                reorg_chain(ctx, prev_block_hash_chain_id, block_height, hash_curr_block)?;
            } else {
                ctx.accounts.fork.height = block_height;
                ctx.accounts.fork.descendants.push(hash_curr_block);
            }
        }

        Ok(())
    }

    pub fn submit_block_header_batch(ctx: Context<SubmitBlockHeaderBatch>, headers: Vec<[u8; 80]>) -> Result<()> {
        // Implement batch submission logic
        Ok(())
    }

    pub fn verify_tx(ctx: Context<VerifyTx>, height: u32, index: u64, txid: [u8; 32], header: [u8; 80], proof: Vec<u8>, confirmations: u64, insecure: bool) -> Result<bool> {
        // Implement transaction verification logic
        Ok(false)
    }

    // Add other functions as needed
}

fn reorg_chain(
    ctx: Context<SubmitBlockHeader>, 
    chain_id: u32,
    height: u32,
    hash_curr_block: [u8; 32],
) -> Result<()> {
    let relay = &mut ctx.accounts.relay_state;
    relay.chain_counter += 1;

    // reorg fork to main
    let mut ancestor_id = chain_id;
    let fork_id = relay.chain_counter;
    let mut fork_height = height - 1;

    // TODO: add new fork struct for old main

    // while ancestor_id != relay.MAIN_CHAIN_ID {
    //     let fork = relay.forks.get(&ancestor_id).ok_or(RelayError::ForkNotFound)?;
        
    //     for i in (0..fork.descendants.len()).rev() {
    //         let descendant = fork.descendants[i];
            
    //         // promote header to main chain
    //         relay.headers.get_mut(&descendant).unwrap().chain_id = relay.MAIN_CHAIN_ID;
            
    //         // demote old header to new fork
    //         relay.headers.get_mut(&relay.chain[height as usize]).unwrap().chain_id = fork_id;
            
    //         // swap header at height
    //         relay.chain[height as usize] = descendant;
    //         fork_height -= 1;
    //     }

    //     let ancestor = fork.ancestor;
    //     ancestor_id = relay.headers.get(&ancestor).unwrap().chain_id;
    // }

    // emit!(ChainReorg {
    //     from: relay.best_block,
    //     to: hash_curr_block,
    //     chain_id,
    // });

    // relay.best_block = hash_curr_block;
    // relay.best_height = height;

    // relay.forks.remove(&chain_id);

    // // extend to current head
    // relay.chain[relay.best_height as usize] = relay.best_block;
    // relay.headers.get_mut(&relay.best_block).unwrap().chain_id = relay.MAIN_CHAIN_ID;

    Ok(())
}

pub fn is_correct_difficulty_target(
    prev_start_target: U256,
    prev_start_time: u32,
    prev_end_target: U256,
    prev_end_time: u32,
    next_target: U256,
) -> Result<bool> {
    // Check if the difficulty at the start and end of the previous period is the same
    require!(calculate_difficulty(prev_start_target) != calculate_difficulty(prev_end_target), RelayError::InvalidDifficultyPeriod);
    let expected_target = retarget_algorithm(prev_start_target, prev_start_time, prev_end_time).unwrap();

    Ok((next_target & expected_target) == next_target)
}

// Helper functions (you'll need to implement these)
fn calculate_difficulty(target: U256) -> U256 {
    let diff1_target = U256::from_str_radix(DIFF1_TARGET, 16).unwrap();
    diff1_target.checked_div(target).unwrap_or_default()
}

pub fn retarget_algorithm(
    previous_target: U256,
    first_timestamp: u32,
    second_timestamp: u32
) -> Result<U256> {
    let mut elapsed_time = second_timestamp.checked_sub(first_timestamp)
        .ok_or(RelayError::ArithmeticError)?;

    // Normalize ratio to factor of 4 if very long or very short
    if elapsed_time < RETARGET_PERIOD / 4 {
        elapsed_time = RETARGET_PERIOD / 4;
    }
    if elapsed_time > RETARGET_PERIOD * 4 {
        elapsed_time = RETARGET_PERIOD * 4;
    }

    // Divide by 256^2 to prevent overflow, will multiply back later
    let adjusted = previous_target
        .checked_div(65536.into())
        .ok_or(RelayError::ArithmeticError)?
        .checked_mul(elapsed_time.into())
        .ok_or(RelayError::ArithmeticError)?;

    let result = adjusted
        .checked_div(RETARGET_PERIOD.into())
        .ok_or(RelayError::ArithmeticError)?
        .checked_mul(65536.into())
        .ok_or(RelayError::ArithmeticError)?;

    Ok(result)
}

fn is_period_start(height: u32) -> bool {
    height % DIFFICULTY_ADJUSTMENT_INTERVAL == 0
}

fn is_period_end(height: u32) -> bool {
    height % DIFFICULTY_ADJUSTMENT_INTERVAL == 2015
}

fn _initialize_fork(fork: &mut Account<'_, Fork>, hash_curr_block: [u8; 32], hash_prev_block: [u8; 32], new_chain_id: u32, height: u32) -> Result<()> {
    fork.height = height;
    fork.ancestor = hash_prev_block;
    fork.descendants = vec![hash_curr_block];
    Ok(())
}

fn _store_block_header(
    header: &mut Account<'_, Header>,
    chain: &mut Account<'_, BlockHash>,
    digest: [u8; 32],
    height: u32,
    chain_id: u32,
) -> Result<()> {
    header.chain_id = chain_id;
    header.height = height;
    chain.block_hash = digest;
    
    Ok(())
}

fn hash256(b: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(Sha256::digest(b));
    hasher.finalize().into()
}

fn extract_target_at(header: &[u8], at: usize) -> U256 {
    let m: u32 = u32::from_le_bytes([header[72 + at], header[73 + at], header[74 + at], 0]);
    let e = header[75 + at];
    let mantissa = U256::from(reverse_uint24(m) as u64);
    let exponent = U256::from(e.saturating_sub(3) as u32);
    
    // Use checked arithmetic to prevent overflow
    mantissa.checked_mul(U256::from(256).checked_pow(exponent).unwrap_or(U256::from(0))).unwrap_or(U256::from(0))
}

fn reverse_uint24(b: u32) -> u32 {
    (b  << 16) | (b & 0x00FF00) | (b  >> 16)
}

fn extract_timestamp_le(data: &[u8]) -> [u8; 4] {
    data[68..72].try_into().unwrap()
}

fn extract_timestamp(data: &[u8]) -> u32 {
    let timestamp_le = extract_timestamp_le(data);
    u32::from_le_bytes(timestamp_le)
}