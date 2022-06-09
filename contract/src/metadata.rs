use crate::*;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::Base64VecU8;
use near_sdk::{require, AccountId};
use near_sdk::serde::{Deserialize, Serialize};
use std::collections::HashMap;
// use std::cmp::PartialEq;

/// This spec can be treated like a version of the standard.
pub const NFT_METADATA_SPEC: &str = "nft-1.0.0";

/// Note that token IDs for NFTs are strings on NEAR. It's still fine to use autoincrementing numbers as unique IDs if desired, but they should be stringified. This is to make IDs more future-proof as chain-agnostic conventions and standards arise, and allows for more flexibility with considerations like bridging NFTs across chains, etc.
pub type TokenId = String;

/// In this implementation, the Token struct takes two extensions standards (metadata and approval) as optional fields, as they are frequently used in modern NFTs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenV1 {
    pub token_id: TokenId,
    pub owner_id: AccountId,
    pub metadata: Option<TokenMetadataV1>,
    pub approved_account_ids: Option<HashMap<AccountId, u64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct Token {
    pub token_id: TokenId,
    pub owner_id: AccountId,
    pub metadata: Option<VersionedTokenMetadata>,
    pub approved_account_ids: Option<HashMap<AccountId, u64>>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum VersionedToken {
    // V1(TokenV1),
    Current(Token),
}

pub fn versioned_token_to_token(versioned_token: VersionedToken) -> Token {
    match versioned_token {
        VersionedToken::Current(current) => current,
        // VersionedToken::V1(v1) => Token {
        //     token_id: v1.token_id,
        //     owner_id: v1.owner_id,
        //     metadata: Some(VersionedTokenMetadata::from(VersionedTokenMetadata::V1(v1.metadata.unwrap()))),
        //     approved_account_ids: v1.approved_account_ids,
        // }
    }
}

/// Metadata for the NFT contract itself.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct NFTContractMetadata {
    pub spec: String,              // required, essentially a version like "nft-1.0.0"
    pub name: String,              // required, ex. "Mosaics"
    pub symbol: String,            // required, ex. "MOSIAC"
    pub icon: Option<String>,      // Data URL
    pub base_uri: Option<String>, // Centralized gateway known to have reliable access to decentralized storage assets referenced by `reference` or `media` URLs
    pub reference: Option<String>, // URL to a JSON file with more info
    pub reference_hash: Option<Base64VecU8>, // Base64-encoded sha256 hash of JSON from reference field. Required if `reference` is included.
}

/// Metadata on the individual token level.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenMetadataV1 { // OLD TOKEN METADATA
    pub title: Option<String>, // ex. "Arch Nemesis: Mail Carrier" or "Parcel #5055"
    pub description: Option<String>, // free-form description
    pub media: Option<String>, // URL to associated media, preferably to decentralized, content-addressed storage
    pub copies: Option<u64>, // number of copies of this set of metadata in existence when token was minted.
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenMetadata { // CURRENT TOKEN METADATA
    pub title: Option<String>, // ex. "Arch Nemesis: Mail Carrier" or "Parcel #5055"
    pub description: Option<String>, // free-form description
    pub media: Option<String>, // URL to associated media, preferably to decentralized, content-addressed storage
    pub copies: Option<u64>, // number of copies of this set of metadata in existence when token was minted.
    // NEW FIELDS
    pub asset_id: Option<String>,
    pub filetype: Option<String>,
    pub extra: Option<String>, // anything extra the NFT wants to store on-chain. Can be stringified JSON.
    // TODO: add `updatedAt`? other fields?
}

#[derive(Debug, Clone, PartialEq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum VersionedTokenMetadata {
    // V1(TokenMetadataV1),
    Current(TokenMetadata),
}
/// Convert TokenMetadataV1 to TokenMetadata
impl From<TokenMetadataV1> for TokenMetadata {
	fn from(v1: TokenMetadataV1) -> Self {
        // match metadata {
        //     UpgradableTokenMetadata::V2(metadata) => metadata,
        //     UpgradableTokenMetadata::V1(v1) => TokenMetadata {
        //             title: v1.title,
        //             description: v1.description,
        //             media: v1.media,
        //             copies: v1.copies,
        //             asset_id: None,
        //             filetype: None,
        //             extra: None,
        //     }
        // }
		TokenMetadata {
			title: v1.title,
			description: v1.description,
			media: v1.media,
			copies: v1.copies,
			asset_id: None,
			filetype: None,
			extra: None,
	    }
	}
}

/// Convert VersionedTokenMetadata to TokenMetadata
// impl From<VersionedTokenMetadata> for TokenMetadata {
// 	fn from(metadata: VersionedTokenMetadata) -> Self {
//         match metadata {
//             VersionedTokenMetadata::Current(metadata) => metadata,
//             VersionedTokenMetadata::V1(v1) => TokenMetadata {
//                     title: v1.title,
//                     description: v1.description,
//                     media: v1.media,
//                     copies: v1.copies,
//                     asset_id: None,
//                     filetype: None,
//                     extra: None,
//             }
//         }
// 	}
// }

pub fn versioned_token_metadata_to_token_metadata(versioned_metadata: VersionedTokenMetadata) -> TokenMetadata {
    match versioned_metadata {
        VersionedTokenMetadata::Current(current) => current,
        // VersionedTokenMetadata::V1(v1) => TokenMetadata {
        //     title: v1.title,
        //     description: v1.description,
        //     media: v1.media,
        //     copies: v1.copies,
        //     asset_id: None,
        //     filetype: None,
        //     extra: None,
        // }
    }
}

// impl From<UpgradableTokenMetadata> for TokenMetadata {
// 	fn from(metadata: UpgradableTokenMetadata) -> Self {
//         match metadata {
//             UpgradableTokenMetadata::V2(metadata) => metadata,
//             UpgradableTokenMetadata::V1(v1) => TokenMetadata {
//                 title: v1.title,
//                 description: v1.description,
//                 media: v1.media,
//                 copies: v1.copies,
//                 asset_id: None,
//                 filetype: None,
//                 extra: None,
//             }
//         }
// 	}
// }

/// Offers details on the contract-level metadata.
pub trait NonFungibleTokenMetadataProvider {
    fn nft_metadata(&self) -> NFTContractMetadata;
}

#[near_bindgen]
impl NonFungibleTokenMetadataProvider for Contract {
    fn nft_metadata(&self) -> NFTContractMetadata {
        self.metadata.get().unwrap()
    }
}

impl NFTContractMetadata {
    pub fn assert_valid(&self) {
        require!(self.spec == NFT_METADATA_SPEC, "Spec is not NFT metadata");
        require!(
            self.reference.is_some() == self.reference_hash.is_some(),
            "Reference and reference hash must be present"
        );
        if let Some(reference_hash) = &self.reference_hash {
            require!(reference_hash.0.len() == 32, "Hash has to be 32 bytes");
        }
    }
}