use std::collections::HashMap;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::{U64, U128};
use near_sdk::{
	env, near_bindgen, serde_json::json, AccountId, BorshStorageKey, PanicOnDefault, CryptoHash,
};
use near_sdk::serde::{Deserialize, Serialize};

pub use crate::metadata::*;
pub use crate::nft_core::*;
pub use crate::utils::*;
pub use crate::approval::*;
pub use crate::royalty::*;
pub use crate::enumeration::*;
pub use crate::nft_type::*;
pub use crate::contract_source::*;

mod metadata;
mod nft_core;
mod utils;
mod approval;
mod royalty;
mod enumeration;
mod nft_type;
mod contract_source;

/// CUSTOM TYPES

/// payout series for royalties to market
#[derive(BorshDeserialize, BorshSerialize, Serialize, PanicOnDefault)]
#[serde(crate = "near_sdk::serde")]
pub struct Payout {
	payout: HashMap<AccountId, U128>
}

/// log type const
pub const EVENT_JSON: &str = "EVENT_JSON:";
/// between token_type_id and edition number e.g. 42:2 where 42 is type and 2 is edition
pub const TOKEN_DELIMETER: char = ':';
/// TokenMetadata.title returned for individual token e.g. "Title — 2/10" where 10 is max copies
pub const TITLE_DELIMETER: &str = " — ";
/// e.g. "Title — 2/10" where 10 is max copies
pub const EDITION_DELIMETER: &str = "/";
/// between filename and extension e.g. "cat.jpg" where cat is filename and jpg is extension
pub const FILE_DELIMETER: char = '.';

