use anchor_lang::{prelude::*, solana_program::clock::UnixTimestamp};

#[account]
#[derive(InitSpace)]
pub struct RateLimitState {
    /// The maximum capacity of the rate limiter.
    limit: u64,
    /// The capacity of the rate limiter at `last_tx_timestamp`.
    /// The actual current capacity is calculated in `capacity_at`, by
    /// accounting for the time that has passed since `last_tx_timestamp` and
    /// the refill rate.
    capacity_at_last_tx: u64,
    /// The timestamp of the last transaction that counted towards the current
    /// capacity. Transactions that exceeded the capacity do not count, they are
    /// just delayed.
    last_tx_timestamp: i64,
    /// The rate per second at which the capacity is refilled.
    refill_rate: u64,
}

impl RateLimitState {
    pub const RATE_LIMIT_DURATION: i64 = 60 * 60 * 24; // 24 hours

    /// Returns the capacity of the rate limiter at the given timestamp.
    pub fn capacity_at(&self, now: UnixTimestamp) -> u64 {
        assert!(self.last_tx_timestamp <= now);

        let calculated_capacity = {
            let time_passed = (now - self.last_tx_timestamp) as u64;
            self.capacity_at_last_tx + (time_passed * self.refill_rate)
        };

        calculated_capacity.min(self.limit)
    }

    /// Computes the timestamp at which the given amount can be consumed.
    /// If it fits within the current capacity, the current timestamp is
    /// returned, and the remaining capacity is reduced.
    /// Otherwise, the timestamp at which the capacity will be available is
    /// returned.
    pub fn consume_or_delay(&mut self, now: UnixTimestamp, amount: u64) -> UnixTimestamp {
        let capacity = self.capacity_at(now);
        if capacity >= amount {
            self.capacity_at_last_tx = capacity - amount;
            self.last_tx_timestamp = now;
            now
        } else {
            now + Self::RATE_LIMIT_DURATION
        }
    }
}
