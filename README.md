# Satori NFT Smart Contract

## Introduction

Repository overview:

- **`/contract`** - an NFT smart contract (the primary component and purpose of this repo)
- **`/market`** - a marketplace smart contract. This is not used in production; rather, it is a basic market contract that implements NEAR's NFT standards, used for the purposes of testing the NFT contract (e.g. listing, sale & payouts functionality).
- **`/neardev`** - a folder created by `near dev-deploy` command during the testing phase, which contains the name of a testnet account (more info [here](https://www.near-sdk.io/upgrading/prototyping#1-rm--rf-neardev--near-dev-deploy) and [here](https://docs.near.org/tools/near-cli#near-dev-deploy))
- **`/out`** - contains compiled WebAssembly ("wasm") for both NFT contract (`/out/main.wasm`) and market contract (`/out/market.wasm`)
  - NB: **`/out/main.wasm`** is the file that Spearmint reads to deploy contract bytecode to an account
- **`/test`** - contains [Mocha](https://mochajs.org/) tests and related configuration and utils
- **`/utils`** - lightweight utility code that reads the current contract name (aka account ID) from `/neardev` and propogates this ID to `/test/config.js`
- **`package.json`** - contains scripts to easily build, deploy (testnet) and test contracts with a single command 🙌
  - **`yarn test:deploy`** is the grandmaster of these commands; it builds both contracts, deploys them, patches the config, and runs tests.

## How to "run" (build, deploy & test) contracts

- **[Install Rust](https://medium.com/r/?url=https%3A%2F%2Fdocs.near.org%2Fdevelop%2Fprerequisites)**

- **[Install NEAR-CLI](https://medium.com/r/?url=https%3A%2F%2Fdocs.near.org%2Ftools%2Fnear-cli%23setup)**

- **Build NFT contract**: `yarn build:contract`

- **Build Marketplace contract**: `yarn build:market` (no need to do this unless you have made changes to the marketplace contract)

- **Build all contracts (NFT & Marketplace)**: `yarn build:all`

- **Deploy NFT contract (testnet)**: `yarn dev:deploy`

  - this command deletes `/neardev` and deploys NFT contract to a new testnet account. The new account ID is saved in `/neardev` and `test/config.js`.
  - NB: `yarn dev:deploy` deploys only the NFT contract, not the marketplace contract. When `yarn test` is run, a marketplace contract is deployed to a derivation of the current NFT contract address if no marketplace contract has been deployed since the most recent NFT contract deployment.

- **Run tests on NFT contract**: `yarn test`

  - this runs mocha tests against the deployed NFT contract and the deployed Marketplace contract

- **Build contracts, deploy and test in a single command**: `yarn test:deploy`

## Development workflow

### Branch

TBC

### Implement

TBC

### Test

- When making a change to the NFT contract, whether it is a bugfix or a new feature, **tests must be added to `/test/api.test.js` to fully test the new functionality and any possible side-effects.**

### Create PR & Request Review

TBC

### Merge PR (kicks off deployment pipeline)

TBC

## Things to keep in mind when developing

- Think about and solve for security in EVERY function. E.g. "Should anyone be able to call this function and do this action, or should it be restricted to the owner of the contract?"
- Be aware of computation and storage costs. Read more about NEAR's [gas](https://docs.near.org/concepts/basics/transactions/gas) and [storage staking](https://docs.near.org/concepts/storage/storage-staking) models if you aren't familiar with them.
- Rust is a strongly typed language! This means that if any of your data shapes change between deployments and you are NOT using enums to version your data structures, you will need to add a migration step in your upgrade process. Generally speaking, it's MUCH easier to upgrade contracts where no data migration is involved or it can be accomplished using enums, so bear this in mind when planning and implementing features and bugfixes.
- Sometimes migrations are unavoidable. In this case, check out this Medium article [COMING SOON] which goes into detail on various migration patterns and how you can use `Versioned` data structures to facilitate data shape changes without having to run a migration step.

## Additional Resources

### Upgrades & Migrations

https://docs.near.org/docs/tutorials/contracts/nfts/upgrade-contract (basic logic update and redeployment; no state migration)

https://www.near-sdk.io/upgrading/production-basics (helpful info about migrate, ignore_state; intro to enums but code breaks; a note about the need for contributions)

https://github.com/evgenykuzyakov/berryclub/commit/d78491b88cbb16a79c15dfc3901e5cfb7df39fe8 (super helpful)

https://nomicon.io/ChainSpec/Upgradability.html (a helpful note on Versioned data structures, but not easily findable for someone looking for contract best practices)

https://github.com/mikedotexe/rust-contract-upgrades/pulls (also very helpful)

### Storage

https://docs.near.org/concepts/storage/storage-staking

### Gas

https://docs.near.org/concepts/basics/transactions/gas

## Rough notes on token series (type) and editions (TODO: UPDATE THIS)

```
// Mappings pseudo code
TokenTypeString -> TokenTypeInt
TokenTypeInt -> TokenTypeStruct (data)

TokenId.split(":")[0] -> TokenTypeInt
TokenId.split(":")[1] -> TokenEditionInt (unique token in type)


In Rust:
// getting owner of token
let owner_id = self.tokens.owner_by_id.get(&token_id)

// getting metadata for token (TokenTypeStruct)
let mut token_id_iter = token_id.split(TOKEN_DELIMETER);
let token_type_id = token_id_iter.next().unwrap().parse().unwrap();
let metadata = self.token_type_by_id.get(&token_type_id).unwrap().metadata;
```

## Instructions

`yarn && yarn test:deploy`

#### Pre-reqs

Rust, cargo, near-cli, etc...
Everything should work if you have NEAR development env for Rust contracts set up.

[Tests](test/api.test.js)
[Contract](contract/src/lib.rs)
