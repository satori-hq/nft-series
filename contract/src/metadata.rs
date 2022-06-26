use crate::*;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::Base64VecU8;
use near_sdk::{require, AccountId};
use near_sdk::serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub metadata: Option<TokenMetadata>,
    pub approved_account_ids: Option<HashMap<AccountId, u64>>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum VersionedToken {
    Current(Token),
}

pub fn versioned_token_to_token(versioned_token: VersionedToken) -> Token {
    match versioned_token {
        VersionedToken::Current(current) => current,
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

/// Metadata for a type/series.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenTypeMetadata {
    /// NFT title, which will be used as base for each individual NFT's title on enumeration methods
    pub title: Option<String>,
    /// NFT description, which will be returned as each individual NFT's description on enumeration methods
    pub description: Option<String>,
    /// As of `v1-v2-migrate`, this is the CID of the ipfs DIRECTORY that contains NFT assets (these filenames, the contents of this directory, are stored as `media` & `extra` on individual NFTs (TokenMetadata)). In v1, this was the CID of the media itself (not the directory), as the directory upload pattern was not used.
    pub media: Option<String>,
    /// total number of copies for this NFT (minted + to-be-minted)
    pub copies: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum VersionedTokenTypeMetadata {
    Current(TokenTypeMetadata),
}

/// OLD Metadata on the individual token level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenMetadataV1 {
    pub title: Option<String>, // ex. "Arch Nemesis: Mail Carrier" or "Parcel #5055"
    pub description: Option<String>, // free-form description
    pub media: Option<String>, // URL to associated media, preferably to decentralized, content-addressed storage
    pub copies: Option<u64>, // number of copies of this set of metadata in existence when token was minted.
}

/// CURRENT Metadata on the individual token level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenMetadata {
    /// This is always `None` when stored in contract; on enumeration, NFT type title + token number + copies ("My NFT - 1/10") is used to generate title and attach to metadata
    pub title: Option<String>,
    /// This is always `None` when stored in contract; on enumeration, NFT type description is attached to metadata
    pub description: Option<String>, // free-form description
    /// When stored in `token_metadata_by_id`, this is filename of media asset on IPFS. When returned as metadata on token enumeration methods, this is {cid}/{filename}, which can be appended to the contract's base url to create a full `media` url
    pub media: Option<String>,
    /// This is always `None` when stored in contract; on enumeration, NFT type `copies` is attached to metadata
    pub copies: Option<u64>,
    // NEW FIELDS
    /// When stored in `token_metadata_by_id`, this is filename of extra asset (e.g. json) on IPFS. When returned as metadata on token enumeration methods, it is {cid}/{filename}, which can be appended to the contract's base url to create a full `extra` url
    pub extra: Option<String>,
    // TODO: add `updatedAt`? other fields?
}

#[derive(Debug, Clone, PartialEq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum VersionedTokenMetadata {
    Current(TokenMetadata),
}

impl From<VersionedTokenMetadata> for TokenMetadata {
    fn from(metadata: VersionedTokenMetadata) -> Self {
        match metadata {
            VersionedTokenMetadata::Current(current) => current,
        }
    }
}

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