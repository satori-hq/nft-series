use crate::*;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, TreeMap, UnorderedSet};
use near_sdk::{
    assert_one_yocto, env, ext_contract, log, require, AccountId, Balance,
    Gas, IntoStorageKey, PromiseOrValue, PromiseResult, StorageUsage,
};
use std::collections::HashMap;

const GAS_FOR_RESOLVE_TRANSFER: Gas = Gas(5_000_000_000_000);
const GAS_FOR_FT_TRANSFER_CALL: Gas = Gas(25_000_000_000_000 + GAS_FOR_RESOLVE_TRANSFER.0);

const NO_DEPOSIT: Balance = 0;

/// Used for all non-fungible tokens. The specification for the
/// [core non-fungible token standard] lays out the reasoning for each method.
/// It's important to check out [NonFungibleTokenReceiver](crate::non_fungible_token::core::NonFungibleTokenReceiver)
/// and [NonFungibleTokenResolver](crate::non_fungible_token::core::NonFungibleTokenResolver) to
/// understand how the cross-contract call work.
///
/// [core non-fungible token standard]: https://nomicon.io/Standards/NonFungibleToken/Core.html
pub trait NonFungibleTokenCore {
  /// Simple transfer. Transfer a given `token_id` from current owner to
  /// `receiver_id`.
  ///
  /// Requirements
  /// * Caller of the method must attach a deposit of 1 yoctoⓃ for security purposes
  /// * Contract MUST panic if called by someone other than token owner or,
  ///   if using Approval Management, one of the approved accounts
  /// * `approval_id` is for use with Approval Management,
  ///   see https://nomicon.io/Standards/NonFungibleToken/ApprovalManagement.html
  /// * If using Approval Management, contract MUST nullify approved accounts on
  ///   successful transfer.
  /// * TODO: needed? Both accounts must be registered with the contract for transfer to
  ///   succeed. See see https://nomicon.io/Standards/StorageManagement.html
  ///
  /// Arguments:
  /// * `receiver_id`: the valid NEAR account receiving the token
  /// * `token_id`: the token to transfer
  /// * `approval_id`: expected approval ID. A number smaller than
  ///    2^53, and therefore representable as JSON. See Approval Management
  ///    standard for full explanation.
  /// * `memo` (optional): for use cases that may benefit from indexing or
  ///    providing information for a transfer
  fn nft_transfer(
      &mut self,
      receiver_id: AccountId,
      token_id: TokenId,
      approval_id: Option<u64>,
      memo: Option<String>,
  );

  /// Transfer token and call a method on a receiver contract. A successful
  /// workflow will end in a success execution outcome to the callback on the NFT
  /// contract at the method `nft_resolve_transfer`.
  ///
  /// You can think of this as being similar to attaching native NEAR tokens to a
  /// function call. It allows you to attach any Non-Fungible Token in a call to a
  /// receiver contract.
  ///
  /// Requirements:
  /// * Caller of the method must attach a deposit of 1 yoctoⓃ for security
  ///   purposes
  /// * Contract MUST panic if called by someone other than token owner or,
  ///   if using Approval Management, one of the approved accounts
  /// * The receiving contract must implement `ft_on_transfer` according to the
  ///   standard. If it does not, FT contract's `ft_resolve_transfer` MUST deal
  ///   with the resulting failed cross-contract call and roll back the transfer.
  /// * Contract MUST implement the behavior described in `ft_resolve_transfer`
  /// * `approval_id` is for use with Approval Management extension, see
  ///   that document for full explanation.
  /// * If using Approval Management, contract MUST nullify approved accounts on
  ///   successful transfer.
  ///
  /// Arguments:
  /// * `receiver_id`: the valid NEAR account receiving the token.
  /// * `token_id`: the token to send.
  /// * `approval_id`: expected approval ID. A number smaller than
  ///    2^53, and therefore representable as JSON. See Approval Management
  ///    standard for full explanation.
  /// * `memo` (optional): for use cases that may benefit from indexing or
  ///    providing information for a transfer.
  /// * `msg`: specifies information needed by the receiving contract in
  ///    order to properly handle the transfer. Can indicate both a function to
  ///    call and the parameters to pass to that function.
  fn nft_transfer_call(
      &mut self,
      receiver_id: AccountId,
      token_id: TokenId,
      approval_id: Option<u64>,
      memo: Option<String>,
      msg: String,
  ) -> PromiseOrValue<bool>;

