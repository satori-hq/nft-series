use crate::*;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{PanicOnDefault};

/// Contract source metadata structure
/// As per NEP 0330 (https://github.com/near/NEPs/blob/master/neps/nep-0330.md), with addition of `commit_hash`
#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, PanicOnDefault)]
#[serde(crate = "near_sdk::serde")]
pub struct ContractSourceMetadata {
  /// e.g. "1.0.0" (for internal use)
	pub version: Option<String>,
  /// Git commit hash of currently deployed contract code
  pub commit_hash: Option<String>,
  /// GitHub repo url for currently deployed contract code
	pub link: Option<String>,
}

/// Contract source metadata trait
pub trait ContractSourceMetadataTrait {
  /// PUBLIC - View contract source metadata (Git references)
	fn contract_source_metadata(&self) -> ContractSourceMetadata;
  /// OWNER-ONLY - Patch/update contract source metadata
  fn patch_contract_source_metadata(&mut self, new_source_metadata: ContractSourceMetadata);
}

#[near_bindgen]
impl ContractSourceMetadataTrait for Contract {
    fn contract_source_metadata(&self) -> ContractSourceMetadata {
        self.contract_metadata.get().unwrap()
    }

    #[payable]
    fn patch_contract_source_metadata(&mut self, new_source_metadata: ContractSourceMetadata) {
      let initial_storage_usage = env::storage_usage();
			let owner_id = env::predecessor_account_id();
			assert_eq!(owner_id.clone(), self.tokens.owner_id, "Unauthorized");

      let source_metadata = self.contract_metadata.get();
      if let Some(mut source_metadata) = source_metadata {
        if new_source_metadata.link.is_some() {
          source_metadata.link = new_source_metadata.link;
        }
        if new_source_metadata.version.is_some() {
          source_metadata.version = new_source_metadata.version;
        }
        if new_source_metadata.commit_hash.is_some() {
          source_metadata.commit_hash = new_source_metadata.commit_hash;
        }
        self.contract_metadata.set(&source_metadata);
      }
      
      let amt_to_refund = if env::storage_usage() > initial_storage_usage { env::storage_usage() - initial_storage_usage } else { initial_storage_usage - env::storage_usage() };
			refund_deposit(amt_to_refund);
    }
}