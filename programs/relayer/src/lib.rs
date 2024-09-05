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

        _store_block_header(&mut ctx.accounts.header, &mut ctx.accounts.chain, digest, genesis_height)?;
        Ok(())
    }

    pub fn submit_block_header(ctx: Context<SubmitBlockHeader>, header: [u8; 80], block_hash: [u8; 32]) -> Result<()> {
        require!(header.len() == 80, RelayError::InvalidHeaderSize);

        let hash_curr_block = hash256(&header);
        require!(hash_curr_block == block_hash, RelayError::InvalidBlockHash);       
        require!(ctx.accounts.header.chain_id == 0, RelayError::DuplicateBlock);

        // let hash_prev_block = extract_prev_block_le(&header);
        // require!(ctx.accounts.headers.contains_key(&hash_prev_block), RelayError::PreviousBlockNotFound);

        // let target = extract_target(&header);
        // require!(u256_from_le_bytes(&hash_curr_block) <= target, RelayError::LowDifficulty);

        // let height = ctx.accounts.headers.get(&hash_prev_block).unwrap().height + 1;

        // if is_period_start(height) {
        //     // TODO: Implement difficulty adjustment check
        //     ctx.accounts.epoch_start_target = target;
        //     ctx.accounts.epoch_start_time = extract_timestamp(&header);
        //     ctx.accounts.epoch_end_target = 0;
        //     ctx.accounts.epoch_end_time = 0;
        // } else if is_period_end(height) {
        //     ctx.accounts.epoch_end_target = target;
        //     ctx.accounts.epoch_end_time = extract_timestamp(&header);
        // }

        // let chain_id = ctx.accounts.headers.get(&hash_prev_block).unwrap().chain_id;
        // let is_new_fork = ctx.accounts.forks.get(&chain_id).unwrap().height != ctx.accounts.headers.get(&hash_prev_block).unwrap().height;

        // if is_new_fork {
        //     let new_chain_id = ctx.accounts.chain_counter + 1;
        //     ctx.accounts.chain_counter = new_chain_id;
        //     initialize_fork(ctx, hash_curr_block, hash_prev_block, new_chain_id, height)?;
        //     store_block_header(ctx, hash_curr_block, height, new_chain_id)?;
        // } else {
        //     store_block_header(ctx, hash_curr_block, height, chain_id)?;

        //     if chain_id == MAIN_CHAIN_ID {
        //         ctx.accounts.best_block = hash_curr_block;
        //         ctx.accounts.best_height = height;
        //         ctx.accounts.forks.get_mut(&chain_id).unwrap().height = height;
        //         ctx.accounts.chain.insert(height, hash_curr_block);
        //     } else if height >= ctx.accounts.best_height + CONFIRMATIONS {
        //         reorg_chain(ctx, chain_id, height, hash_curr_block)?;
        //     } else {
        //         ctx.accounts.forks.get_mut(&chain_id).unwrap().height = height;
        //         ctx.accounts.forks.get_mut(&chain_id).unwrap().descendants.push(hash_curr_block);
        //     }
        // }

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


fn _store_block_header(
    header: &mut Account<'_, Header>,
    chain: &mut Account<'_, BlockHash>,
    digest: [u8; 32],
    height: u32,
) -> Result<()> {
    header.chain_id = CHAIN_ID;
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