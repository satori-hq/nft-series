use crate::*;

use near_sdk::{ log };

pub type AssetDetail = Vec<u64>; // E.g. [1, 10] where 1 is asset_id and 10 is supply_remaining
pub type TokenTypeId = u64;
pub type TokenTypeTitle = String;

/// methods for NFT type (otherwise known as "series")
pub trait NonFungibleTokenType {

  /// Create a new NFT type (aka series)
  fn nft_create_type(
      &mut self,
      metadata: TokenMetadata,
      royalty: HashMap<AccountId, u32>,
			asset_count: u64,
      asset_filetypes: Vec<String>,
      asset_distribution: Option<Vec<AssetDetail>>, // must be present unless type is fully generative
  );

  /// Cap copies of an existing NFT type/series to currently minted supply
	fn nft_cap_copies(
		&mut self,
		token_type_title: TokenTypeTitle,
	);

  /// Update any metadata or royalty fields of an existing NFT type/series EXCEPT `copies`
  fn nft_patch_type(
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
}

#[near_bindgen]
impl NonFungibleTokenType for Contract {
  #[payable]
  fn nft_create_type(
        &mut self,
        metadata: TokenMetadata,
        royalty: HashMap<AccountId, u32>,
				asset_count: u64,
				asset_filetypes: Vec<String>,
				asset_distribution: Option<Vec<AssetDetail>>,
    ) {
		let initial_storage_usage = env::storage_usage();
    let owner_id = env::predecessor_account_id();
		assert_eq!(owner_id.clone(), self.tokens.owner_id, "Unauthorized");
		let title = metadata.title.clone();
		assert!(title.is_some(), "token_metadata.title is required");

		let is_non_generative = asset_count == 1 && metadata.copies.unwrap() > 1;
		let is_fully_generative = asset_count == metadata.copies.unwrap();
		let is_semi_generative = !is_non_generative && !is_fully_generative;

		if is_fully_generative {
			// this is a fully-generative type/series (unique media per copy; could be 1 copy or 10,000 copies). As such: 
			// 1. asset_filetypes should be a vector with a length of 1 OR asset_count
			// 2. asset_distribution should not be provided (by definition, each NFT in a fully-generative type/series has unique media)
			let asset_filetypes_len = asset_filetypes.len();
			assert!(asset_filetypes_len == 1 || asset_filetypes_len == asset_count as usize, "For fully-generative type/series, asset_filetypes should be a vector with a length of 1 OR asset_count.");
			assert!(asset_distribution.is_none(), "asset_distribution should not be provided for fully-generative type/series");
		} else if is_non_generative {
			// this is a non-generative type/series. As such:
			// 1. asset_distribution should not be provided (there is only one asset to be used across all NFTs, therefore there is no concept of distribution)
			assert!(asset_distribution.is_none(), "asset_distribution should not be provided for non-generative type/series");
		} else {
			// this is a semi-generative series. As such:
			// 1. asset_distribution must be present and cannot be empty
			// 2. asset_distribution and asset_filetypes vectors must be the same length
			let distribution = asset_distribution.clone().unwrap();
			assert!(!distribution.is_empty(), "asset_distribution must not be empty");
			assert_eq!(asset_filetypes.len(), distribution.len(), "asset_filetypes and asset_distribution must be same length");
		}

		if is_semi_generative {
			// validate asset_distribution elements (must contain two integers: asset_id and total_supply;
			let mut total_supply = 0 as u64;
			let distribution = asset_distribution.clone().unwrap();
			for distr_detail in distribution {
				let asset_id = distr_detail.get(0);
				assert!(asset_id.is_some(), "Asset ID must be provided");
				let supply_remaining = distr_detail.get(1).unwrap().clone();
				total_supply = total_supply + supply_remaining;
			}
			assert!(total_supply == metadata.copies.unwrap(), "Total supply must equal copies. Received {} total supply & {} copies", total_supply, metadata.copies.unwrap());
		}

		let token_type_id = self.token_type_by_id.len() + 1;
		assert!(self.token_type_by_title.insert(&title.unwrap(), &token_type_id).is_none(), "token_metadata.title exists");
		self.token_type_by_id.insert(&token_type_id, &TokenType{
			metadata,
			owner_id,
			royalty,
			asset_count,
			asset_filetypes,
			asset_distribution: asset_distribution.unwrap_or(Vec::new()),
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
		log!(format!("Used gas: {:#?}", env::used_gas()))
  }

	fn nft_cap_copies(
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
  fn nft_patch_type(
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
			// don't allow to patch copies (this must go through `nft_cap_copies`)
		}
		if let Some(royalty) = royalty {
			token_type.royalty = royalty
		}
		self.token_type_by_id.insert(&token_type_id, &token_type);

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

		assert_eq!(env::predecessor_account_id(), self.tokens.owner_id, "Unauthorized");

		let initial_storage_usage = env::storage_usage();

		let token_type_id = self.token_type_by_title.get(&token_type_title).expect("no type");
		let mut token_type = self.token_type_by_id.get(&token_type_id).expect("no token");
		assert_eq!(&env::predecessor_account_id(), &token_type.owner_id, "not type owner");

		let num_tokens = token_type.tokens.len();
		let max_copies = token_type.metadata.copies.unwrap_or(u64::MAX);
		let asset_count = token_type.asset_count;
		let copies = token_type.metadata.copies.unwrap();
		assert_ne!(num_tokens, max_copies, "type supply maxed");

		let mut asset_id = 1;
		let num_filetypes = token_type.asset_filetypes.len();
		let mut file_type = token_type.asset_filetypes.get(0).unwrap().clone();

		if asset_count == copies {
			// fully-generative case (unique media per NFT; could be 1/1 or 1/10,000)
			asset_id = num_tokens + 1;
			if num_filetypes > 1 {
				// fully-generative case with specified filetype for each asset
				// get filetype at index of this asset
				file_type = token_type.asset_filetypes.get((asset_id - 1) as usize).unwrap().clone();
			}
		} else {
			if asset_count == 1 {
				// non-generative case
				// nothing to do; move on
			} else {
				// semi-generative case
				// use asset_distribution vector to determine asset to associate with this NFT
				let random_num = env::block_timestamp();
				// log!(format!("asset distribution line 199: {:#?}", token_type.asset_distribution));
				let idx = random_num % token_type.asset_distribution.len() as u64;
				let mut asset = token_type.asset_distribution.get(idx as usize).unwrap().clone();
				// log!(format!("asset line 201: {:#?}", asset));
				asset_id = asset.get(0).unwrap().clone();
				let mut supply_remaining = asset.get(1).unwrap().clone();
				file_type = token_type.asset_filetypes.get(idx as usize).unwrap().to_string();
				// log!(format!("asset id line 207: {}", asset_id));

				// cleanup
				if supply_remaining > 1 {
					// decrement supply
					supply_remaining = supply_remaining - 1;
					asset.remove(1);
					asset.insert(1, supply_remaining);
					// log!(format!("asset: {:#?}", asset));
					token_type.asset_distribution.remove(idx as usize);
					token_type.asset_distribution.insert(idx as usize, asset);
					// token_type.asset_distribution.insert(index: usize, element: T)
				} else {
					// no supply left; remove asset from asset distribution list and remove filetype from asset filetypes list (these need to remain NSYNC)
					// TODO: is there a chance, e.g. in O(n) worst case, that this could run out of gas and never be removed?
					// log!(format!("asset distribution line 216: {:#?}", token_type.asset_distribution));
					token_type.asset_distribution.remove(idx as usize);
					token_type.asset_filetypes.remove(idx as usize);
					// log!(format!("asset distribution line 218: {:#?}", token_type.asset_distribution));
				}
			}
		}

		// log!(format!("asset id line 229: {}", asset_id));

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
			asset_id: Some(asset_id.to_string()),
			file_type: Some(file_type),
		});
		// log!(format!("final_metadata: {:#?}", final_metadata));

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

		// log!(format!("token line 263: {:#?}", token));
			
		token
	}
}