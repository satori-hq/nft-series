use crate::*;
use near_sdk::{log};

pub type TokenTypeId = u64;
pub type TokenTypeTitle = String;

pub type AssetDetail = Vec<String>; // Vec with 3 x string elements. E.g. ["1.jpg", "10", "1.json"] where 1.jpg is asset filename 10 is supply_remaining, and "1.json" is json filename. (final element should be empty string if no json is available)
pub type TokenTypeAssets = Vec<AssetDetail>;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct TokenTypeV1 {
	pub metadata: TokenTypeMetadata,
	pub owner_id: AccountId,
	pub royalty: HashMap<AccountId, u32>,
	pub tokens: UnorderedSet<TokenId>,
	pub approved_market_id: Option<AccountId>,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct TokenType {
	pub metadata: TokenTypeMetadata,
	pub owner_id: AccountId,
	pub royalty: HashMap<AccountId, u32>,
	pub tokens: UnorderedSet<TokenId>,
	pub approved_market_id: Option<AccountId>,
	// NEW
	pub asset_count: u64,
	/// filename for cover asset located within IPFS directory (metadata.media)
	pub cover_asset: Option<String>,
}

impl From<TokenTypeV1> for TokenType {
	fn from(v1: TokenTypeV1) -> Self {
		TokenType {
			metadata: v1.metadata,
			owner_id: v1.owner_id,
			royalty: v1.royalty,
			tokens: v1.tokens,
			approved_market_id: v1.approved_market_id,
			asset_count: 1, // all existing token types have 1 asset
			cover_asset: None,
		}
	}
}

pub fn versioned_token_type_to_token_type(versioned_token_type: VersionedTokenType) -> TokenType {
	match versioned_token_type {
			VersionedTokenType::Current(current) => current,
			VersionedTokenType::V1(v1) => {
				TokenType {
					metadata: v1.metadata,
					owner_id: v1.owner_id,
					royalty: v1.royalty,
					tokens: v1.tokens,
					approved_market_id: v1.approved_market_id,
					asset_count: 1, // all existing token types have 1 asset
					cover_asset: None,
				}
			}
			// FINISH
	}
}

// #[derive(Serialize, Deserialize)]
// #[serde(crate = "near_sdk::serde")]
#[derive(BorshDeserialize, BorshSerialize)]
pub enum VersionedTokenType {
		V1(TokenTypeV1),
    Current(TokenType),
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenTypeJson {
	pub metadata: TokenTypeMetadata,
	pub owner_id: AccountId,
	pub royalty: HashMap<AccountId, u32>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TypeMintArgs {
	pub token_type_title: TokenTypeTitle,
	pub receiver_id: AccountId,
}

/// methods for NFT type (otherwise known as "series")
pub trait NonFungibleTokenType {

  /// Create a new NFT type (aka series)
  fn nft_create_type(
      &mut self,
      metadata: TokenTypeMetadata,
      royalty: HashMap<AccountId, u32>,
			assets: TokenTypeAssets,
			cover_asset: String,
  );

  /// Cap copies of an existing NFT type/series to currently minted supply
	fn nft_cap_copies(
		&mut self,
		token_type_title: TokenTypeTitle,
	);

  /// Update any metadata or royalty fields of an existing NFT type/series EXCEPT `copies`
  fn nft_update_type(
      &mut self,
      token_type_title: TokenTypeTitle,
      metadata: Option<TokenMetadata>,
      royalty: Option<HashMap<AccountId, u32>>,
  );

  /// Mint an NFT for specified type/series
	fn nft_mint_type(
		&mut self,
		token_type_title: TokenTypeTitle,
		receiver_id: AccountId,
    _metadata: Option<TokenMetadata>,
) -> Token;

	/// Mint a batch of NFTs for specified type/series
	fn nft_batch_mint_type(
		&mut self,
		token_type_title: TokenTypeTitle,
		receiver_ids: Vec<AccountId>
	) -> Vec<Token>;

	/// Delete an NFT type/series that is empty (no NFTs minted yet)
	fn nft_delete_type(
		&mut self,
		token_type_title: TokenTypeTitle,
	);
}

#[near_bindgen]
impl NonFungibleTokenType for Contract {
  #[payable]
  fn nft_create_type(
        &mut self,
        metadata: TokenTypeMetadata,
        royalty: HashMap<AccountId, u32>,
				assets: TokenTypeAssets,
				cover_asset: String, // filename for cover asset located within IPFS directory (metadata.media)
    ) {

		let initial_storage_usage = env::storage_usage();

		// VALIDATION
    let owner_id = env::predecessor_account_id();
		assert_eq!(owner_id.clone(), self.tokens().owner_id, "Unauthorized");
		// `title` required
		let title = metadata.title.clone();
		assert!(title.is_some(), "token_metadata.title is required");
		// `copies` required
		let copies = metadata.copies.clone();
		assert!(copies.is_some(), "token_metadata.copies is required");
		// `media` required
		let media = metadata.media.clone();
		assert!(media.is_some(), "token_metadata.media is required");

		let token_type_id = self.token_type_by_id.len() + 1;

		assert!(self.token_type_by_title.insert(&metadata.title.clone().unwrap(), &token_type_id).is_none(), "token_metadata.title exists");

		assert!(!assets.is_empty(), "assets vector must not be empty");

		assert!(assets.len() <= metadata.copies.unwrap() as usize, "length of assets vector must not exceed copies");

		// sum of total_supply must be equal to `metadata.copies`
		let mut total_supply = 0 as u64;
		for asset_detail in assets.clone() { // TODO: may want to ultimately verify each of these elements on Spearmint instead to save on gas costs (also so we don't hit gas limit... FYI 10,000 is fine, but 50,000+ becomes problematic)
			// verify asset filename exists
			let asset_filename = asset_detail.get(0);
			assert!(asset_filename.is_some(), "Asset filename must be provided");
			// verify 3rd element ("extra") exists (should be empty string if no "extra" file is available for this asset)
			let asset_extra = asset_detail.get(2);
			assert!(asset_extra.is_some(), "3 elements must be provided in each sub-array of assets (if there is no 'extra'/json file available for this asset, 3rd element should be empty string.)");
			let supply_remaining: u64 = asset_detail.get(1).unwrap().clone().parse().unwrap();
			// tally total_supply to verify against metadata.copies
			total_supply = total_supply + supply_remaining;
		}
		assert!(total_supply == metadata.copies.unwrap(), "Total supply must equal copies. Received {} total supply & {} copies", total_supply, metadata.copies.unwrap());

		let token_type = TokenType {
			metadata,
			owner_id,
			royalty,
			tokens: UnorderedSet::new(
				StorageKey::TokensByTypeInner {
					token_type_id
				}
				.try_to_vec()
				.unwrap(),
			),
			approved_market_id: None,
			asset_count: assets.len() as u64,
			cover_asset: Some(cover_asset),
		};
		let versioned_token_type = VersionedTokenType::from(VersionedTokenType::Current(token_type));

		self.token_type_by_id.insert(&token_type_id, &versioned_token_type);

		self.token_type_assets_by_id.insert(&token_type_id, &assets);

    refund_deposit(env::storage_usage() - initial_storage_usage);
  }

	fn nft_cap_copies(
		&mut self,
		token_type_title: TokenTypeTitle,
		) {
		assert_eq!(env::predecessor_account_id(), self.tokens().owner_id, "Unauthorized");
		let token_type_id = self.token_type_by_title.get(&token_type_title).expect("no type");
		let mut versioned_token_type = self.token_type_by_id.get(&token_type_id).expect("no token");
		let mut token_type = versioned_token_type_to_token_type(versioned_token_type);
		token_type.metadata.copies = Some(token_type.tokens.len());
		versioned_token_type = VersionedTokenType::from(VersionedTokenType::Current(token_type));
		self.token_type_by_id.insert(&token_type_id, &versioned_token_type);
		// TODO: remove assets vector?
	}

	#[payable]
  fn nft_update_type(
        &mut self,
				token_type_title: TokenTypeTitle,
				metadata: Option<TokenMetadata>,
        royalty: Option<HashMap<AccountId, u32>>,
    ) {
		let initial_storage_usage = env::storage_usage();
    let owner_id = env::predecessor_account_id();
		assert_eq!(owner_id.clone(), self.tokens().owner_id, "Unauthorized");

		let token_type_id = self.token_type_by_title.get(&token_type_title).expect("no type");
		let mut versioned_token_type = self.token_type_by_id.get(&token_type_id).expect("no token");
		let mut token_type = versioned_token_type_to_token_type(versioned_token_type);

		if let Some(metadata) = metadata {
			if metadata.title.is_some() && metadata.title.clone().unwrap() != token_type.metadata.title.clone().unwrap() {
				assert_eq!(self.token_type_by_title.contains_key(&metadata.title.clone().unwrap()), false, "token_metadata.title already exists");

				// update token_type_by_title
				self.token_type_by_title.remove(&token_type.metadata.title.clone().unwrap());
				self.token_type_by_title.insert(&metadata.title.clone().unwrap(), &token_type_id);

				token_type.metadata.title = metadata.title;
			}
			// don't validate that description is_some, as description can be none
			token_type.metadata.description = metadata.description;
			// don't allow media updates for now
			// if metadata.media.is_some() {
			// 	token_type.metadata.media = metadata.media
			// }
			// don't allow to patch copies (this must go through `nft_cap_copies`)
			// don't allow to patch asset_distribution for now
		}
		if let Some(royalty) = royalty {
			token_type.royalty = royalty
		}
		// convert back to versioned
		versioned_token_type = VersionedTokenType::from(VersionedTokenType::Current(token_type));

		self.token_type_by_id.insert(&token_type_id, &versioned_token_type);

		let amt_to_refund = if env::storage_usage() > initial_storage_usage { env::storage_usage() - initial_storage_usage } else { initial_storage_usage - env::storage_usage() };
    refund_deposit(amt_to_refund);
  }

	#[payable]
	fn nft_mint_type(
		&mut self,
		token_type_title: TokenTypeTitle,
		receiver_id: AccountId,
    _metadata: Option<TokenMetadata>,
		) -> Token {

		assert_eq!(env::predecessor_account_id(), self.tokens().owner_id, "Unauthorized");

		let initial_storage_usage = env::storage_usage();

		// get token type & mint args
		let token_type_id = self.token_type_by_title.get(&token_type_title).expect("no type");
		let mut versioned_token_type = self.token_type_by_id.get(&token_type_id).expect("no token");
		let mut token_type = versioned_token_type_to_token_type(versioned_token_type);

		assert_eq!(&env::predecessor_account_id(), &token_type.owner_id, "not type owner");

		let num_tokens = token_type.tokens.len();
		let max_copies = token_type.metadata.copies.unwrap_or(u64::MAX);
		assert_ne!(num_tokens, max_copies, "type supply maxed");
		
		let mut final_metadata = TokenMetadata {
			title: None, // this remains None; NFT title is taken from token_type on enumeration so there is no need to store it on individual token metadata as well
			description: None, // this remains None; NFT description is taken from token_type on enumeration so there is no need to store it on individual token metadata as well
			media: None, // initiate as None. If this is an updated v1 type or a post-v1 type, meaning `assets` array is present, `media` will become the asset filename that can be located inside the token_type directory CID (this directory CID is stored as `media` on token_type). E.g. "cat.jpg" => on enumeration, TokenMetadata.media will read "<TokenType.media>/<TokenMetadata.media>", e.g. "abcd1234/cat.jpg"
			copies: None, // this remains None; NFT copies is taken from token_type on enumeration so there is no need to store it on individual token metadata as well
			extra: None, // this will become the "extra" (e.g. off-chain json) filename that can be located inside the token_type directory CID (this directory CID is stored as `media` on token_type). E.g. "cat.json" (doesn't have to correspond to filename of media asset, btw) => on enumeration, TokenMetadata.extra will read "<TokenType.media>/<TokenMetadata.extra>", e.g. "abcd1234/cat.json"
		};

		// get the assets vector for this token_type; let the fun begin!
		let mut assets = self.token_type_assets_by_id.get(&token_type_id).expect("No assets");

		let random_num = random_u128();
		let random_asset_idx = random_num % assets.len() as u128;
		let mut asset_detail = assets.get(random_asset_idx as usize).unwrap().clone();
		let asset_filename = asset_detail.get(0).unwrap().clone(); // first element is filename of media asset stored inside IPFS directory
		let mut supply_remaining: u64 = asset_detail.get(1).unwrap().clone().parse().unwrap(); // second element is supply remaining for this asset
		let extra_filename = asset_detail.get(2).unwrap().clone(); // third element is filename of "extra" (e.g. off-chain json) stored inside IPFS directory

		// cleanup
		if supply_remaining > 1 {
			// if there is supply remaining, decrement supply
			supply_remaining = supply_remaining - 1;
			asset_detail.remove(1);
			asset_detail.insert(1, supply_remaining.to_string());
			assets.remove(random_asset_idx as usize);
			assets.insert(random_asset_idx as usize, asset_detail);
		} else {
			// no supply left; remove asset from `assets` vector
			assets.remove(random_asset_idx as usize);
		}

		self.token_type_assets_by_id.insert(&token_type_id, &assets);

		if extra_filename.len() > 0 { // if extra_filename is not an empty string (empty string means no "extra" data is available for this NFT), attach "extra" filename to NFT metadata
			final_metadata.extra = Some(extra_filename.to_string());
		};
		
		final_metadata.media = Some(asset_filename.to_string());

		let token_id = format!("{}{}{}", &token_type_id, TOKEN_DELIMETER, num_tokens + 1);
		token_type.tokens.insert(&token_id);

		// convert back to versioned
		versioned_token_type = VersionedTokenType::from(VersionedTokenType::Current(token_type));
		self.token_type_by_id.insert(&token_type_id, &versioned_token_type);

		let token = self.tokens_mut().internal_mint(token_id.clone(), receiver_id.clone(), Some(VersionedTokenMetadata::from(VersionedTokenMetadata::Current(final_metadata))));

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

	#[payable]
	fn nft_batch_mint_type(
		&mut self,
		token_type_title: TokenTypeTitle,
		receiver_ids: Vec<AccountId>
	) -> Vec<Token>
	{
		// Don't allow batch minting for token types with more than one asset because the same random
		// number seed will be used for all mints
		let token_type_id = self.token_type_by_title.get(&token_type_title).expect("no type");
		let versioned_token_type = self.token_type_by_id.get(&token_type_id).expect("no token");
		let token_type = versioned_token_type_to_token_type(versioned_token_type);
		let asset_count = token_type.asset_count;
		log!(format!("asset_count: {}", asset_count));
		assert!(asset_count == 1, "batch minting not allowed for token types with more than one asset");

		assert_eq!(env::predecessor_account_id(), self.tokens().owner_id, "Unauthorized");
		let mut tokens = Vec::new();

		// Check length of receiver_ids
		assert!(receiver_ids.len() <= 1000, "receiver_ids must be less than or equal to 1000");

		for receiver_id in receiver_ids {
			tokens.push(self.nft_mint_type(token_type_title.clone(), receiver_id.clone(), None));
		}
		tokens
	}

	#[payable]
	fn nft_delete_type(
		&mut self,
		token_type_title: TokenTypeTitle,
	) {
		let initial_storage_usage = env::storage_usage();
    let owner_id = env::predecessor_account_id();
		assert_eq!(owner_id.clone(), self.tokens().owner_id, "Unauthorized");

		let token_type_id = self.token_type_by_title.get(&token_type_title).expect("no type");
		let versioned_token_type = self.token_type_by_id.get(&token_type_id).expect("no token");
		let token_type = versioned_token_type_to_token_type(versioned_token_type);
		
		// check if there are any tokens (can't delete if there are minted NFTs)
		let num_tokens = token_type.tokens.len();
		assert!(num_tokens < 1, "Cannot delete a type that contains tokens (found {} tokens)", num_tokens);

		// remove from token_type_by_id
		self.token_type_by_id.remove(&token_type_id);
		// remove from token_type_by_title
		self.token_type_by_title.remove(&token_type_title);
		// remove from token_type_assets_by_id
		self.token_type_assets_by_id.remove(&token_type_id);

		let amt_to_refund = if env::storage_usage() > initial_storage_usage { env::storage_usage() - initial_storage_usage } else { initial_storage_usage - env::storage_usage() };
    refund_deposit(amt_to_refund);
	}
}