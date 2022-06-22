use crate::*;

// pub type AssetDetail = Vec<u64>; // E.g. [1, 10] where 1 is asset_id and 10 is supply_remaining
pub type AssetDetail = Vec<String>; // E.g. ["1", "10"] where 1 is asset_id and 10 is supply_remaining, or ["cat", "10"] where "cat" is asset_id and 10 is supply_remaining
pub type TokenTypeId = u64;
pub type TokenTypeTitle = String;

/// methods for NFT type (otherwise known as "series")
pub trait NonFungibleTokenType {

  /// Create a new NFT type (aka series)
  fn nft_create_type(
      &mut self,
      metadata: TokenTypeMetadata,
      royalty: HashMap<AccountId, u32>,
			asset_count: u64,
      asset_filetypes: Vec<String>,
      asset_distribution: Option<Vec<AssetDetail>>, // must be present unless type is fully generative
			json: bool,
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
				asset_count: u64,
				asset_filetypes: Vec<String>,
				asset_distribution: Option<Vec<AssetDetail>>, // may alternatively be able to use near_sdk_collections TreeMap for optimized storage
				json: bool,
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

		// MORE VALIDATION
		// "non-generative" = numbered editions of single media asset (e.g. 1 asset for 100 copies)
		// "fully-generative" = unique asset for each NFT (e.g. 100 assets for 100 copies)
		// "semi-generative" = multi-edition (multiple assets but not 1:1 e.g. 5 assets distributed among 100 copies)
		
		let is_non_generative = asset_count == 1 && metadata.copies.unwrap() > 1; 
		let is_fully_generative = asset_count == metadata.copies.unwrap();
		let is_semi_generative = !is_non_generative && !is_fully_generative;

		let asset_filetypes_len = asset_filetypes.len();

		// For all series types (non-, semi- and fully-generative), asset_filetypes should be a vector with a length of 1 OR asset_distribution (otherwise, we will not know which filetype to associate with NFT media at time of mint)
		assert!(asset_filetypes_len == 1 || asset_filetypes_len == asset_count as usize, "asset_filetypes should be a vector with a length of 1 OR asset_count.");

		if is_semi_generative {
			// this is a semi-generative series (multi-edition)
			// As such:
			// 1. asset_distribution vector must be present and cannot be empty
			assert!(asset_distribution.is_some(), "asset_distribution must be provided for semi-generative series");
			let asset_distribution = asset_distribution.clone().unwrap();
			assert!(!asset_distribution.is_empty(), "asset_distribution must not be empty");

			// 2. length of asset_distribution vector must equal asset_count
			assert!(asset_distribution.len() == asset_count as usize, "for semi-generative series, length of asset_distribution vector must equal asset_count");

			// 3. each asset_distribution element must contain two integers: asset_id and total_supply
			// sum of total_supply must be equal to `metadata.copies`
			let mut total_supply = 0 as u64;
			for distr_detail in asset_distribution {
				let asset_id = distr_detail.get(0);
				assert!(asset_id.is_some(), "Asset ID must be provided");
				let supply_remaining: u64 = distr_detail.get(1).unwrap().clone().parse().unwrap();
				total_supply = total_supply + supply_remaining;
			}
			assert!(total_supply == metadata.copies.unwrap(), "Total supply must equal copies. Received {} total supply & {} copies", total_supply, metadata.copies.unwrap());
		} else {
			// shared validation for fully-generative and non-generative series
			// 1. asset_distribution should not be provided (by definition, each NFT in a fully-generative type/series has unique media; and for non-generative series, there is only one asset to be used across all NFTs, therefore there is no concept of distribution.)
			assert!(asset_distribution.is_none(), "asset_distribution should not be provided for fully-generative or non-generative series");
		}

		let token_type_id = self.token_type_by_id.len() + 1;
		assert!(self.token_type_by_title.insert(&title.unwrap(), &token_type_id).is_none(), "token_metadata.title exists");

		self.token_type_by_id.insert(&token_type_id, &TokenType{
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
		});

		let mint_args = TokenTypeMintArgs{
			asset_filetypes,
			asset_distribution: asset_distribution.unwrap_or(Vec::new()),
			asset_count: Some(asset_count),
			has_json: Some(json),
		};

		self.token_type_mint_args_by_id.insert(&token_type_id, &VersionedTokenTypeMintArgs::from(VersionedTokenTypeMintArgs::Current(mint_args)));

    refund_deposit(env::storage_usage() - initial_storage_usage);
  }

