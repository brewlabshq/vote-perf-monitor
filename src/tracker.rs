use {
    chrono::{DateTime, Local},
    rustc_hash::{FxHashMap, FxHashSet},
    std::sync::Arc,
};

type Slot = u64;

pub const VOTE_CREDITS_GRACE_SLOTS: u8 = 2;
pub const VOTE_CREDITS_MAXIMUM_PER_SLOT: u8 = 16;
pub const BLUK_DUMP: u8 = 50;

/// pending vote awaiting confirmation in a finalized block
#[derive(Debug, Clone)]
pub struct PendingVote {
    pub signature: Arc<String>, // arc to avoid repeated allocations
    pub voted_slots: FxHashSet<Slot>,
    pub transaction_slot: Slot,
    pub timestamp: DateTime<Local>,
    pub instruction_data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct ConfirmVote {
    pub signature: String,
    pub voted_slot: Slot,
    pub finalized_slot: Slot,
    pub tvc_credits: u64,
    pub latency: u64,
    pub timestamp: DateTime<Local>,
}

#[derive(Debug)]
pub struct VoteTracker {
    // cleanup after its commited to the db
    last_cleanup_slot: Slot,

    // Sig -> Vote
    pending_votes: FxHashMap<Arc<String>, PendingVote>,

    confirm_votes: FxHashMap<String, ConfirmVote>,
}

impl VoteTracker {
    pub fn new() -> Self {
        Self {
            last_cleanup_slot: 0,
            pending_votes: FxHashMap::with_capacity_and_hasher(1000, Default::default()),

            confirm_votes: FxHashMap::with_capacity_and_hasher(1000, Default::default()),
        }
    }
    pub fn add_pending_vote(&mut self, vote: PendingVote) {
        self.pending_votes.insert(vote.signature.clone(), vote);
    }

    // pub fn confirm_votes(
    //     &mut self,
    //     signature: String,
    //     finalized_slot: Slot,
    //     voted_slot: Slot,
    // ) -> Option<ConfirmVote> {
    //     if finalized_slot < voted_slot {
    //         tracing::warn!(
    //             "invalid slot order: finalized_slot {} < voted_slot {}",
    //             finalized_slot,
    //             voted_slot
    //         );
    //         return None;
    //     }

    //     if let Some(pending) = self.pending_votes.get(&signature) {
    //         if pending.voted_slots.contains(&voted_slot) {

    //             let confirm_vote: ConfirmVote {}

    //             Some(confirm_vote)
    //         } else {
    //             None
    //         }
    //     } else {
    //         // direct confirmation

    //         let confirm_vote: ConfirmVote {

    //         }
    //     }
    // }
}
