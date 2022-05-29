use crate::*;

// use crate::non_fungible_token::token::TokenId;
use near_sdk::{assert_one_yocto, env, log, ext_contract, require, AccountId, Balance, Gas, Promise};


/// Trait used when it's desired to have a non-fungible token that has a
/// traditional escrow or approval system. This allows Alice to allow Bob
/// to take only the token with the unique identifier "19" but not others.
/// It should be noted that in the [core non-fungible token standard] there
/// is a method to do "transfer and call" which may be preferred over using
/// an approval management standard in certain use cases.
///
/// [approval management standard]: https://nomicon.io/Standards/NonFungibleToken/ApprovalManagement.html
/// [core non-fungible token standard]: https://nomicon.io/Standards/NonFungibleToken/Core.html
pub trait NonFungibleTokenApproval {
    /// Add an approved account for a specific token.
    ///
    /// Requirements
    /// * Caller of the method must attach a deposit of at least 1 yoctoⓃ for
    ///   security purposes
    /// * Contract MAY require caller to attach larger deposit, to cover cost of
    ///   storing approver data
    /// * Contract MUST panic if called by someone other than token owner
    /// * Contract MUST panic if addition would cause `nft_revoke_all` to exceed
    ///   single-block gas limit
    /// * Contract MUST increment approval ID even if re-approving an account
    /// * If successfully approved or if had already been approved, and if `msg` is
    ///   present, contract MUST call `nft_on_approve` on `account_id`. See
    ///   `nft_on_approve` description below for details.
    ///
    /// Arguments:
    /// * `token_id`: the token for which to add an approval
    /// * `account_id`: the account to add to `approvals`
    /// * `msg`: optional string to be passed to `nft_on_approve`
    ///
    /// Returns void, if no `msg` given. Otherwise, returns promise call to
    /// `nft_on_approve`, which can resolve with whatever it wants.
    fn nft_approve(
        &mut self,
        token_id: TokenId,
        account_id: AccountId,
        msg: Option<String>,
    ) -> Option<Promise>;

    /// Revoke an approved account for a specific token.
    ///
    /// Requirements
    /// * Caller of the method must attach a deposit of 1 yoctoⓃ for security
    ///   purposes
    /// * If contract requires >1yN deposit on `nft_approve`, contract
    ///   MUST refund associated storage deposit when owner revokes approval
    /// * Contract MUST panic if called by someone other than token owner
    ///
    /// Arguments:
    /// * `token_id`: the token for which to revoke an approval
    /// * `account_id`: the account to remove from `approvals`
    fn nft_revoke(&mut self, token_id: TokenId, account_id: AccountId);

    /// Revoke all approved accounts for a specific token.
    ///
    /// Requirements
    /// * Caller of the method must attach a deposit of 1 yoctoⓃ for security
    ///   purposes
    /// * If contract requires >1yN deposit on `nft_approve`, contract
    ///   MUST refund all associated storage deposit when owner revokes approvals
    /// * Contract MUST panic if called by someone other than token owner
    ///
    /// Arguments:
    /// * `token_id`: the token with approvals to revoke
    fn nft_revoke_all(&mut self, token_id: TokenId);

    /// Check if a token is approved for transfer by a given account, optionally
    /// checking an approval_id
    ///
    /// Arguments:
    /// * `token_id`: the token for which to revoke an approval
    /// * `approved_account_id`: the account to check the existence of in `approvals`
    /// * `approval_id`: an optional approval ID to check against current approval ID for given account
    ///
    /// Returns:
    /// if `approval_id` given, `true` if `approved_account_id` is approved with given `approval_id`
    /// otherwise, `true` if `approved_account_id` is in list of approved accounts
    fn nft_is_approved(
        &self,
        token_id: TokenId,
        approved_account_id: AccountId,
        approval_id: Option<u64>,
    ) -> bool;
  }

  /// Approval receiver is the trait for the method called (or attempted to be called) when an NFT contract adds an approval for an account.
pub trait NonFungibleTokenApprovalReceiver {
  /// Respond to notification that contract has been granted approval for a token.
  ///
  /// Notes
  /// * Contract knows the token contract ID from `predecessor_account_id`
  ///
  /// Arguments:
  /// * `token_id`: the token to which this contract has been granted approval
  /// * `owner_id`: the owner of the token
  /// * `approval_id`: the approval ID stored by NFT contract for this approval.
  ///   Expected to be a number within the 2^53 limit representable by JSON.
  /// * `msg`: specifies information needed by the approved contract in order to
  ///    handle the approval. Can indicate both a function to call and the
  ///    parameters to pass to that function.
  fn nft_on_approve(
      &mut self,
      token_id: TokenId,
      owner_id: AccountId,
      approval_id: u64,
      msg: String,
  ) -> near_sdk::PromiseOrValue<String>; // TODO: how to make "any"?
}

const GAS_FOR_NFT_APPROVE: Gas = Gas(15_000_000_000_000);
const NO_DEPOSIT: Balance = 0;

fn expect_token_found<T>(option: Option<T>) -> T {
    option.unwrap_or_else(|| env::panic_str("Token not found"))
}

fn expect_approval<T>(option: Option<T>) -> T {
    option.unwrap_or_else(|| env::panic_str("next_approval_by_id must be set for approval ext"))
}