	fn nft_cap_copies(
		&mut self,
		token_type_title: TokenTypeTitle,
		) {
		assert_eq!(env::predecessor_account_id(), self.tokens().owner_id, "Unauthorized");
		let token_type_id = self.token_type_by_title.get(&token_type_title).expect("no type");
		let mut token_type = self.token_type_by_id.get(&token_type_id).expect("no token");
		token_type.metadata.copies = Some(token_type.tokens.len());
		self.token_type_by_id.insert(&token_type_id, &token_type);
		// TODO: remove asset_distribution & asset_filetypes vectors?
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
		let mut token_type = self.token_type_by_id.get(&token_type_id).expect("no token");

		if let Some(metadata) = metadata {
			if metadata.title.is_some() {
				token_type.metadata.title = metadata.title
			}
			// don't validate that description is_some, as description can be none
			token_type.metadata.description = metadata.description;
			// don't allow media updates for now
			// if metadata.media.is_some() {
			// 	token_type.metadata.media = metadata.media
			// }
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

		assert_eq!(env::predecessor_account_id(), self.tokens().owner_id, "Unauthorized");

		let initial_storage_usage = env::storage_usage();

		// get token type & mint args
		let token_type_id = self.token_type_by_title.get(&token_type_title).expect("no type");
		let mut token_type = self.token_type_by_id.get(&token_type_id).expect("no token");
		assert_eq!(&env::predecessor_account_id(), &token_type.owner_id, "not type owner");

		let num_tokens = token_type.tokens.len();
		let max_copies = token_type.metadata.copies.unwrap_or(u64::MAX);
		assert_ne!(num_tokens, max_copies, "type supply maxed");

		let copies = token_type.metadata.copies.unwrap();
		
		// TODO finish adding custom metadata (if provided) to final_metadata
		// you can add custom metadata to each token here
		// make sure you update self.nft_token to "patch" over the type metadata
		let mut final_metadata = TokenMetadata {
			title: None, // ex. "Arch Nemesis: Mail Carrier" or "Parcel #5055"
			description: None, // free-form description
			media: None, // URL to associated media, preferably to decentralized, content-addressed storage
			copies: None, // number of copies of this set of metadata in existence when token was minted.
			asset_id: None,
			filetype: None,
			extra: None,
		};

		let mint_args = self.token_type_mint_args_by_id.get(&token_type_id);

		if let Some(VersionedTokenTypeMintArgs::Current(mut token_type_mint_args)) = mint_args {
			let mut asset_id = "1".to_string();
			let num_filetypes = token_type_mint_args.asset_filetypes.len();
			let mut file_type = token_type_mint_args.asset_filetypes.get(0).unwrap().clone();

			// "non-generative" = numbered editions of single media asset (e.g. 1 asset for 100 copies)
			// "fully-generative" = unique asset for each NFT (e.g. 100 assets for 100 copies)
			// "semi-generative" = multi-edition (multiple assets but not 1:1 e.g. 5 assets distributed among 100 copies)

			if token_type_mint_args.asset_count == Some(copies) {
				// fully-generative case (unique media per NFT; could be 1/1 or 1/10,000)
				asset_id = (num_tokens + 1).to_string();
				if num_filetypes > 1 {
					// fully-generative case with specified filetype for each asset
					// get filetype at index of this asset
					file_type = token_type_mint_args.asset_filetypes.get(num_tokens as usize).unwrap().clone();
				}
			} else {
				if token_type_mint_args.asset_count == Some(1) {
					// non-generative case
					// nothing to do; asset_id stays as 1, and file_type is first element in filetypes vector. Move on!
				} else {
					// semi-generative case
					// use asset_distribution vector to determine asset to associate with this NFT
					let random_num = random_u128();
					let idx = random_num % token_type_mint_args.asset_distribution.len() as u128;
					let mut asset = token_type_mint_args.asset_distribution.get(idx as usize).unwrap().clone();
					asset_id = asset.get(0).unwrap().clone();
					let mut supply_remaining: u64 = asset.get(1).unwrap().clone().parse().unwrap();

					if token_type_mint_args.asset_filetypes.len() > 1 {
						file_type = token_type_mint_args.asset_filetypes.get(idx as usize).unwrap().to_string();
					}

					// cleanup
					if supply_remaining > 1 {
						// decrement supply
						supply_remaining = supply_remaining - 1;
						asset.remove(1);
						asset.insert(1, supply_remaining.to_string());
						token_type_mint_args.asset_distribution.remove(idx as usize);
						token_type_mint_args.asset_distribution.insert(idx as usize, asset);
					} else {
						// no supply left; remove asset from asset distribution list and remove filetype from asset filetypes list (these need to remain NSYNC)
						token_type_mint_args.asset_distribution.remove(idx as usize);
						token_type_mint_args.asset_filetypes.remove(idx as usize);
					}
				}
			}

			if token_type_mint_args.has_json == Some(true) {
				final_metadata.extra = Some(format!("{}.json", asset_id.to_string()))
			};
			
			self.token_type_mint_args_by_id.insert(&token_type_id, &VersionedTokenTypeMintArgs::from(VersionedTokenTypeMintArgs::Current(token_type_mint_args)));

			final_metadata.asset_id = Some(asset_id.to_string());
			final_metadata.filetype = Some(file_type);
		}

		let token_id = format!("{}{}{}", &token_type_id, TOKEN_DELIMETER, num_tokens + 1);
		token_type.tokens.insert(&token_id);
		self.token_type_by_id.insert(&token_type_id, &token_type);

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
	fn nft_delete_type(
		&mut self,
		token_type_title: TokenTypeTitle,
	) {
		let initial_storage_usage = env::storage_usage();
    let owner_id = env::predecessor_account_id();
		assert_eq!(owner_id.clone(), self.tokens().owner_id, "Unauthorized");

		let token_type_id = self.token_type_by_title.get(&token_type_title).expect("no type");
		let token_type = self.token_type_by_id.get(&token_type_id).expect("no token");
		// check if there are any tokens (can't delete if there are minted NFTs)
		let num_tokens = token_type.tokens.len();
		assert!(num_tokens < 1, "Cannot delete a type that contains tokens (found {} tokens)", num_tokens);

		// remove from token_type_by_id
		self.token_type_by_id.remove(&token_type_id);
		// remove from token_type_by_title
		self.token_type_by_title.remove(&token_type_title);
		// remove from token_type_mint_args_by_id
		self.token_type_mint_args_by_id.remove(&token_type_id); // TODO: will this error on contracts where self.token_type_mint_args_by_id is not present?

		let amt_to_refund = if env::storage_usage() > initial_storage_usage { env::storage_usage() - initial_storage_usage } else { initial_storage_usage - env::storage_usage() };
    refund_deposit(amt_to_refund);
	}
}