  /// Returns the token with the given `token_id` or `null` if no such token.
    fn nft_token(&self, token_id: TokenId) -> Option<Token>;
}

#[ext_contract(ext_self)]
trait NFTResolver {
    fn nft_resolve_transfer(
        &mut self,
        previous_owner_id: AccountId,
        receiver_id: AccountId,
        token_id: TokenId,
        approved_account_ids: Option<HashMap<AccountId, u64>>,
    ) -> bool;
}

trait NonFungibleTokenResolver {
  /*
      resolves the promise of the cross contract call to the receiver contract
      this is stored on THIS contract and is meant to analyze what happened in the cross contract call when nft_on_transfer was called
      as part of the nft_transfer_call method
  */
  fn nft_resolve_transfer(
    &mut self,
    previous_owner_id: AccountId,
    receiver_id: AccountId,
    token_id: TokenId,
    approved_account_ids: Option<HashMap<AccountId, u64>>,
) -> bool;
}

#[ext_contract(ext_receiver)]
pub trait NonFungibleTokenReceiver {
    /// Returns true if token should be returned to `sender_id`
    fn nft_on_transfer(
        &mut self,
        sender_id: AccountId,
        previous_owner_id: AccountId,
        token_id: TokenId,
        msg: String,
    ) -> PromiseOrValue<bool>;
}

/// NEW Implementation of the non-fungible token standard.
/// Allows to include NEP-171 compatible token to any contract.
/// There are next traits that any contract may implement:
///     - NonFungibleTokenCore -- interface with nft_transfer methods. NonFungibleToken provides methods for it.
///     - NonFungibleTokenApproval -- interface with nft_approve methods. NonFungibleToken provides methods for it.
///     - NonFungibleTokenEnumeration -- interface for getting lists of tokens. NonFungibleToken provides methods for it.
///     - NonFungibleTokenMetadata -- return metadata for the token in NEP-177, up to contract to implement.
///
/// For example usage, see examples/non-fungible-token/src/lib.rs.
#[derive(BorshDeserialize, BorshSerialize)]
pub struct NonFungibleTokenV1 { // OLD
    // owner of contract
    pub owner_id: AccountId,

    // The storage size in bytes for each new token
    pub extra_storage_in_bytes_per_token: StorageUsage,

    // always required
    pub owner_by_id: TreeMap<TokenId, AccountId>,

    // required by metadata extension
    pub token_metadata_by_id: Option<LookupMap<TokenId, TokenMetadataV1>>, // OLD TOKEN METADATA

    // required by enumeration extension
    pub tokens_per_owner: Option<LookupMap<AccountId, UnorderedSet<TokenId>>>,

