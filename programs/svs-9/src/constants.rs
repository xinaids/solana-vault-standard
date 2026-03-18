pub const ALLOCATOR_VAULT_SEED: &[u8] = b"allocator_vault";
pub const CHILD_ALLOCATION_SEED: &[u8] = b"child_allocation";
pub const SHARES_SEED: &[u8] = b"shares";
pub const IDLE_VAULT_SEED: &[u8] = b"idle_vault";
pub const MAX_CHILDREN: u8 = 10;
pub const BPS_DENOMINATOR: u16 = 10_000;
pub const MIN_DEPOSIT: u64 = 1_000;
// SVS-1 vault discriminator for child validation
pub const SVS1_VAULT_DISCRIMINATOR: [u8; 8] = [211, 8, 232, 43, 2, 152, 117, 119];