#[ext_contract(ext_approval_receiver)]
pub trait NonFungibleTokenReceiver {
    fn nft_on_approve(
        &mut self,
        token_id: TokenId,
        owner_id: AccountId,
        approval_id: u64,
        msg: String,
    );
}

#[near_bindgen]
impl NonFungibleTokenApproval for Contract {

  #[payable]
    fn nft_approve(
        &mut self,
        token_id: TokenId,
        account_id: AccountId,
        msg: Option<String>,
    ) -> Option<Promise> {
        assert_at_least_one_yocto();
        let approvals_by_id = self
            .tokens
            .approvals_by_id
            .as_mut()
            .unwrap_or_else(|| env::panic_str("NFT does not support Approval Management"));

        let owner_id = expect_token_found(self.tokens.owner_by_id.get(&token_id));

        require!(env::predecessor_account_id() == owner_id, "Predecessor must be token owner.");

        let next_approval_id_by_id = expect_approval(self.tokens.next_approval_id_by_id.as_mut());
        // update HashMap of approvals for this token
        let approved_account_ids = &mut approvals_by_id.get(&token_id).unwrap_or_default();
        let approval_id: u64 = next_approval_id_by_id.get(&token_id).unwrap_or(1u64);
        let old_approval_id = approved_account_ids.insert(account_id.clone(), approval_id);

        // save updated approvals HashMap to contract's LookupMap
        approvals_by_id.insert(&token_id, approved_account_ids);

        // increment next_approval_id for this token
        next_approval_id_by_id.insert(&token_id, &(approval_id + 1));

        // If this approval replaced existing for same account, no storage was used.
        // Otherwise, require that enough deposit was attached to pay for storage, and refund
        // excess.
        let storage_used =
            if old_approval_id.is_none() { bytes_for_approved_account_id(&account_id) } else { 0 };
        refund_deposit(storage_used);

        // if given `msg`, schedule call to `nft_on_approve` and return it. Else, return None.
        msg.map(|msg| {
            ext_approval_receiver::nft_on_approve(
                token_id,
                owner_id,
                approval_id,
                msg,
                account_id,
                NO_DEPOSIT,
                env::prepaid_gas() - GAS_FOR_NFT_APPROVE,
            )
        })
    }

    #[payable]
    fn nft_revoke(&mut self, token_id: TokenId, account_id: AccountId) {
        assert_one_yocto();
        let approvals_by_id = self.tokens.approvals_by_id.as_mut().unwrap_or_else(|| {
            env::panic_str("NFT does not support Approval Management");
        });

        let owner_id = expect_token_found(self.tokens.owner_by_id.get(&token_id));
        let predecessor_account_id = env::predecessor_account_id();

        require!(predecessor_account_id == owner_id, "Predecessor must be token owner.");

        // if token has no approvals, do nothing
        if let Some(approved_account_ids) = &mut approvals_by_id.get(&token_id) {
            // if account_id was already not approved, do nothing
            if approved_account_ids.remove(&account_id).is_some() {
                refund_approved_account_ids_iter(
                    predecessor_account_id,
                    core::iter::once(&account_id),
                );
                // if this was the last approval, remove the whole HashMap to save space.
                if approved_account_ids.is_empty() {
                    approvals_by_id.remove(&token_id);
                } else {
                    // otherwise, update approvals_by_id with updated HashMap
                    approvals_by_id.insert(&token_id, approved_account_ids);
                }
            }
        }
    }

    #[payable]
    fn nft_revoke_all(&mut self, token_id: TokenId) {
        assert_one_yocto();
        let approvals_by_id = self.tokens.approvals_by_id.as_mut().unwrap_or_else(|| {
            env::panic_str("NFT does not support Approval Management");
        });

        let owner_id = expect_token_found(self.tokens.owner_by_id.get(&token_id));
        let predecessor_account_id = env::predecessor_account_id();

        require!(predecessor_account_id == owner_id, "Predecessor must be token owner.");

        // if token has no approvals, do nothing
        if let Some(approved_account_ids) = &mut approvals_by_id.get(&token_id) {
            // otherwise, refund owner for storage costs of all approvals...
            refund_approved_account_ids(predecessor_account_id, approved_account_ids);
            // ...and remove whole HashMap of approvals
            approvals_by_id.remove(&token_id);
        }
    }

    fn nft_is_approved(
        &self,
        token_id: TokenId,
        approved_account_id: AccountId,
        approval_id: Option<u64>,
    ) -> bool {
        expect_token_found(self.tokens.owner_by_id.get(&token_id));

        let approvals_by_id = if let Some(a) = self.tokens.approvals_by_id.as_ref() {
            a
        } else {
            // contract does not support approval management
            return false;
        };

        let approved_account_ids = if let Some(ids) = approvals_by_id.get(&token_id) {
            ids
        } else {
            // token has no approvals
            return false;
        };

        let actual_approval_id = if let Some(id) = approved_account_ids.get(&approved_account_id) {
            id
        } else {
            // account not in approvals HashMap
            return false;
        };

        if let Some(given_approval_id) = approval_id {
            &given_approval_id == actual_approval_id
        } else {
            // account approved, no approval_id given
            true
        }
    }
}