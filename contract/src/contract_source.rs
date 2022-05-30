use crate::*;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{PanicOnDefault};

/// Contract source metadata structure
#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, PanicOnDefault)]
#[serde(crate = "near_sdk::serde")]
pub struct ContractSourceMetadata {
	pub version: Option<String>,
  pub commit_hash: Option<String>,
	pub link: Option<String>,
}

/// Contract source metadata trait
pub trait ContractSourceMetadataTrait {
  // view method
	fn contract_source_metadata(&self) -> ContractSourceMetadata;
  // patch/update method
  fn patch_contract_source_metadata(&mut self, new_source_metadata: ContractSourceMetadata);
}

/// Implementation of the view function
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