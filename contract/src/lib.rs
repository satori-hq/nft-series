use std::collections::HashMap;
use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata, NFT_METADATA_SPEC,
};
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_contract_standards::non_fungible_token::core::{
	NonFungibleTokenCore, NonFungibleTokenResolver
};
use near_contract_standards::non_fungible_token::NonFungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::{U64, U128};
use near_sdk::{
	assert_one_yocto, log, require, env, near_bindgen, serde_json::json, Balance, AccountId, BorshStorageKey, PanicOnDefault, Promise, PromiseOrValue,
};
use near_sdk::serde::{Deserialize, Serialize};

/// CUSTOM TYPES

/// payout series for royalties to market
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
#[derive(Serialize)]
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
/// between asset_id, supply_remaining and file_type e.g. "1:10:jpg" where 1 is asset ID, 10 is supply remaining & jpg is file type
pub const ASSET_DETAIL_DELIMETER: char = ':';

pub type TokenTypeId = u64;
pub type TokenTypeTitle = String;

// #[derive(BorshDeserialize, BorshSerialize)]
// #[derive(Serialize, Deserialize)]
// #[serde(crate = "near_sdk::serde")]
// pub enum AssetDetail {
// 	AssetId(String),
// 	SupplyRemaining(i128),
// 	FileType(String),
// }

pub type AssetDetail = Vec<u128>; // E.g. [1, 10] where 1 is asset_id and 10 is supply_remaining

