use crate::*;

use near_sdk::json_types::{U128};

/// "getter" methods for Contract
trait NonFungibleTokenEnumeration {
  /// get total number of NFTs minted across all series (types) in this contract
  fn nft_total_supply(&self) -> U128;

  /// get token objects for all NFTs on this contract, using `from_index` as starting point (if provided) and limiting count to `limit` (if provided)
  fn nft_tokens(&self, from_index: Option<U128>, limit: Option<u64>) -> Vec<Token>;

  /// get all token IDs on this contract, using `from_index` as starting point (if provided) and limiting count to `limit` (if provided).
  /// Added for the purposes of upgrading metadata for existing tokens
  fn nft_token_ids(&self, from_index: Option<U128>, limit: Option<u64>) -> Vec<String>;

  /// get number of NFTs owned by a specified owner (across all series/types)
  fn nft_supply_for_owner(self, account_id: AccountId) -> U128;

  /// get token objects for all NFTs owned by a specified owner (across all series/types)
  fn nft_tokens_for_owner(
    &self,
    account_id: AccountId,
    from_index: Option<U128>,
    limit: Option<u64>,
  ) -> Vec<Token>;

  /// get info on a specific type/series, by title
  fn nft_get_type(&self, token_type_title: TokenTypeTitle) -> TokenTypeJson;

  /// get type format as [TOKEN_DELIMETER, TITLE_DELIMETER, EDITION_DELIMETER]
  fn nft_get_type_format(&self) -> (char, &'static str, &'static str);

  /// get info on all types/series contained within this contract
  fn nft_get_types(
    &self,
    from_index: Option<U128>,
    limit: Option<u64>
  ) -> Vec<TokenTypeJson>;

  /// get number of NFTs minted (existing!) for a specified type/series
  fn nft_supply_for_type(
    &self,
    token_type_title: TokenTypeTitle,
  ) -> U64;

  /// get token objects for all NFTs of a specified type/series
  fn nft_tokens_by_type(
    &self,
    token_type_title: TokenTypeTitle,
    from_index: Option<U128>,
    limit: Option<u64>
  ) -> Vec<Token>;

}

#[near_bindgen]
impl NonFungibleTokenEnumeration for Contract {

  fn nft_total_supply(&self) -> U128 {
    (self.tokens().owner_by_id.len() as u128).into()
  }
  
  fn nft_tokens(&self, from_index: Option<U128>, limit: Option<u64>) -> Vec<Token> {
    // Get starting index, whether or not it was explicitly given.
    // Defaults to 0 based on the spec:
    // https://nomicon.io/Standards/NonFungibleToken/Enumeration.html#interface
    let tokens = self.tokens();
    let start_index: u128 = from_index.map(From::from).unwrap_or_default();
    assert!(
        (tokens.owner_by_id.len() as u128) >= start_index,
        "Out of bounds, please use a smaller from_index."
    );
    let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
    assert_ne!(limit, 0, "Cannot provide limit of 0.");
    tokens.owner_by_id
        .iter()
        .skip(start_index as usize)
        .take(limit)
        .map(|(token_id, _)| self.nft_token(token_id).unwrap())
        .collect()
  }

  /// Added for the purposes of upgrading metadata for existing tokens
  fn nft_token_ids(&self, from_index: Option<U128>, limit: Option<u64>) -> Vec<String> {
    // Get starting index, whether or not it was explicitly given.
    // Defaults to 0 based on the spec:
    // https://nomicon.io/Standards/NonFungibleToken/Enumeration.html#interface
    let tokens = self.tokens();
    let start_index: u128 = from_index.map(From::from).unwrap_or_default();
    assert!(
        (tokens.owner_by_id.len() as u128) >= start_index,
        "Out of bounds, please use a smaller from_index."
    );
    let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
    assert_ne!(limit, 0, "Cannot provide limit of 0.");
    tokens.owner_by_id
        .iter()
        .skip(start_index as usize)
        .take(limit)
        .map(|(token_id, _)| token_id)
        .collect()
  }
  
  fn nft_supply_for_owner(self, account_id: AccountId) -> U128 {
      let tokens_per_owner = self.tokens().tokens_per_owner.as_ref().expect(
          "Could not find tokens_per_owner when calling a method on the enumeration standard.",
      );
      tokens_per_owner
          .get(&account_id)
          .map(|account_tokens| U128::from(account_tokens.len() as u128))
          .unwrap_or(U128(0))
  }
  
  fn nft_tokens_for_owner(
        &self,
        account_id: AccountId,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<Token> {
        let tokens_per_owner = self.tokens().tokens_per_owner.as_ref().expect(
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
  
  fn nft_get_type(&self, token_type_title: TokenTypeTitle) -> TokenTypeJson {
    let versioned_token_type = self.token_type_by_id.get(&self.token_type_by_title.get(&token_type_title).expect("no type")).expect("no type");
		let token_type = versioned_token_type_to_token_type(versioned_token_type);
    TokenTypeJson {
      metadata: token_type.metadata,
      owner_id: token_type.owner_id,
      royalty: token_type.royalty,
    }
  }
  
  fn nft_get_type_format(&self) -> (char, &'static str, &'static str) {
    (TOKEN_DELIMETER, TITLE_DELIMETER, EDITION_DELIMETER)
  }
  
  fn nft_get_types(
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
        
    let token_types = self.token_type_by_id.iter()
      .skip(start_index as usize)
      .take(limit)
      .map(|(_, versioned_token_type)| {
        let token_type = versioned_token_type_to_token_type(versioned_token_type);
        TokenTypeJson {
          metadata: token_type.metadata,
          owner_id: token_type.owner_id,
          royalty: token_type.royalty,
        }
      })
      .collect();
      token_types
  }
  
  fn nft_supply_for_type(
        &self,
        token_type_title: TokenTypeTitle,
    ) -> U64 {
        let versioned_token_type = self.token_type_by_id.get(&self.token_type_by_title.get(&token_type_title).expect("no type")).expect("no type");
        let token_type = versioned_token_type_to_token_type(versioned_token_type);
        token_type.tokens.len().into()
  }
  
  fn nft_tokens_by_type(
    &self,
    token_type_title: TokenTypeTitle,
    from_index: Option<U128>,
    limit: Option<u64>
  ) -> Vec<Token> {
    let start_index: u128 = from_index.map(From::from).unwrap_or_default();
    let versioned_token_type = self.token_type_by_id.get(&self.token_type_by_title.get(&token_type_title).expect("no type")).expect("no type");
    let token_type = versioned_token_type_to_token_type(versioned_token_type);
    let tokens = token_type.tokens;
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