// CONTRACT

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct ContractV1 { // OLD
	tokens: NonFungibleTokenV1,
	metadata: LazyOption<NFTContractMetadata>,
	token_type_by_title: LookupMap<TokenTypeTitle, TokenTypeId>,
	token_type_by_id: UnorderedMap<TokenTypeId, TokenTypeV1>,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract { // CURRENT
	tokens_v1: NonFungibleTokenV1,
	tokens: VersionedNonFungibleToken,
	metadata: LazyOption<NFTContractMetadata>,
	contract_source_metadata: LazyOption<VersionedContractSourceMetadata>, // CONTRACT SOURCE METADATA: https://github.com/near/NEPs/blob/master/neps/nep-0330.md
	token_type_by_title: LookupMap<TokenTypeTitle, TokenTypeId>,
	token_type_by_id_v1: UnorderedMap<TokenTypeId, TokenTypeV1>,
	token_type_by_id: UnorderedMap<TokenTypeId, VersionedTokenType>,
	token_type_assets_by_id: LookupMap<TokenTypeId, TokenTypeAssets>, // parallel with token_type_by_id - used by minting function to set up NFT
	// token_type_mint_args_by_id: LookupMap<TokenTypeId, VersionedTokenTypeMintArgs>, // parallel with token_type_by_id - used by minting function to set up NFT
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum VersionedContract { 
    Current(Contract),
}

const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/svg+xml,%3Csvg width='89' height='87' viewBox='0 0 89 87' fill='none' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath d='M17.5427 48.1358C16.0363 48.1994 14.5323 47.9631 13.1165 47.4402C11.7006 46.9174 10.4007 46.1182 9.29096 45.0884C8.18118 44.0586 7.2833 42.8184 6.64855 41.4384C6.0138 40.0585 5.65465 38.5659 5.59156 37.0459C5.52847 35.5259 5.76267 34.0083 6.28084 32.5796C6.79901 31.151 7.59098 29.8393 8.61153 28.7194C9.63208 27.5996 10.8612 26.6936 12.2288 26.0531C13.5963 25.4126 15.0755 25.0502 16.5819 24.9865C24.9751 24.6329 35.6235 28.7963 45.0454 33.5128H45.1584C45.3247 33.5017 45.4826 33.4353 45.6073 33.3239C45.732 33.2125 45.8166 33.0624 45.8476 32.8973C45.8787 32.7322 45.8544 32.5613 45.7788 32.4115C45.7032 32.2618 45.5804 32.1416 45.4298 32.0699C34.3631 26.937 21.7648 22.4372 12.0376 23.1957C10.3305 23.3283 8.66598 23.7988 7.13906 24.5805C5.61215 25.3622 4.25275 26.4397 3.13852 27.7515C2.02429 29.0633 1.17706 30.5837 0.645141 32.2259C0.113223 33.8681 -0.0929378 35.5999 0.0384375 37.3225C0.169813 39.0451 0.636138 40.7247 1.41081 42.2655C2.18547 43.8062 3.25329 45.1779 4.55332 46.3022C5.85334 47.4265 7.36013 48.2815 8.98759 48.8182C10.6151 49.3549 12.3313 49.563 14.0385 49.4304C15.6964 49.2805 17.3083 48.7998 18.7805 48.016C18.3708 48.0818 17.9574 48.1218 17.5427 48.1358Z' fill='%23D5D4D8'/%3E%3Cpath d='M70.6208 62.6276C69.1906 61.7674 67.6059 61.2014 65.9579 60.9622C66.2954 61.1347 66.6237 61.3251 66.9414 61.5326C69.4762 63.2327 71.2378 65.8793 71.8388 68.8901C72.4398 71.9009 71.8309 75.0293 70.146 77.587C68.4612 80.1448 65.8383 81.9225 62.8545 82.5289C59.8708 83.1353 56.7704 82.5209 54.2356 80.8208C47.2384 76.1328 41.0438 66.4373 36.1491 57.0271C36.0056 56.9422 35.8383 56.9077 35.6734 56.9291C35.5084 56.9504 35.3551 57.0264 35.2374 57.1451C35.1198 57.2637 35.0446 57.4184 35.0234 57.5849C35.0022 57.7514 35.0364 57.9202 35.1205 58.065C41.0947 68.7699 48.6853 79.8968 56.9655 85.0525C58.4248 85.9573 60.0463 86.5631 61.7376 86.8355C63.4289 87.1079 65.1567 87.0415 66.8226 86.6401C68.4884 86.2386 70.0596 85.51 71.4464 84.4959C72.8332 83.4818 74.0084 82.2019 74.905 80.7295C75.8016 79.2571 76.4021 77.6208 76.672 75.9143C76.9419 74.2077 76.8761 72.4641 76.4783 70.7832C76.0805 69.1023 75.3584 67.5169 74.3534 66.1175C73.3484 64.7182 72.08 63.5323 70.6208 62.6276Z' fill='%23D5D4D8'/%3E%3Cpath d='M85.8925 28.0491C83.6519 25.3945 80.4581 23.7464 77.0135 23.4673C73.5688 23.1881 70.1553 24.3008 67.5235 26.5606C66.3246 27.6147 65.3366 28.8904 64.6127 30.319C64.8388 30.1023 65.0705 29.8913 65.3192 29.6917C66.498 28.7232 67.8557 28.0006 69.3135 27.5659C70.7713 27.1312 72.3001 26.9929 73.8113 27.1592C75.3224 27.3255 76.7859 27.7929 78.1165 28.5345C79.4472 29.276 80.6187 30.2769 81.5629 31.4789C82.5072 32.681 83.2054 34.0603 83.6171 35.5369C84.0289 37.0134 84.1459 38.5578 83.9613 40.0803C83.7768 41.6028 83.2944 43.0732 82.5421 44.4061C81.7899 45.739 80.7828 46.9079 79.5792 47.8449C73.0173 53.0861 62.058 56.029 51.6922 57.8084L51.6074 57.8825C51.4778 57.9889 51.3873 58.136 51.3504 58.3005C51.3135 58.4649 51.3324 58.637 51.404 58.7893C51.4762 58.9429 51.5971 59.0678 51.7476 59.1442C51.8981 59.2207 52.0695 59.2443 52.2348 59.2114C64.1662 56.7875 76.9906 52.9664 84.4174 46.5845C87.0482 44.3235 88.6815 41.1008 88.9581 37.625C89.2348 34.1492 88.1321 30.7048 85.8925 28.0491Z' fill='%23D5D4D8'/%3E%3Cpath d='M56.649 8.35602C56.0177 6.7294 55.0717 5.24598 53.866 3.99237C52.6603 2.73876 51.2192 1.7401 49.6268 1.05467C48.0344 0.369244 46.3227 0.0107821 44.5915 0.000239517C42.8603 -0.010303 41.1443 0.327284 39.5439 0.99327C37.9434 1.65926 36.4905 2.6403 35.2699 3.87914C34.0493 5.11797 33.0856 6.58976 32.4349 8.20857C31.7842 9.82738 31.4596 11.5608 31.4802 13.3075C31.5007 15.0543 31.8659 16.7795 32.5544 18.3822C33.1795 19.8541 34.0751 21.1932 35.194 22.3288C35.047 22.0266 34.9114 21.7186 34.7927 21.3992C34.2388 19.9674 33.9729 18.4387 34.0104 16.9022C34.048 15.3657 34.3881 13.8521 35.0112 12.4496C35.6342 11.047 36.5277 9.78363 37.6394 8.73301C38.7512 7.68238 40.0591 6.86554 41.4868 6.33006C42.9146 5.79458 44.4337 5.55116 45.9556 5.61402C47.4776 5.67688 48.9719 6.04475 50.3515 6.69618C51.7311 7.34761 52.9684 8.26957 53.9914 9.40836C55.0144 10.5472 55.8025 11.88 56.3099 13.3292C59.2207 21.2395 58.599 32.6858 57.0842 43.1569C57.0842 43.2139 57.1351 43.271 57.1577 43.3337C57.2187 43.4914 57.3302 43.624 57.4746 43.7103C57.6189 43.7966 57.7876 43.8318 57.954 43.8101C58.1204 43.7885 58.2748 43.7113 58.3927 43.5909C58.5106 43.4704 58.5852 43.3136 58.6046 43.1455C60.0063 30.9406 60.368 17.4526 56.649 8.35602Z' fill='%23D5D4D8'/%3E%3Cpath d='M37.6695 71.65C37.6148 72.0889 37.5298 72.5234 37.4152 72.9503C36.5737 75.8831 34.6186 78.362 31.9753 79.8479C29.3319 81.3338 26.2141 81.7065 23.2999 80.8849C20.3856 80.0633 17.9108 78.1139 16.4135 75.4606C14.9162 72.8074 14.5177 69.6649 15.3045 66.7168C17.5653 58.5327 24.8168 49.573 32.1984 41.9706C32.2366 41.8076 32.2203 41.6364 32.1519 41.4837C32.0835 41.331 31.967 41.2054 31.8205 41.1266C31.6739 41.0478 31.5057 41.0202 31.342 41.048C31.1782 41.0759 31.0282 41.1576 30.9154 41.2805C22.6748 50.3258 14.5245 61.0193 12.2298 70.5892C11.8279 72.2676 11.7575 74.0095 12.0227 75.7153C12.288 77.4212 12.8835 79.0576 13.7755 80.5312C14.6675 82.0048 15.8383 83.2867 17.2213 84.3036C18.6042 85.3206 20.1721 86.0528 21.8354 86.4584C23.4988 86.8639 25.225 86.9349 26.9155 86.6673C28.6061 86.3997 30.2278 85.7987 31.6882 84.8987C33.1485 83.9986 34.4189 82.8172 35.4268 81.4217C36.4346 80.0263 37.1602 78.4442 37.5621 76.7658C37.9426 75.0857 37.9792 73.3449 37.6695 71.65Z' fill='%23D5D4D8'/%3E%3C/svg%3E%0A";

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
		// STANDARD
    NonFungibleToken, // ACTIVE - self.tokens.owner_by_id located here
		NonFungibleToken2, // INACTIVE - self.tokens_v1.owner_by_id (empty) located here
    Metadata, // ACTIVE - self.metadata located here
		SourceMetadata, // ACTIVE - self.contract_source_metadata located here
    TokenMetadata, // INACTIVE - self.tokens_v1.token_metadata_by_id (empty) located here
		TokenMetadata2, // ACTIVE - self.tokens.token_metadata_by_id located here
    Enumeration, // ACTIVE - self.tokens.tokens_per_owner located here
		Enumeration2, // INACTIVE - self.tokens_v1.tokens_per_owner (empty) located here
    Approval, // ACTIVE - self.tokens.approvals_by_id located here
		Approval2, // INACTIVE - self.tokens_v1.approvals_by_id (empty) located here
		TokensPerOwner { account_hash: Vec<u8> },
		TokenPerOwnerInner { account_id_hash: CryptoHash },
		// CUSTOM
    TokenTypeByTitle,
    TokenTypeById, // INACTIVE - self.token_type_by_id_v1 located here
		TokenTypeById2, // ACTIVE - self.token_type_by_id located here
    TokensByTypeInner { token_type_id: u64 },
		TokenTypeAssetsById,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new_default_meta(owner_id: AccountId) -> Self {
        Self::new(
            owner_id,
            NFTContractMetadata {
                spec: NFT_METADATA_SPEC.to_string(),
                name: "Sonar by Satori".to_string(),
                symbol: "SONAR".to_string(),
                icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
                base_uri: None,
                reference: None,
                reference_hash: None,
            },
						"b245d8f7fbe72c250cbabbd16544477f9958be2e".to_string(), // example commit sha
        )
    }

    #[init]
    pub fn new(owner_id: AccountId, metadata: NFTContractMetadata, commit_sha: String) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();
				let source_metadata = ContractSourceMetadata {
					version: Some("v2".to_string()), // THIS MUST BE MANUALLY UPDATED ON EACH VERSION CHANGE
					commit_sha: Some(commit_sha), // SHA OF HEAD COMMIT IS QUERIED BY CONTRACT CALLER (SPEARMINT API)
					link: Some("https://github.com/satori-hq/nft-series".to_string()),
				};
        Self {
						tokens_v1: NonFungibleTokenV1::new(
							StorageKey::NonFungibleToken2,
							owner_id.clone(),
							Some(StorageKey::TokenMetadata),
							Some(StorageKey::Enumeration2),
							Some(StorageKey::Approval2),
						),
            tokens: VersionedNonFungibleToken::from(VersionedNonFungibleToken::Current(NonFungibleToken::new(
							StorageKey::NonFungibleToken,
							owner_id.clone(),
							Some(StorageKey::TokenMetadata2),
							Some(StorageKey::Enumeration),
							Some(StorageKey::Approval),
						))),
						token_type_by_id_v1: UnorderedMap::new(StorageKey::TokenTypeById),
						token_type_by_id: UnorderedMap::new(StorageKey::TokenTypeById2),
						token_type_by_title: LookupMap::new(StorageKey::TokenTypeByTitle),
						token_type_assets_by_id: LookupMap::new(StorageKey::TokenTypeAssetsById),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
						contract_source_metadata: LazyOption::new(StorageKey::SourceMetadata, Some(&VersionedContractSourceMetadata::Current(source_metadata))),
        }
    }

		fn tokens(&self) -> &NonFungibleToken {
			match &self.tokens {
					VersionedNonFungibleToken::Current(data) => data,
			}
		}

		fn tokens_mut(&mut self) -> &mut NonFungibleToken {
			match &mut self.tokens {
				VersionedNonFungibleToken::Current(data) => data,
			}
		}

		/// Update `base_uri` for contract
		#[payable]
		pub fn patch_base_uri(
				&mut self,
				base_uri: Option<String>,
		) {
			let initial_storage_usage = env::storage_usage();
			let owner_id = env::predecessor_account_id();
			assert_eq!(owner_id.clone(), self.tokens().owner_id, "Unauthorized");

			if let Some(base_uri) = base_uri {
				let metadata = self.metadata.get();
				if let Some(mut metadata) = metadata {
					metadata.base_uri = Some(base_uri);
					self.metadata.set(&metadata);
				}
			}
			let amt_to_refund = if env::storage_usage() > initial_storage_usage { env::storage_usage() - initial_storage_usage } else { initial_storage_usage - env::storage_usage() };
			refund_deposit(amt_to_refund);
		}

}