    // required by approval extension
    pub approvals_by_id: Option<LookupMap<TokenId, HashMap<AccountId, u64>>>,
    pub next_approval_id_by_id: Option<LookupMap<TokenId, u64>>,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct NonFungibleToken { // CURRENT
    // owner of contract
    pub owner_id: AccountId,

    // The storage size in bytes for each new token
    pub extra_storage_in_bytes_per_token: StorageUsage,

    // always required
    pub owner_by_id: TreeMap<TokenId, AccountId>,

    // required by metadata extension
    pub token_metadata_by_id: Option<LookupMap<TokenId, VersionedTokenMetadata>>, // CURRENT TOKEN METADATA

    // required by enumeration extension
    pub tokens_per_owner: Option<LookupMap<AccountId, UnorderedSet<TokenId>>>,

    // required by approval extension
    pub approvals_by_id: Option<LookupMap<TokenId, HashMap<AccountId, u64>>>,
    pub next_approval_id_by_id: Option<LookupMap<TokenId, u64>>,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum VersionedNonFungibleToken {
    Current(NonFungibleToken),
}

impl NonFungibleTokenV1 {
    pub fn new<Q, R, S, T>(
        owner_by_id_prefix: Q,
        owner_id: AccountId,
        token_metadata_prefix: Option<R>,
        enumeration_prefix: Option<S>,
        approval_prefix: Option<T>,
    ) -> Self
    where
        Q: IntoStorageKey,
        R: IntoStorageKey,
        S: IntoStorageKey,
        T: IntoStorageKey,
    {
        let (approvals_by_id, next_approval_id_by_id) = if let Some(prefix) = approval_prefix {
            let prefix: Vec<u8> = prefix.into_storage_key();
            (
                Some(LookupMap::new(prefix.clone())),
                Some(LookupMap::new([prefix, "n".into()].concat())),
            )
        } else {
            (None, None)
        };

        Self {
            owner_id,
            extra_storage_in_bytes_per_token: 0,
            owner_by_id: TreeMap::new(owner_by_id_prefix),
            token_metadata_by_id: token_metadata_prefix.map(LookupMap::new),
            tokens_per_owner: enumeration_prefix.map(LookupMap::new),
            approvals_by_id,
            next_approval_id_by_id,
        }
    }
}

impl NonFungibleToken {
    pub fn new<Q, R, S, T>(
        owner_by_id_prefix: Q,
        owner_id: AccountId,
        token_metadata_prefix: Option<R>,
        enumeration_prefix: Option<S>,
        approval_prefix: Option<T>,
        ) -> Self
        where
            Q: IntoStorageKey,
            R: IntoStorageKey,
            S: IntoStorageKey,
            T: IntoStorageKey,
        {
        let (approvals_by_id, next_approval_id_by_id) = if let Some(prefix) = approval_prefix {
            let prefix: Vec<u8> = prefix.into_storage_key();
            (
                Some(LookupMap::new(prefix.clone())),
                Some(LookupMap::new([prefix, "n".into()].concat())),
            )
        } else {
            (None, None)
        };

        let mut this = Self {
            owner_id,
            extra_storage_in_bytes_per_token: 0,
            owner_by_id: TreeMap::new(owner_by_id_prefix),
            token_metadata_by_id: token_metadata_prefix.map(LookupMap::new),
            tokens_per_owner: enumeration_prefix.map(LookupMap::new),
            approvals_by_id,
            next_approval_id_by_id,
        };
        this.measure_min_token_storage_cost();
        this
    }

    // TODO: does this seem reasonable?
    fn measure_min_token_storage_cost(&mut self) {
        let initial_storage_usage = env::storage_usage();
        let tmp_token_id = "a".repeat(64); // TODO: what's a reasonable max TokenId length?
        let tmp_owner_id = AccountId::new_unchecked("a".repeat(64));

        // 1. set some dummy data
        self.owner_by_id.insert(&tmp_token_id, &tmp_owner_id);
        if let Some(token_metadata_by_id) = &mut self.token_metadata_by_id {
            let token = TokenMetadata {
                title: Some("a".repeat(64)),
                description: Some("a".repeat(64)),
                media: Some("a".repeat(64)),
                copies: Some(1),
                asset_id: Some(String::from("1")),
                filetype: Some(String::from("jpg")),
                extra: Some(String::from("1.json")),
            };
            token_metadata_by_id.insert(
                &tmp_token_id,
                &VersionedTokenMetadata::from(VersionedTokenMetadata::Current(token)),
                // &token,
            );
        }
        if let Some(tokens_per_owner) = &mut self.tokens_per_owner {
            let u = &mut UnorderedSet::new(StorageKey::TokensPerOwner {
                account_hash: env::sha256(tmp_owner_id.as_bytes()),
            });
            u.insert(&tmp_token_id);
            tokens_per_owner.insert(&tmp_owner_id, u);
        }
        if let Some(approvals_by_id) = &mut self.approvals_by_id {
            let mut approvals = HashMap::new();
            approvals.insert(tmp_owner_id.clone(), 1u64);
            approvals_by_id.insert(&tmp_token_id, &approvals);
        }
        if let Some(next_approval_id_by_id) = &mut self.next_approval_id_by_id {
            next_approval_id_by_id.insert(&tmp_token_id, &1u64);
        }
        let u = UnorderedSet::new(
            StorageKey::TokenPerOwnerInner { account_id_hash: hash_account_id(&tmp_owner_id) }
                .try_to_vec()
                .unwrap(),
        );
        if let Some(tokens_per_owner) = &mut self.tokens_per_owner {
            tokens_per_owner.insert(&tmp_owner_id, &u);
        }

        // 2. see how much space it took
        self.extra_storage_in_bytes_per_token = env::storage_usage() - initial_storage_usage;

        // 3. roll it all back
        if let Some(next_approval_id_by_id) = &mut self.next_approval_id_by_id {
            next_approval_id_by_id.remove(&tmp_token_id);
        }
        if let Some(approvals_by_id) = &mut self.approvals_by_id {
            approvals_by_id.remove(&tmp_token_id);
        }
        if let Some(tokens_per_owner) = &mut self.tokens_per_owner {
            tokens_per_owner.remove(&tmp_owner_id);
        }
        if let Some(token_metadata_by_id) = &mut self.token_metadata_by_id {
            token_metadata_by_id.remove(&tmp_token_id);
        }
        if let Some(tokens_per_owner) = &mut self.tokens_per_owner {
            tokens_per_owner.remove(&tmp_owner_id);
        }
        self.owner_by_id.remove(&tmp_token_id);
    }

	/// Transfer token_id from `from` to `to`
	///
	/// Do not perform any safety checks or do any logging
	pub fn internal_transfer_unguarded(
		&mut self,
		#[allow(clippy::ptr_arg)] token_id: &TokenId,
		from: &AccountId,
		to: &AccountId,
	    ) {
			// update owner
			self.owner_by_id.insert(token_id, to);

			// if using Enumeration standard, update old & new owner's token lists
			if let Some(tokens_per_owner) = &mut self.tokens_per_owner {
			// owner_tokens should always exist, so call `unwrap` without guard
			let mut owner_tokens = tokens_per_owner.get(from).unwrap_or_else(|| {
					env::panic_str("Unable to access tokens per owner in unguarded call.")
			});
			owner_tokens.remove(token_id);
			if owner_tokens.is_empty() {
					tokens_per_owner.remove(from);
			} else {
					tokens_per_owner.insert(from, &owner_tokens);
			}

			let mut receiver_tokens = tokens_per_owner.get(to).unwrap_or_else(|| {
					UnorderedSet::new(StorageKey::TokensPerOwner {
							account_hash: env::sha256(to.as_bytes()),
					})
			});
			receiver_tokens.insert(token_id);
			tokens_per_owner.insert(to, &receiver_tokens);
		}
	}

    /// Transfer from current owner to receiver_id, checking that sender is allowed to transfer.
    /// Clear approvals, if approval extension being used.
    /// Return previous owner and approvals.
    pub fn internal_transfer(
        &mut self,
        sender_id: &AccountId,
        receiver_id: &AccountId,
        #[allow(clippy::ptr_arg)] token_id: &TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
        ) -> (AccountId, Option<HashMap<AccountId, u64>>) {
        let owner_id = self.owner_by_id.get(token_id).unwrap_or_else(|| env::panic_str("Token not found"));

        // clear approvals, if using Approval Management extension
        // this will be rolled back by a panic if sending fails
        let approved_account_ids = self.approvals_by_id.as_mut().and_then(|by_id| by_id.remove(token_id));

        // check if authorized
        let sender_id = if sender_id != &owner_id {
            // if approval extension is NOT being used, or if token has no approved accounts
            let app_acc_ids = approved_account_ids.as_ref().unwrap_or_else(|| env::panic_str("Unauthorized"));

            // Approval extension is being used; get approval_id for sender.
            let actual_approval_id = app_acc_ids.get(sender_id);

            // Panic if sender not approved at all
            if actual_approval_id.is_none() {
                env::panic_str("Sender not approved");
            }

            // If approval_id included, check that it matches
            require!(
                approval_id.is_none() || actual_approval_id == approval_id.as_ref(),
                format!(
                        "The actual approval_id {:?} is different from the given approval_id {:?}",
                        actual_approval_id, approval_id
                )
            );
            Some(sender_id)
        } else {
            None
        };

        require!(&owner_id != receiver_id, "Current and next owner must differ");

        self.internal_transfer_unguarded(token_id, &owner_id, receiver_id);

        // NonFungibleToken::emit_transfer(&owner_id, receiver_id, token_id, sender_id, memo);
        env::log_str(format!("{}{}", EVENT_JSON, json!({
            "standard": "nep171",
            "version": "1.0.0",
            "event": "nft_transfer",
            "data": [
                {
                    "old_owner_id": owner_id, "new_owner_id": receiver_id, "token_ids": [token_id]
                }
            ]
        })).as_ref());

        // return previous owner & approvals
        (owner_id, approved_account_ids)
    }

    /// Mint a new token without checking whether the caller id is equal to the `owner_id`
    pub fn internal_mint(
        &mut self,
        token_id: TokenId,
        token_owner_id: AccountId,
        token_metadata: Option<VersionedTokenMetadata>,
    ) -> Token {
        let initial_storage_usage = env::storage_usage();
        if self.token_metadata_by_id.is_some() && token_metadata.is_none() {
            env::panic_str("Must provide metadata");
        }
        if self.owner_by_id.get(&token_id).is_some() {
            env::panic_str("token_id must be unique");
        }

        let owner_id: AccountId = token_owner_id;

        // Core behavior: every token must have an owner
        self.owner_by_id.insert(&token_id, &owner_id);

        // Metadata extension: Save metadata, keep variable around to return later.
        // Note that check above already panicked if metadata extension in use but no metadata
        // provided to call.
        self.token_metadata_by_id
            .as_mut()
            // .and_then(|by_id| by_id.insert(&token_id, &VersionedTokenMetadata::from(VersionedTokenMetadata::Current(token_metadata.unwrap()))));
            .and_then(|by_id| by_id.insert(&token_id, token_metadata.as_ref().unwrap()));

        // Enumeration extension: Record tokens_per_owner for use with enumeration view methods.
        if let Some(tokens_per_owner) = &mut self.tokens_per_owner {
            let mut token_ids = tokens_per_owner.get(&owner_id).unwrap_or_else(|| {
                UnorderedSet::new(StorageKey::TokensPerOwner {
                    account_hash: env::sha256(owner_id.as_bytes()),
                })
            });
            token_ids.insert(&token_id);
            tokens_per_owner.insert(&owner_id, &token_ids);
        }

        // Approval Management extension: return empty HashMap as part of Token
        let approved_account_ids =
            if self.approvals_by_id.is_some() { Some(HashMap::new()) } else { None };

        // Return any extra attached deposit not used for storage
        refund_deposit(env::storage_usage() - initial_storage_usage);

        let token = Token { token_id, owner_id, metadata: Some(versioned_token_metadata_to_token_metadata(token_metadata.unwrap())), approved_account_ids };

        token
    }
}

#[near_bindgen]
impl NonFungibleTokenCore for Contract {

    #[payable]
	fn nft_transfer(
		&mut self,
		receiver_id: AccountId,
		token_id: TokenId,
		approval_id: Option<u64>,
		memo: Option<String>,
	    ) {
		assert_one_yocto();
		let sender_id = env::predecessor_account_id();
		self.tokens_mut().internal_transfer(&sender_id, &receiver_id, &token_id, approval_id, memo);
	}

    #[payable]
    fn nft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
        msg: String,
        ) -> PromiseOrValue<bool> {
        assert_one_yocto();
        let sender_id = env::predecessor_account_id();
        let (old_owner, old_approvals) = self.tokens_mut().internal_transfer(&sender_id, &receiver_id, &token_id, approval_id, memo);
        // Initiating receiver's call and the callback
        ext_receiver::nft_on_transfer(
            sender_id,
            old_owner.clone(),
            token_id.clone(),
            msg,
            receiver_id.clone(),
            NO_DEPOSIT,
            env::prepaid_gas() - GAS_FOR_FT_TRANSFER_CALL,
        )
        .then(ext_self::nft_resolve_transfer(
            old_owner,
            receiver_id,
            token_id,
            old_approvals,
            env::current_account_id(),
            NO_DEPOSIT,
            GAS_FOR_RESOLVE_TRANSFER,
        ))
        .into()
    }

	fn nft_token(&self, token_id: TokenId) -> Option<Token> {
        let tokens = self.tokens();
		let owner_id = tokens.owner_by_id.get(&token_id)?;
        let approved_account_ids = tokens
            .approvals_by_id
			.as_ref()
            .and_then(|by_id| by_id.get(&token_id).or_else(|| Some(HashMap::new())));

		// CUSTOM (switch metadata for the token_type metadata)
		let mut token_id_iter = token_id.split(TOKEN_DELIMETER);
		let token_type_id = token_id_iter.next().unwrap().parse().unwrap();
		// make edition titles nice for showing in wallet
        let token_type = self.token_type_by_id.get(&token_type_id).unwrap();
        // let mut final_metadata = token_type.metadata;
		let mut final_metadata = TokenMetadata {
            title: token_type.metadata.title,
            description: token_type.metadata.description,
            media: token_type.metadata.media,
            copies: token_type.metadata.copies,
            asset_id: None,
            filetype: None,
            extra: None,
        };
		// let copies = final_metadata.copies;
		if let Some(copies) = final_metadata.copies {
			final_metadata.title = Some(
				format!(
					"{}{}{}{}{}",
					final_metadata.title.unwrap(),
					TITLE_DELIMETER,
					token_id_iter.next().unwrap(),
					EDITION_DELIMETER,
					copies
				)
			);
		}

        let token_metadata_versioned = tokens.token_metadata_by_id.as_ref().unwrap().get(&token_id).unwrap();
        let token_metadata = versioned_token_metadata_to_token_metadata(token_metadata_versioned);
        let asset_id = &token_metadata.asset_id;
        let filetype = &token_metadata.filetype;
        let extra = &token_metadata.extra;
        let media = final_metadata.clone().media.unwrap();
        if asset_id.is_some() && filetype.is_some() {
            // older NFTs (pre-generative upgrade c. 6/15/22) won't have asset_id or file_type
            // media cid for this series + asset token ID + filetype maps to a media asset on IPFS
            final_metadata.media = Some(format!("{}/{}.{}", media.clone(), asset_id.clone().unwrap(), filetype.clone().unwrap()));
        }
        if extra.is_some() {
            // media cid for this series + asset token ID + .json maps to a json asset on IPFS
            final_metadata.extra = Some(format!("{}/{}.json", media.clone(), asset_id.clone().unwrap()));
        }
		
		// CUSTOM
		// implement this if you need to combine individual token metadata
		// e.g. metadata.extra with TokenType.metadata.extra and return something unique
		// let token_metadata = self.tokens.token_metadata_by_id.get(&token_id)?;
		// metadata.extra = token_metadata.extra;
        let token = Token {
            token_id,
            owner_id,
            metadata: Some(final_metadata),
            approved_account_ids,
        };
        Some(token)
	}
}

impl NonFungibleTokenResolver for NonFungibleToken {
    /// Returns true if token was successfully transferred to `receiver_id`.
    fn nft_resolve_transfer(
        &mut self,
        previous_owner_id: AccountId,
        receiver_id: AccountId,
        token_id: TokenId,
        approved_account_ids: Option<HashMap<AccountId, u64>>,
    ) -> bool {
        // Get whether token should be returned
        let must_revert = match env::promise_result(0) {
            PromiseResult::NotReady => env::abort(),
            PromiseResult::Successful(value) => {
                if let Ok(yes_or_no) = near_sdk::serde_json::from_slice::<bool>(&value) {
                    yes_or_no
                } else {
                    true
                }
            }
            PromiseResult::Failed => true,
        };

        // if call succeeded, return early
        if !must_revert {
            return true;
        }

        // OTHERWISE, try to set owner back to previous_owner_id and restore approved_account_ids

        // Check that receiver didn't already transfer it away or burn it.
        if let Some(current_owner) = self.owner_by_id.get(&token_id) {
            if current_owner != receiver_id {
                // The token is not owned by the receiver anymore. Can't return it.
                return true;
            }
        } else {
            // The token was burned and doesn't exist anymore.
            // Refund storage cost for storing approvals to original owner and return early.
            if let Some(approved_account_ids) = approved_account_ids {
                refund_approved_account_ids(previous_owner_id, &approved_account_ids);
            }
            return true;
        };

        log!("Return token {} from @{} to @{}", token_id, receiver_id, previous_owner_id);

        self.internal_transfer_unguarded(&token_id, &receiver_id, &previous_owner_id);

        // If using Approval Management extension,
        // 1. revert any approvals receiver already set, refunding storage costs
        // 2. reset approvals to what previous owner had set before call to nft_transfer_call
        if let Some(by_id) = &mut self.approvals_by_id {
            if let Some(receiver_approvals) = by_id.get(&token_id) {
                refund_approved_account_ids(receiver_id, &receiver_approvals);
            }
            if let Some(previous_owner_approvals) = approved_account_ids {
                by_id.insert(&token_id, &previous_owner_approvals);
            }
        }

        false
    }
}