#[derive(BorshDeserialize, BorshSerialize)]
pub struct TokenTypeAssets {
	token_type_title: TokenTypeTitle,
	receiver_id: AccountId,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct TokenType {
	metadata: TokenMetadata,
	asset_filetypes: Vec<String>, // e.g. jpg, png, mp4
	asset_distribution: Vec<AssetDetail>, 
	owner_id: AccountId,
	royalty: HashMap<AccountId, u32>,
	tokens: UnorderedSet<TokenId>,
	approved_market_id: Option<AccountId>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenTypeJson {
	metadata: TokenMetadata,
	owner_id: AccountId,
	royalty: HashMap<AccountId, u32>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TypeMintArgs {
	token_type_title: TokenTypeTitle,
	receiver_id: AccountId,
}

/// STANDARD
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    tokens: NonFungibleToken,
    metadata: LazyOption<NFTContractMetadata>,
	// CUSTOM
	token_type_by_title: LookupMap<TokenTypeTitle, TokenTypeId>,
	token_type_by_id: UnorderedMap<TokenTypeId, TokenType>,
}
const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/svg+xml,%3Csvg width='89' height='87' viewBox='0 0 89 87' fill='none' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath d='M17.5427 48.1358C16.0363 48.1994 14.5323 47.9631 13.1165 47.4402C11.7006 46.9174 10.4007 46.1182 9.29096 45.0884C8.18118 44.0586 7.2833 42.8184 6.64855 41.4384C6.0138 40.0585 5.65465 38.5659 5.59156 37.0459C5.52847 35.5259 5.76267 34.0083 6.28084 32.5796C6.79901 31.151 7.59098 29.8393 8.61153 28.7194C9.63208 27.5996 10.8612 26.6936 12.2288 26.0531C13.5963 25.4126 15.0755 25.0502 16.5819 24.9865C24.9751 24.6329 35.6235 28.7963 45.0454 33.5128H45.1584C45.3247 33.5017 45.4826 33.4353 45.6073 33.3239C45.732 33.2125 45.8166 33.0624 45.8476 32.8973C45.8787 32.7322 45.8544 32.5613 45.7788 32.4115C45.7032 32.2618 45.5804 32.1416 45.4298 32.0699C34.3631 26.937 21.7648 22.4372 12.0376 23.1957C10.3305 23.3283 8.66598 23.7988 7.13906 24.5805C5.61215 25.3622 4.25275 26.4397 3.13852 27.7515C2.02429 29.0633 1.17706 30.5837 0.645141 32.2259C0.113223 33.8681 -0.0929378 35.5999 0.0384375 37.3225C0.169813 39.0451 0.636138 40.7247 1.41081 42.2655C2.18547 43.8062 3.25329 45.1779 4.55332 46.3022C5.85334 47.4265 7.36013 48.2815 8.98759 48.8182C10.6151 49.3549 12.3313 49.563 14.0385 49.4304C15.6964 49.2805 17.3083 48.7998 18.7805 48.016C18.3708 48.0818 17.9574 48.1218 17.5427 48.1358Z' fill='%23D5D4D8'/%3E%3Cpath d='M70.6208 62.6276C69.1906 61.7674 67.6059 61.2014 65.9579 60.9622C66.2954 61.1347 66.6237 61.3251 66.9414 61.5326C69.4762 63.2327 71.2378 65.8793 71.8388 68.8901C72.4398 71.9009 71.8309 75.0293 70.146 77.587C68.4612 80.1448 65.8383 81.9225 62.8545 82.5289C59.8708 83.1353 56.7704 82.5209 54.2356 80.8208C47.2384 76.1328 41.0438 66.4373 36.1491 57.0271C36.0056 56.9422 35.8383 56.9077 35.6734 56.9291C35.5084 56.9504 35.3551 57.0264 35.2374 57.1451C35.1198 57.2637 35.0446 57.4184 35.0234 57.5849C35.0022 57.7514 35.0364 57.9202 35.1205 58.065C41.0947 68.7699 48.6853 79.8968 56.9655 85.0525C58.4248 85.9573 60.0463 86.5631 61.7376 86.8355C63.4289 87.1079 65.1567 87.0415 66.8226 86.6401C68.4884 86.2386 70.0596 85.51 71.4464 84.4959C72.8332 83.4818 74.0084 82.2019 74.905 80.7295C75.8016 79.2571 76.4021 77.6208 76.672 75.9143C76.9419 74.2077 76.8761 72.4641 76.4783 70.7832C76.0805 69.1023 75.3584 67.5169 74.3534 66.1175C73.3484 64.7182 72.08 63.5323 70.6208 62.6276Z' fill='%23D5D4D8'/%3E%3Cpath d='M85.8925 28.0491C83.6519 25.3945 80.4581 23.7464 77.0135 23.4673C73.5688 23.1881 70.1553 24.3008 67.5235 26.5606C66.3246 27.6147 65.3366 28.8904 64.6127 30.319C64.8388 30.1023 65.0705 29.8913 65.3192 29.6917C66.498 28.7232 67.8557 28.0006 69.3135 27.5659C70.7713 27.1312 72.3001 26.9929 73.8113 27.1592C75.3224 27.3255 76.7859 27.7929 78.1165 28.5345C79.4472 29.276 80.6187 30.2769 81.5629 31.4789C82.5072 32.681 83.2054 34.0603 83.6171 35.5369C84.0289 37.0134 84.1459 38.5578 83.9613 40.0803C83.7768 41.6028 83.2944 43.0732 82.5421 44.4061C81.7899 45.739 80.7828 46.9079 79.5792 47.8449C73.0173 53.0861 62.058 56.029 51.6922 57.8084L51.6074 57.8825C51.4778 57.9889 51.3873 58.136 51.3504 58.3005C51.3135 58.4649 51.3324 58.637 51.404 58.7893C51.4762 58.9429 51.5971 59.0678 51.7476 59.1442C51.8981 59.2207 52.0695 59.2443 52.2348 59.2114C64.1662 56.7875 76.9906 52.9664 84.4174 46.5845C87.0482 44.3235 88.6815 41.1008 88.9581 37.625C89.2348 34.1492 88.1321 30.7048 85.8925 28.0491Z' fill='%23D5D4D8'/%3E%3Cpath d='M56.649 8.35602C56.0177 6.7294 55.0717 5.24598 53.866 3.99237C52.6603 2.73876 51.2192 1.7401 49.6268 1.05467C48.0344 0.369244 46.3227 0.0107821 44.5915 0.000239517C42.8603 -0.010303 41.1443 0.327284 39.5439 0.99327C37.9434 1.65926 36.4905 2.6403 35.2699 3.87914C34.0493 5.11797 33.0856 6.58976 32.4349 8.20857C31.7842 9.82738 31.4596 11.5608 31.4802 13.3075C31.5007 15.0543 31.8659 16.7795 32.5544 18.3822C33.1795 19.8541 34.0751 21.1932 35.194 22.3288C35.047 22.0266 34.9114 21.7186 34.7927 21.3992C34.2388 19.9674 33.9729 18.4387 34.0104 16.9022C34.048 15.3657 34.3881 13.8521 35.0112 12.4496C35.6342 11.047 36.5277 9.78363 37.6394 8.73301C38.7512 7.68238 40.0591 6.86554 41.4868 6.33006C42.9146 5.79458 44.4337 5.55116 45.9556 5.61402C47.4776 5.67688 48.9719 6.04475 50.3515 6.69618C51.7311 7.34761 52.9684 8.26957 53.9914 9.40836C55.0144 10.5472 55.8025 11.88 56.3099 13.3292C59.2207 21.2395 58.599 32.6858 57.0842 43.1569C57.0842 43.2139 57.1351 43.271 57.1577 43.3337C57.2187 43.4914 57.3302 43.624 57.4746 43.7103C57.6189 43.7966 57.7876 43.8318 57.954 43.8101C58.1204 43.7885 58.2748 43.7113 58.3927 43.5909C58.5106 43.4704 58.5852 43.3136 58.6046 43.1455C60.0063 30.9406 60.368 17.4526 56.649 8.35602Z' fill='%23D5D4D8'/%3E%3Cpath d='M37.6695 71.65C37.6148 72.0889 37.5298 72.5234 37.4152 72.9503C36.5737 75.8831 34.6186 78.362 31.9753 79.8479C29.3319 81.3338 26.2141 81.7065 23.2999 80.8849C20.3856 80.0633 17.9108 78.1139 16.4135 75.4606C14.9162 72.8074 14.5177 69.6649 15.3045 66.7168C17.5653 58.5327 24.8168 49.573 32.1984 41.9706C32.2366 41.8076 32.2203 41.6364 32.1519 41.4837C32.0835 41.331 31.967 41.2054 31.8205 41.1266C31.6739 41.0478 31.5057 41.0202 31.342 41.048C31.1782 41.0759 31.0282 41.1576 30.9154 41.2805C22.6748 50.3258 14.5245 61.0193 12.2298 70.5892C11.8279 72.2676 11.7575 74.0095 12.0227 75.7153C12.288 77.4212 12.8835 79.0576 13.7755 80.5312C14.6675 82.0048 15.8383 83.2867 17.2213 84.3036C18.6042 85.3206 20.1721 86.0528 21.8354 86.4584C23.4988 86.8639 25.225 86.9349 26.9155 86.6673C28.6061 86.3997 30.2278 85.7987 31.6882 84.8987C33.1485 83.9986 34.4189 82.8172 35.4268 81.4217C36.4346 80.0263 37.1602 78.4442 37.5621 76.7658C37.9426 75.0857 37.9792 73.3449 37.6695 71.65Z' fill='%23D5D4D8'/%3E%3C/svg%3E%0A";
// const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 288 288'%3E%3Cg id='l' data-name='l'%3E%3Cpath d='M187.58,79.81l-30.1,44.69a3.2,3.2,0,0,0,4.75,4.2L191.86,103a1.2,1.2,0,0,1,2,.91v80.46a1.2,1.2,0,0,1-2.12.77L102.18,77.93A15.35,15.35,0,0,0,90.47,72.5H87.34A15.34,15.34,0,0,0,72,87.84V201.16A15.34,15.34,0,0,0,87.34,216.5h0a15.35,15.35,0,0,0,13.08-7.31l30.1-44.69a3.2,3.2,0,0,0-4.75-4.2L96.14,186a1.2,1.2,0,0,1-2-.91V104.61a1.2,1.2,0,0,1,2.12-.77l89.55,107.23a15.35,15.35,0,0,0,11.71,5.43h3.13A15.34,15.34,0,0,0,216,201.16V87.84A15.34,15.34,0,0,0,200.66,72.5h0A15.35,15.35,0,0,0,187.58,79.81Z'/%3E%3C/g%3E%3C/svg%3E";
#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
	// STANDARD
    NonFungibleToken,
    Metadata,
    TokenMetadata,
    Enumeration,
    Approval,
		TokensPerOwner { account_hash: Vec<u8> },
		// TokenPerOwnerInner { account_id_hash: CryptoHash },
	// CUSTOM
    TokenTypeByTitle,
    TokenTypeById,
    TokensByTypeInner { token_type_id: u64 },
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
        )
    }

    #[init]
    pub fn new(owner_id: AccountId, metadata: NFTContractMetadata) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();
        Self {
            tokens: NonFungibleToken::new(
                StorageKey::NonFungibleToken,
                owner_id,
                Some(StorageKey::TokenMetadata),
                Some(StorageKey::Enumeration),
                Some(StorageKey::Approval),
            ),
						token_type_by_id: UnorderedMap::new(StorageKey::TokenTypeById),
						token_type_by_title: LookupMap::new(StorageKey::TokenTypeByTitle),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
        }
    }

	// CUSTOM

		#[payable]
		pub fn patch_base_uri(
				&mut self,
				base_uri: Option<String>,
		) {
			let initial_storage_usage = env::storage_usage();
			let owner_id = env::predecessor_account_id();
			assert_eq!(owner_id.clone(), self.tokens.owner_id, "Unauthorized");

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
    
    #[payable]
    pub fn nft_create_type(
        &mut self,
        metadata: TokenMetadata,
        royalty: HashMap<AccountId, u32>,
				asset_filetypes: Vec<String>,
				asset_distribution: Vec<AssetDetail>,
    ) {
		let initial_storage_usage = env::storage_usage();
        let owner_id = env::predecessor_account_id();
		assert_eq!(owner_id.clone(), self.tokens.owner_id, "Unauthorized");
		let title = metadata.title.clone();
		assert!(title.is_some(), "token_metadata.title is required");
		assert!(!asset_distribution.is_empty(), "asset_distribution must not be empty");
		assert_eq!(asset_filetypes.len(), asset_distribution.len(), "asset_filetypes and asset_distribution must be same length");

		// validate asset_distribution elements (must contain two integers: asset_id and supply_remaining)
		for distr_detail in &asset_distribution {
			let asset_id = distr_detail.get(0);
			assert!(asset_id.is_some(), "Asset ID must be provided");
			let supply_remaining = distr_detail.get(1);
			assert!(supply_remaining.is_some(), "Supply remaining must be provided");
		}

		let token_type_id = self.token_type_by_id.len() + 1;
        assert!(self.token_type_by_title.insert(&title.unwrap(), &token_type_id).is_none(), "token_metadata.title exists");
        self.token_type_by_id.insert(&token_type_id, &TokenType{
			metadata,
			owner_id,
			royalty,
			asset_filetypes,
			asset_distribution,
			tokens: UnorderedSet::new(
				StorageKey::TokensByTypeInner {
					token_type_id
				}
				.try_to_vec()
				.unwrap(),
			),
			approved_market_id: None,
		});

        refund_deposit(env::storage_usage() - initial_storage_usage);
    }

	pub fn cap_copies(
		&mut self,
		token_type_title: TokenTypeTitle,
	) {
		assert_eq!(env::predecessor_account_id(), self.tokens.owner_id, "Unauthorized");
		let token_type_id = self.token_type_by_title.get(&token_type_title).expect("no type");
		let mut token_type = self.token_type_by_id.get(&token_type_id).expect("no token");
		token_type.metadata.copies = Some(token_type.tokens.len());
		self.token_type_by_id.insert(&token_type_id, &token_type);
	}

	#[payable]
    pub fn nft_patch_type(
        &mut self,
				token_type_title: TokenTypeTitle,
				metadata: Option<TokenMetadata>,
        royalty: Option<HashMap<AccountId, u32>>,
    ) {
		let initial_storage_usage = env::storage_usage();
    let owner_id = env::predecessor_account_id();
		assert_eq!(owner_id.clone(), self.tokens.owner_id, "Unauthorized");

		let token_type_id = self.token_type_by_title.get(&token_type_title).expect("no type");
		let mut token_type = self.token_type_by_id.get(&token_type_id).expect("no token");

		if let Some(metadata) = metadata {
			if metadata.title.is_some() {
				token_type.metadata.title = metadata.title
			}
			// don't validate that description is_some, as description can be none
			token_type.metadata.description = metadata.description;
			if metadata.media.is_some() {
				token_type.metadata.media = metadata.media
			}
			// don't allow to patch copies (this must go through `cap_copies`)
		}
		if let Some(royalty) = royalty {
			token_type.royalty = royalty
		}
		self.token_type_by_id.insert(&token_type_id, &token_type);

		let amt_to_refund = if env::storage_usage() > initial_storage_usage { env::storage_usage() - initial_storage_usage } else { initial_storage_usage - env::storage_usage() };
    refund_deposit(amt_to_refund);
  }

	#[payable]
	pub fn nft_mint_type(
		&mut self,
		token_type_title: TokenTypeTitle,
		receiver_id: AccountId,
    _metadata: Option<TokenMetadata>,
	) -> Token {

		assert_eq!(env::predecessor_account_id(), self.tokens.owner_id, "Unauthorized");

		let initial_storage_usage = env::storage_usage();

		let token_type_id = self.token_type_by_title.get(&token_type_title).expect("no type");
		let mut token_type = self.token_type_by_id.get(&token_type_id).expect("no token");
		assert_eq!(&env::predecessor_account_id(), &token_type.owner_id, "not type owner");

		let num_tokens = token_type.tokens.len();
		let max_copies = token_type.metadata.copies.unwrap_or(u64::MAX);
		assert_ne!(num_tokens, max_copies, "type supply maxed");

		let token_id = format!("{}{}{}", &token_type_id, TOKEN_DELIMETER, num_tokens + 1);
		token_type.tokens.insert(&token_id);
		self.token_type_by_id.insert(&token_type_id, &token_type);

		// TODO finish adding custom metadata (if provided) to final_metadata
		// you can add custom metadata to each token here
		// make sure you update self.nft_token to "patch" over the type metadata
		let final_metadata = Some(TokenMetadata {
			title: None, // ex. "Arch Nemesis: Mail Carrier" or "Parcel #5055"
			description: None, // free-form description
			media: None, // URL to associated media, preferably to decentralized, content-addressed storage
			copies: None, // number of copies of this set of metadata in existence when token was minted.
		});
		// if let Some(metadata) = metadata {
			
		// }
		let token = self.tokens.internal_mint(token_id.clone(), receiver_id.clone(), final_metadata);

        refund_deposit(env::storage_usage() - initial_storage_usage);

		env::log_str(format!("{}{}", EVENT_JSON, json!({
			"standard": "nep171",
			"version": "1.0.0",
			"event": "nft_mint",
			"data": [
			  	{
					  "owner_id": receiver_id,
					  "token_ids": [token_id]
				}
			]
		})).as_ref());
			
		token
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
			self.tokens.owner_by_id.insert(token_id, to);

			// if using Enumeration standard, update old & new owner's token lists
			if let Some(tokens_per_owner) = &mut self.tokens.tokens_per_owner {
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
			// let receiver_token_set = if let Some(receiver_token_set) = tokens_per_owner.get(&to) {
			// 	receiver_token_set
			// } else {
			// 	UnorderedSet::new()
			// };
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
			let owner_id = self.tokens.owner_by_id.get(token_id).unwrap_or_else(|| env::panic_str("Token not found"));

			// clear approvals, if using Approval Management extension
			// this will be rolled back by a panic if sending fails
			let approved_account_ids = self.tokens.approvals_by_id.as_mut().and_then(|by_id| by_id.remove(token_id));

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

	#[payable]
	pub fn nft_transfer(
		&mut self,
		receiver_id: AccountId,
		token_id: TokenId,
		approval_id: Option<u64>,
		memo: Option<String>,
	) {
		assert_one_yocto();
		let sender_id = env::predecessor_account_id();
		self.internal_transfer(&sender_id, &receiver_id, &token_id, approval_id, memo);
	}
	
	/// convert the royalty percentage and amount to pay into a payout (U128)
	fn royalty_to_payout(&self, royalty_percentage: u32, amount_to_pay: Balance) -> U128 {
	    U128(royalty_percentage as u128 * amount_to_pay / 10_000u128)
	}

	/// CUSTOM re-implement core standard here, not using macros from near-contract-standards


	/// pass through
	#[payable]
	pub fn nft_transfer_call(
		&mut self,
		receiver_id: AccountId,
		token_id: TokenId,
		approval_id: Option<u64>,
		memo: Option<String>,
		msg: String,
	) -> PromiseOrValue<bool> {
		self.tokens.nft_transfer_call(receiver_id, token_id, approval_id, memo, msg)
	}

	//calculates the payout for a token given the passed in balance. This is a view method
	pub fn nft_payout(&self, token_id: TokenId, balance: U128, max_len_payout: u32) -> Payout {
		//get the token object
		let token = self.nft_token(token_id.clone()).expect("no token");

		//get the owner of the token
		let owner_id = token.owner_id;
		//keep track of the total perpetual royalties
		let mut total_perpetual = 0;
		//get the u128 version of the passed in balance (which was U128 before)
		let balance_u128 = u128::from(balance);
		//keep track of the payout object to send back
		let mut payout_object = Payout {
				payout: HashMap::new()
		};
		//get the royalty object from token
		let mut token_id_iter = token_id.split(TOKEN_DELIMETER);
		let token_type_id = token_id_iter.next().unwrap().parse().unwrap();
		let royalty = self.token_type_by_id.get(&token_type_id).expect("no type").royalty;
		// let royalty = token.royalty;

		//make sure we're not paying out to too many people (GAS limits this)
		assert!(royalty.len() as u32 <= max_len_payout, "Market cannot payout to that many receivers");

		//go through each key and value in the royalty object
		for (k, v) in royalty.iter() {
			//get the key
			let key = k.clone();
			//only insert into the payout if the key isn't the token owner (we add their payout at the end)
			if key != owner_id {
				payout_object.payout.insert(key, self.royalty_to_payout(*v, balance_u128));
				total_perpetual += *v;
			}
		}

		// payout to previous owner who gets 100% - total perpetual royalties
		let owner_payout = self.royalty_to_payout(10000 - total_perpetual, balance_u128);
		if u128::from(owner_payout) > 0 {
			payout_object.payout.insert(owner_id, owner_payout);
		}

		//return the payout object
		payout_object
	}

	/// CUSTOM royalties payout
	#[payable]
	pub fn nft_transfer_payout(
		&mut self,
		receiver_id: AccountId,
		token_id: TokenId,
		approval_id: u64,
		memo: Option<String>,
		balance: Option<U128>,
		max_len_payout: Option<u32>,
	) -> Option<Payout> {

		// lazy minting?
		let type_mint_args = memo.clone();
		let previous_token = if let Some(type_mint_args) = type_mint_args {
			log!(format!("type_mint_args: {}", type_mint_args));
			let TypeMintArgs{token_type_title, receiver_id} = near_sdk::serde_json::from_str(&type_mint_args).expect("invalid TypeMintArgs");
			self.nft_mint_type(token_type_title, receiver_id.clone(), None)
		} else {
			let prev_token = self.nft_token(token_id.clone()).expect("no token");
			self.tokens.nft_transfer(receiver_id.clone(), token_id.clone(), Some(approval_id), memo);
			prev_token
		};

        // compute payouts based on balance option
        let owner_id = previous_token.owner_id;
        let payout_struct = if let Some(balance) = balance {
			let complete_royalty = 10_000u128;
            let balance_piece = u128::from(balance) / complete_royalty;
			let mut total_royalty_percentage = 0;
            // let mut payout: Payout = HashMap::new();
			let mut payout_struct: Payout = Payout{
				payout: HashMap::new()
			};
			let mut token_id_iter = token_id.split(TOKEN_DELIMETER);
			let token_type_id = token_id_iter.next().unwrap().parse().unwrap();
            let royalty = self.token_type_by_id.get(&token_type_id).expect("no type").royalty;

            if let Some(max_len_payout) = max_len_payout {
                assert!(royalty.len() as u32 <= max_len_payout, "exceeds max_len_payout");
            }
            for (k, v) in royalty.iter() {
                let key = k.clone();
				// skip seller and payout once at end
                if key != owner_id {
                    payout_struct.payout.insert(key, U128(*v as u128 * balance_piece));
                    total_royalty_percentage += *v;
                }
            }
            // payout to seller
						let seller_payout = (complete_royalty - total_royalty_percentage as u128) * balance_piece;
						if seller_payout > 0 {
							payout_struct.payout.insert(owner_id.clone(), U128(seller_payout));
						}
            // payout_struct.payout.insert(owner_id.clone(), U128((complete_royalty - total_royalty_percentage as u128) * balance_piece));
            Some(payout_struct)
        } else {
            None
        };

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

        payout_struct
	}

	/// CUSTOM re-implementation of near-contract-standards (not using macros)
	
	/// CUSTOM every enumeration method goes through here (watch the gas on views...)
	
	pub fn nft_token(&self, token_id: TokenId) -> Option<Token> {
		let owner_id = self.tokens.owner_by_id.get(&token_id)?;
        let approved_account_ids = self.tokens
            .approvals_by_id
			.as_ref()
            .and_then(|by_id| by_id.get(&token_id).or_else(|| Some(HashMap::new())));

		// CUSTOM (switch metadata for the token_type metadata)
		let mut token_id_iter = token_id.split(TOKEN_DELIMETER);
		let token_type_id = token_id_iter.next().unwrap().parse().unwrap();
		// make edition titles nice for showing in wallet
		let mut metadata = self.token_type_by_id.get(&token_type_id).unwrap().metadata;
		let copies = metadata.copies;
		if let Some(copies) = copies {
			metadata.title = Some(
				format!(
					"{}{}{}{}{}",
					metadata.title.unwrap(),
					TITLE_DELIMETER,
					token_id_iter.next().unwrap(),
					EDITION_DELIMETER,
					copies
				)
			);
		}
		
		// CUSTOM
		// implement this if you need to combine individual token metadata
		// e.g. metadata.extra with TokenType.metadata.extra and return something unique
		// let token_metadata = self.tokens.token_metadata_by_id.get(&token_id)?;
		// metadata.extra = token_metadata.extra;

        Some(Token { token_id, owner_id, metadata: Some(metadata), approved_account_ids })
	}

	pub fn nft_total_supply(&self) -> U128 {
		(self.tokens.owner_by_id.len() as u128).into()
	}

    pub fn nft_tokens(&self, from_index: Option<U128>, limit: Option<u64>) -> Vec<Token> {
        // Get starting index, whether or not it was explicitly given.
        // Defaults to 0 based on the spec:
        // https://nomicon.io/Standards/NonFungibleToken/Enumeration.html#interface
        let start_index: u128 = from_index.map(From::from).unwrap_or_default();
        assert!(
            (self.tokens.owner_by_id.len() as u128) > start_index,
            "Out of bounds, please use a smaller from_index."
        );
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        self.tokens.owner_by_id
            .iter()
            .skip(start_index as usize)
            .take(limit)
            .map(|(token_id, _)| self.nft_token(token_id).unwrap())
            .collect()
    }

    pub fn nft_supply_for_owner(self, account_id: AccountId) -> U128 {
        let tokens_per_owner = self.tokens.tokens_per_owner.expect(
            "Could not find tokens_per_owner when calling a method on the enumeration standard.",
        );
        tokens_per_owner
            .get(&account_id)
            .map(|account_tokens| U128::from(account_tokens.len() as u128))
            .unwrap_or(U128(0))
    }

	pub fn nft_tokens_for_owner(
        &self,
        account_id: AccountId,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<Token> {
        let tokens_per_owner = self.tokens.tokens_per_owner.as_ref().expect(
            "Could not find tokens_per_owner when calling a method on the enumeration standard.",
        );
        let token_set = if let Some(token_set) = tokens_per_owner.get(&account_id) {
            token_set
        } else {
            return vec![];
        };
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        let start_index: u128 = from_index.map(From::from).unwrap_or_default();
        assert!(
            token_set.len() as u128 > start_index,
            "Out of bounds, please use a smaller from_index."
        );
        token_set
            .iter()
            .skip(start_index as usize)
            .take(limit)
            .map(|token_id| self.nft_token(token_id).unwrap())
            .collect()
    }

	/// CUSTOM VIEWS for typed tokens

	pub fn nft_get_type(&self, token_type_title: TokenTypeTitle) -> TokenTypeJson {
		let token_type = self.token_type_by_id.get(&self.token_type_by_title.get(&token_type_title).expect("no type")).expect("no type");
		TokenTypeJson{
			metadata: token_type.metadata,
			owner_id: token_type.owner_id,
			royalty: token_type.royalty,
		}
	}

	pub fn nft_get_type_format(&self) -> (char, &'static str, &'static str) {
		(TOKEN_DELIMETER, TITLE_DELIMETER, EDITION_DELIMETER)
	}

	pub fn nft_get_types(
		&self,
		from_index: Option<U128>,
		limit: Option<u64>
	) -> Vec<TokenTypeJson> {
        let start_index: u128 = from_index.map(From::from).unwrap_or_default();
        assert!(
            (self.token_type_by_id.len() as u128) >= start_index,
            "Out of bounds, please use a smaller from_index."
        );
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        
		self.token_type_by_id.iter()
            .skip(start_index as usize)
            .take(limit)
            .map(|(_, token_type)| TokenTypeJson{
				metadata: token_type.metadata,
				owner_id: token_type.owner_id,
				royalty: token_type.royalty,
			})
            .collect()
    }

	pub fn nft_supply_for_type(
        &self,
        token_type_title: TokenTypeTitle,
    ) -> U64 {
        self.token_type_by_id.get(&self.token_type_by_title.get(&token_type_title).expect("no type")).expect("no type").tokens.len().into()
    }

	pub fn nft_tokens_by_type(
		&self,
        token_type_title: TokenTypeTitle,
		from_index: Option<U128>,
		limit: Option<u64>
	) -> Vec<Token> {

        let start_index: u128 = from_index.map(From::from).unwrap_or_default();
		let tokens = self.token_type_by_id.get(&self.token_type_by_title.get(&token_type_title).expect("no type")).expect("no type").tokens;
        assert!(
            (tokens.len() as u128) >= start_index,
            "Out of bounds, please use a smaller from_index."
        );
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        
		tokens.iter()
            .skip(start_index as usize)
            .take(limit)
            .map(|token_id| self.nft_token(token_id).unwrap())
            .collect()
    }
}

// near-contract-standards macros
// near_contract_standards::impl_non_fungible_token_core!(Contract, tokens);
// near_contract_standards::impl_non_fungible_token_enumeration!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_approval!(Contract, tokens);

#[near_bindgen]
impl NonFungibleTokenMetadataProvider for Contract {
    fn nft_metadata(&self) -> NFTContractMetadata {
        self.metadata.get().unwrap()
    }
}

#[near_bindgen]
impl NonFungibleTokenResolver for Contract {
	#[private]
	fn nft_resolve_transfer(
		&mut self,
		previous_owner_id: AccountId,
		receiver_id: AccountId,
		token_id: TokenId,
		approved_account_ids: Option<HashMap<AccountId, u64>>,
	) -> bool {
		self.tokens.nft_resolve_transfer(
			previous_owner_id,
			receiver_id,
			token_id,
			approved_account_ids,
		)
	}
}

/// from https://github.com/near/near-sdk-rs/blob/e4abb739ff953b06d718037aa1b8ab768db17348/near-contract-standards/src/non_fungible_token/utils.rs#L29

pub fn refund_deposit(storage_used: u64) {
    let required_cost = env::storage_byte_cost() * Balance::from(storage_used);
    let attached_deposit = env::attached_deposit();

    assert!(
        required_cost <= attached_deposit,
        "Must attach {} yoctoNEAR to cover storage",
        required_cost,
    );

    let refund = attached_deposit - required_cost;
	// log!("refund_deposit amount {}", refund);
    if refund > 1 {
        Promise::new(env::predecessor_account_id()).transfer(refund);
    }
}