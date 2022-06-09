use crate::*;

use near_sdk::json_types::{U128};

pub trait NonFungibleTokenRoyalty {
  //calculates the payout for a token given the passed in balance. This is a view method
  fn nft_payout(&self, token_id: TokenId, balance: U128, max_len_payout: u32) -> Payout;

  //transfers the token to the receiver ID and returns the payout object that should be payed given the passed in balance. 
  fn nft_transfer_payout(
    &mut self,
    receiver_id: AccountId,
    token_id: TokenId,
    approval_id: u64,
    memo: Option<String>,
    balance: Option<U128>,
    max_len_payout: Option<u32>,
  ) -> Option<Payout>;
} 

#[near_bindgen]
impl NonFungibleTokenRoyalty for Contract {
  	//calculates the payout for a token given the passed in balance. This is a view method
	fn nft_payout(&self, token_id: TokenId, balance: U128, max_len_payout: u32) -> Payout {
		//get the token object
		// let token = versioned_token_to_token(self.nft_token(token_id.clone()).expect("no token"));
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
				payout_object.payout.insert(key, royalty_to_payout(*v, balance_u128));
				total_perpetual += *v;
			}
		}

		// payout to previous owner who gets 100% - total perpetual royalties
		let owner_payout = royalty_to_payout(10000 - total_perpetual, balance_u128);
		if u128::from(owner_payout) > 0 {
			payout_object.payout.insert(owner_id, owner_payout);
		}

		//return the payout object
		payout_object
	}

	/// CUSTOM royalties payout
	#[payable]
	fn nft_transfer_payout(
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
			self.nft_transfer(receiver_id.clone(), token_id.clone(), Some(approval_id), memo);
			prev_token
		};
		// let previous_token = versioned_token_to_token(previous_token_versioned);

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

}