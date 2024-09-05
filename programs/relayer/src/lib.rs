pub mod errors;
pub mod state;

use anchor_lang::prelude::*;
use state::*;
use sha2::{Sha256, Digest};
use errors::RelayError;
use spl_math::uint::U256;

declare_id!("7iY5TvGUTxfPX2vD71k6xkHCTDKDquruKLtikL9Pmtk7");

#[program]
pub mod btc_relay {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, genesis_header: [u8; 80], genesis_height: u32) -> Result<()> {
        require!(genesis_header.len() == 80, RelayError::InvalidHeaderSize);
        require!(genesis_height > 0, RelayError::InvalidGenesisHeight);

        let relay = &mut ctx.accounts.relay_state;
        let digest = hash256(&genesis_header);
        let target = extract_target_at(&genesis_header, 0);
        let timestamp = extract_timestamp(&genesis_header);
        let chain_id = 1;

        // store bitcoin header 
        

        Ok(())
    }

    pub fn submit_block_header(ctx: Context<SubmitBlockHeader>, header: [u8; 80]) -> Result<()> {
        // Implement block header submission logic
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