// TAO ID Generator - Snowflake-like IDs with embedded shard information
// Based on Meta's TAO ID scheme: 64-bit IDs with shard routing

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// TAO ID Generator following Meta's pattern
/// 64-bit ID format: [timestamp:42][shard_id:10][sequence:12]
/// This allows for 1024 shards and 4096 IDs per millisecond per shard
#[derive(Debug)]
pub struct TaoIdGenerator {
    shard_id: u16,
    sequence: AtomicU64,
    last_timestamp: AtomicU64,
}

impl TaoIdGenerator {
    /// Create new ID generator for given shard
    pub fn new(shard_id: u16) -> Self {
        assert!(shard_id < 1024, "Shard ID must be less than 1024");

        Self {
            shard_id,
            sequence: AtomicU64::new(0),
            last_timestamp: AtomicU64::new(0),
        }
    }

    /// Generate next unique ID with embedded shard information
    pub fn next_id(&self) -> i64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let last_ts = self.last_timestamp.load(Ordering::Relaxed);

        let sequence = if now == last_ts {
            // Same millisecond - increment sequence
            let seq = self.sequence.fetch_add(1, Ordering::Relaxed);
            if seq >= 4096 {
                // Sequence overflow - wait for next millisecond
                std::thread::sleep(std::time::Duration::from_millis(1));
                self.sequence.store(0, Ordering::Relaxed);
                return self.next_id();
            }
            seq
        } else {
            // New millisecond - reset sequence
            self.last_timestamp.store(now, Ordering::Relaxed);
            self.sequence.store(1, Ordering::Relaxed);
            0
        };

        // Construct 64-bit ID: [timestamp:42][shard_id:10][sequence:12]
        let id = ((now & 0x3FFFFFFFFFF) << 22) |    // 42 bits timestamp
                 ((self.shard_id as u64) << 12) |   // 10 bits shard_id
                 (sequence & 0xFFF); // 12 bits sequence

        id as i64
    }

    /// Extract shard ID from TAO ID
    pub fn extract_shard_id(id: i64) -> u16 {
        ((id as u64) >> 12 & 0x3FF) as u16
    }

    /// Extract timestamp from TAO ID
    pub fn extract_timestamp(id: i64) -> u64 {
        (id as u64) >> 22
    }

    /// Extract sequence from TAO ID
    pub fn extract_sequence(id: i64) -> u16 {
        ((id as u64) & 0xFFF) as u16
    }

    /// Get current shard ID
    pub fn shard_id(&self) -> u16 {
        self.shard_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_generation() {
        let generator = TaoIdGenerator::new(123);

        // Generate multiple IDs
        let id1 = generator.next_id();
        let id2 = generator.next_id();
        let id3 = generator.next_id();

        // IDs should be unique
        assert_ne!(id1, id2);
        assert_ne!(id2, id3);

        // All should have same shard ID
        assert_eq!(TaoIdGenerator::extract_shard_id(id1), 123);
        assert_eq!(TaoIdGenerator::extract_shard_id(id2), 123);
        assert_eq!(TaoIdGenerator::extract_shard_id(id3), 123);

        // Sequences should increment
        let seq1 = TaoIdGenerator::extract_sequence(id1);
        let seq2 = TaoIdGenerator::extract_sequence(id2);
        let seq3 = TaoIdGenerator::extract_sequence(id3);

        assert_eq!(seq1, 0);
        assert_eq!(seq2, 1);
        assert_eq!(seq3, 2);
    }

    #[test]
    fn test_shard_extraction() {
        let generator = TaoIdGenerator::new(500);
        let id = generator.next_id();

        assert_eq!(TaoIdGenerator::extract_shard_id(id), 500);
        assert_eq!(generator.shard_id(), 500);
    }
}
