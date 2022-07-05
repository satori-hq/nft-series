use near_sdk::{env, require, AccountId, Balance, CryptoHash, Promise};
use near_sdk::json_types::{U128};
use std::collections::HashMap;
use std::mem::size_of;

// TODO: need a way for end users to determine how much an approval will cost.
pub fn bytes_for_approved_account_id(account_id: &AccountId) -> u64 {
    // The extra 4 bytes are coming from Borsh serialization to store the length of the string.
    account_id.as_str().len() as u64 + 4 + size_of::<u64>() as u64
}

pub fn refund_approved_account_ids_iter<'a, I>(
    account_id: AccountId,
    approved_account_ids: I,
) -> Promise
where
    I: Iterator<Item = &'a AccountId>,
{
    let storage_released: u64 = approved_account_ids.map(bytes_for_approved_account_id).sum();
    Promise::new(account_id).transfer(Balance::from(storage_released) * env::storage_byte_cost())
}

pub fn refund_approved_account_ids(
    account_id: AccountId,
    approved_account_ids: &HashMap<AccountId, u64>,
) -> Promise {
    refund_approved_account_ids_iter(account_id, approved_account_ids.keys())
}

/// from https://github.com/near/near-sdk-rs/blob/e4abb739ff953b06d718037aa1b8ab768db17348/near-contract-standards/src/non_fungible_token/utils.rs#L29
pub fn refund_deposit(storage_used: u64) {
    let required_cost = env::storage_byte_cost() * Balance::from(storage_used);
    let attached_deposit = env::attached_deposit();

    require!(
        required_cost <= attached_deposit,
        format!("Must attach {} yoctoNEAR to cover storage", required_cost)
    );

    let refund = attached_deposit - required_cost;
    if refund > 1 {
        Promise::new(env::predecessor_account_id()).transfer(refund);
    }
}

pub fn hash_account_id(account_id: &AccountId) -> CryptoHash {
    let mut hash = CryptoHash::default();
    hash.copy_from_slice(&env::sha256(account_id.as_bytes()));
    hash
}

/// Assert that at least 1 yoctoNEAR was attached.
pub(crate) fn assert_at_least_one_yocto() {
    require!(env::attached_deposit() >= 1, "Requires attached deposit of at least 1 yoctoNEAR")
}

/// convert the royalty percentage and amount to pay into a payout (U128)
pub(crate) fn royalty_to_payout(royalty_percentage: u32, amount_to_pay: Balance) -> U128 {
    U128(royalty_percentage as u128 * amount_to_pay / 10_000u128)
}

pub(crate) fn random_u128() -> u128 {
    let random_seed = env::random_seed(); // len 32
    // using first 16 bytes (doesn't affect randomness)
    as_u128(random_seed.get(..16).unwrap())
}

fn as_u128(arr: &[u8]) -> u128 {
    ((arr[0] as u128) << 0) +
    ((arr[1] as u128) << 8) +
    ((arr[2] as u128) << 16) +
    ((arr[3] as u128) << 24)
    // ((arr[4] as u128) << 32) +
    // ((arr[5] as u128) << 40) +
    // ((arr[6] as u128) << 48) +
    // ((arr[7] as u128) << 56) +
    // ((arr[8] as u128) << 64) +
    // ((arr[9] as u128) << 72) +
    // ((arr[10] as u128) << 80) +
    // ((arr[11] as u128) << 88) +
    // ((arr[12] as u128) << 96) +
    // ((arr[13] as u128) << 104) +
    // ((arr[14] as u128) << 112) +
    // ((arr[15] as u128) << 120)
}