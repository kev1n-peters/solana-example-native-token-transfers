use anchor_lang::{prelude::*, solana_program::clock};

use crate::{chain_id::ChainId, error::NTTError, normalized_amount::NormalizedAmount};

use super::rate_limit::RateLimitState;

#[account]
#[derive(InitSpace)]
// TODO: maybe remove the queue from the name? it's not always queued
pub struct OutboundQueuedTransfer {
    pub bump: u8,
    pub sequence: u64,
    pub amount: NormalizedAmount,
    pub recipient_chain: ChainId,
    // TODO: revise max length?
    #[max_len(120)]
    pub recipient_address: Vec<u8>,
    pub release_timestamp: i64,
    // TODO: change this to a bitmap to store which endpoints have released the
    // transfer? (multi endpoint)
    pub released: bool,
}

impl OutboundQueuedTransfer {
    pub const SEED_PREFIX: &'static [u8] = b"outbound_queue";

    pub fn release(&mut self) -> Result<()> {
        let now = clock::Clock::get()?.unix_timestamp;
        if self.release_timestamp > now {
            return Err(NTTError::ReleaseTimestampNotReached.into());
        }

        if self.released {
            return Err(NTTError::MessageAlreadySent.into());
        }

        self.released = true;

        Ok(())
    }
}

#[account]
#[derive(InitSpace)]
pub struct OutboundRateLimit {
    pub rate_limit: RateLimitState,
}

/// Global rate limit for all outbound transfers to all chains.
/// NOTE: only one of this account can exist, so we don't need to check the PDA.
impl OutboundRateLimit {
    pub const SEED_PREFIX: &'static [u8] = b"outbound_rate_limit";
}
