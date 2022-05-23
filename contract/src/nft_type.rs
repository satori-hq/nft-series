use crate::*;

/// methods for NFT type (otherwise known as "series")
pub trait NonFungibleTokenType {

  /// Create a new NFT type (aka series)
  fn nft_create_type(
      &mut self,
      metadata: TokenMetadata,
      royalty: HashMap<AccountId, u32>,
      asset_filetypes: Vec<String>,
      asset_distribution: Vec<AssetDetail>,
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
				asset_filetypes: Vec<String>,
				asset_distribution: Vec<AssetDetail>,
    ) {
		let initial_storage_usage = env::storage_usage();
        let owner_id = env::predecessor_account_id();
		assert_eq!(owner_id.clone(), self.tokens.owner_id, "Unauthorized");
		let title = metadata.title.clone();
		assert!(title.is_some(), "token_metadata.title is required");
		assert!(!asset_distribution.is_empty(), "asset_distribution must not be empty");
		assert!(asset_distribution.len() == 1, "Only 1 asset per type supported at this time");
		assert_eq!(asset_filetypes.len(), asset_distribution.len(), "asset_filetypes and asset_distribution must be same length");

		// validate asset_distribution elements (must contain two integers: asset_id and supply_remaining, and total supply_remaining must equal copies)
		let mut total_supply = 0;
		for distr_detail in &asset_distribution {
			let asset_id = distr_detail.get(0);
			assert!(asset_id.is_some(), "Asset ID must be provided");
			let supply_remaining = distr_detail.get(1).unwrap().clone();
			total_supply = total_supply + supply_remaining;
		}
		assert!(total_supply == metadata.copies.unwrap(), "Total supply must equal copies");

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
		assert_ne!(num_tokens, max_copies, "type supply maxed");

		let mut asset_id = 1;
		let mut file_type = token_type.asset_filetypes.get(0).unwrap().clone();

		if token_type.asset_distribution.len() != 1 {
			// generate random number... do the generative minting thing, updating asset_id and file_extension above, and mutating asset_distribution array (& setting on type)
			// not supported yet (a type can't be created with more than one media asset currently)
		}

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
			// asset_id,
			// file_type,
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
}