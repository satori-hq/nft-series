const fs = require("fs");
const assert = require("assert");
const testUtils = require("./test-utils");
const nearAPI = require("near-api-js");
const BN = require("bn.js");
const {
  utils: {
    format: { parseNearAmount, formatNearAmount },
  },
  transactions: { deployContract, functionCall },
} = nearAPI;

const {
  gas,
  contractId,
  contractAccount,
  getAccount,
  createOrInitAccount,
  getAccountBalance,
} = testUtils;

const COPIES_TO_MINT = 2;
const APPROVALS_TO_ATTEMPT = 2;
const TOKEN_DELIMETER = ":";
const CONTRACT_TOKEN_DELIMETER = "||";
const BOB_ROYALTY = 1000;

describe("NFT Series", function () {
  this.timeout(60000);

  const now = Date.now().toString();
  let token_type_title = "dog-" + now;
  let token_id;
  console.log("contractId: ", contractId);
  const typeCopies = COPIES_TO_MINT * 2;

  /// users
  const aliceId = "alice-" + now + "." + contractId;
  const bobId = "bob-" + now + "." + contractId;
  const marketId = "market." + contractId;

  let alice, bob, market;
  it("should create user & contract accounts", async function () {
    alice = await getAccount(aliceId);
    bob = await getAccount(bobId);
    console.log("\n\n created:", aliceId, "\n\n");

    market = await createOrInitAccount(marketId);
    const marketState = await market.state();
    if (marketState.code_hash === "11111111111111111111111111111111") {
      const marketBytes = fs.readFileSync("./out/market.wasm");
      console.log(
        "\n\n deploying market contractBytes:",
        marketBytes.length,
        "\n\n"
      );
      const newMarketArgs = {
        owner_id: contractId,
      };
      const actions = [
        deployContract(marketBytes),
        functionCall("new", newMarketArgs, gas),
      ];
      await market.signAndSendTransaction(marketId, actions);
      console.log("\n\n created:", marketId, "\n\n");
    }
  });

  it("should be deployed", async function () {
    const state = await contractAccount.state();
    try {
      await contractAccount.functionCall({
        contractId,
        methodName: "new_default_meta",
        args: {
          owner_id: contractId,
        },
        gas,
      });
    } catch (e) {
      if (!/contract has already been initialized/.test(e.toString())) {
        console.warn(e);
      }
    }

    assert.notStrictEqual(state.code_hash, "11111111111111111111111111111111");
  });

  it("should allow the owner to update the contract's base_uri", async function () {
    const updatedBaseUri = "https://ipfs.io";

    await contractAccount.functionCall({
      contractId,
      methodName: "patch_base_uri",
      args: {
        base_uri: updatedBaseUri,
      },
      gas,
      attachedDeposit: parseNearAmount("0.1"),
    });

    const metadata_updated = await contractAccount.viewFunction(
      contractId,
      "nft_metadata"
    );

    assert.strictEqual(metadata_updated.base_uri, updatedBaseUri);
  });

  it("should allow someone to create a type", async function () {
    await contractAccount.functionCall({
      contractId,
      methodName: "nft_create_type",
      args: {
        metadata: {
          title: token_type_title,
          media: "https://placedog.net/500",
          copies: typeCopies,
        },
        asset_filetypes: ["jpg", "png"],
        asset_distribution: [
          [1, 10],
          [2, 20],
        ],
        royalty: {
          [bobId]: BOB_ROYALTY,
        },
      },
      gas,
      attachedDeposit: parseNearAmount("0.1"),
    });

    const token_type = await contractAccount.viewFunction(
      contractId,
      "nft_get_type",
      {
        token_type_title,
      }
    );

    assert.strictEqual(token_type.owner_id, contractId);
    assert.strictEqual(token_type.metadata.copies, COPIES_TO_MINT * 2);
    console.log(token_type.metadata.copies);
    assert.strictEqual(token_type.royalty[bobId], 1000);
  });

  it("should allow the owner to update any type metadata fields EXCEPT for `copies`", async function () {
    const updatedTitle = token_type_title + " - updated";
    const updatedDescription = "Updated description";
    const updatedMedia = "https://placedog.net/501";
    const updatedCopies = COPIES_TO_MINT * 100;

    let token_type_original = await contractAccount.viewFunction(
      contractId,
      "nft_get_type",
      {
        token_type_title,
      }
    );

    await contractAccount.functionCall({
      contractId,
      methodName: "nft_patch_type",
      args: {
        token_type_title,
        metadata: {
          ...token_type_original.metadata,
          title: updatedTitle,
          description: updatedDescription,
          media: updatedMedia,
          copies: updatedCopies,
        },
      },
      gas,
      attachedDeposit: parseNearAmount("0.1"),
    });

    const token_type_updated = await contractAccount.viewFunction(
      contractId,
      "nft_get_type",
      {
        token_type_title,
      }
    );

    assert.strictEqual(token_type_updated.metadata.title, updatedTitle);
    assert.strictEqual(
      token_type_updated.metadata.description,
      updatedDescription
    );
    assert.strictEqual(token_type_updated.metadata.media, updatedMedia);
    assert.strictEqual(token_type_updated.metadata.copies, typeCopies);

    // revert to original values for future tests
    await contractAccount.functionCall({
      contractId,
      methodName: "nft_patch_type",
      args: {
        token_type_title,
        metadata: {
          ...token_type_original.metadata,
        },
      },
      gas,
      attachedDeposit: parseNearAmount("0.1"),
    });

    const token_type_reverted = await contractAccount.viewFunction(
      contractId,
      "nft_get_type",
      {
        token_type_title,
      }
    );

    assert.strictEqual(
      token_type_reverted.metadata.title,
      token_type_original.metadata.title
    );
    assert.strictEqual(
      token_type_reverted.metadata.description,
      token_type_original.metadata.description
    );
    assert.strictEqual(
      token_type_reverted.metadata.media,
      token_type_original.metadata.media
    );
  });

  it("should allow the owner to update royalties for a type", async function () {
    const token_type_original = await contractAccount.viewFunction(
      contractId,
      "nft_get_type",
      {
        token_type_title,
      }
    );

    const updatedRoyalties = {
      [bobId]: 2000,
    };

    await contractAccount.functionCall({
      contractId,
      methodName: "nft_patch_type",
      args: {
        token_type_title,
        royalty: updatedRoyalties,
      },
      gas,
      attachedDeposit: parseNearAmount("0.1"),
    });

    const token_type_updated = await contractAccount.viewFunction(
      contractId,
      "nft_get_type",
      {
        token_type_title,
      }
    );

    assert.deepEqual(token_type_updated.royalty, updatedRoyalties);

    // revert to original value for future tests
    await contractAccount.functionCall({
      contractId,
      methodName: "nft_patch_type",
      args: {
        token_type_title,
        royalty: token_type_original.royalty,
      },
      gas,
      attachedDeposit: parseNearAmount("0.1"),
    });

    const token_type_reverted = await contractAccount.viewFunction(
      contractId,
      "nft_get_type",
      {
        token_type_title,
      }
    );

    assert.deepEqual(token_type_reverted.royalty, token_type_original.royalty);
  });

  it("should NOT allow a NON owner to mint copies", async function () {
    try {
      await alice.functionCall({
        contractId,
        methodName: "nft_mint_type",
        args: {
          token_type_title,
          receiver_id: contractId,
        },
        gas,
        attachedDeposit: parseNearAmount("0.1"),
      });
      assert(false);
    } catch (e) {
      assert(true);
    }
  });

  it("should allow the owner to mint a token of a particular type", async function () {
    // const stateBefore = await (await getAccount(contractId)).state();
    // console.log('stateBefore', stateBefore)
    const contractBalanceBefore = (await getAccountBalance(contractId))
      .available;

    for (let i = 0; i < COPIES_TO_MINT; i++) {
      await contractAccount.functionCall({
        contractId,
        methodName: "nft_mint_type",
        args: {
          token_type_title,
          receiver_id: contractId,
        },
        gas,
        attachedDeposit: parseNearAmount("0.1"),
      });
    }

    const contractBalanceAfter = (await getAccountBalance(contractId))
      .available;
    console.log(
      "\n\n\n Contract Balance Available",
      formatNearAmount(
        new BN(contractBalanceBefore)
          .sub(new BN(contractBalanceAfter))
          .toString(),
        6
      )
    );

    // const stateAfter = await (await getAccount(contractId)).state();
    // console.log('stateAfter', stateAfter)

    const supply_for_type = await contractAccount.viewFunction(
      contractId,
      "nft_supply_for_type",
      {
        token_type_title,
      }
    );
    assert.strictEqual(parseInt(supply_for_type, 10), COPIES_TO_MINT);

    const tokens = await contractAccount.viewFunction(
      contractId,
      "nft_tokens_by_type",
      {
        token_type_title,
      }
    );
    const [TOKEN_DELIMETER, TITLE_DELIMETER, EDITION_DELIMETER] =
      await contractAccount.viewFunction(contractId, "nft_get_type_format");
    const {
      token_id: _token_id,
      owner_id,
      metadata: { title, copies },
    } = tokens[tokens.length - 1];
    assert.strictEqual(owner_id, contractId);
    token_id = _token_id;
    const formattedTitle = `${token_type_title}${TITLE_DELIMETER}${
      token_id.split(TOKEN_DELIMETER)[1]
    }${EDITION_DELIMETER}${copies}`;
    assert.strictEqual(title, formattedTitle);
  });

  it("should allow the owner cap the copies to whatever is already minted", async function () {
    await contractAccount.functionCall({
      contractId,
      methodName: "cap_copies",
      args: {
        token_type_title,
      },
      gas,
    });

    const token_type = await contractAccount.viewFunction(
      contractId,
      "nft_get_type",
      {
        token_type_title,
      }
    );

    assert.strictEqual(token_type.metadata.copies, COPIES_TO_MINT);
  });

  it("should NOT allow the owner to mint more than copies", async function () {
    try {
      await contractAccount.functionCall({
        contractId,
        methodName: "nft_mint_type",
        args: {
          token_type_title,
          receiver_id: contractId,
        },
        gas,
        attachedDeposit: parseNearAmount("0.1"),
      });
      assert(false);
    } catch (e) {
      assert(true);
    }
  });

  it("should allow the owner to transfer the nft", async function () {
    console.log("\n\n token_id", token_id);

    await contractAccount.functionCall({
      contractId: contractId,
      methodName: "nft_transfer",
      args: {
        receiver_id: aliceId,
        token_id,
      },
      gas,
      attachedDeposit: "1",
    });

    const { owner_id } = await contractAccount.viewFunction(
      contractId,
      "nft_token",
      { token_id }
    );
    assert.strictEqual(owner_id, aliceId);
  });

  it("should allow alice to list the token for sale", async function () {
    let sale_args = {
      sale_conditions: {
        near: parseNearAmount("1"),
      },
      token_type: token_id.split(TOKEN_DELIMETER)[0],
      is_auction: false,
    };

    for (let i = 0; i < APPROVALS_TO_ATTEMPT; i++) {
      try {
        await alice.functionCall({
          contractId: contractId,
          methodName: "nft_approve",
          args: {
            token_id,
            account_id: marketId,
            msg: JSON.stringify(sale_args),
          },
          gas,
          attachedDeposit: parseNearAmount("0.01"),
        });
      } catch (e) {
        // swallow and keep iterating
        console.warn(e);
      }
    }
  });

  it("should allow someone to buy the token and should have paid bob a royalty", async function () {
    const bobBalanceBefore = (await getAccountBalance(bobId)).total;

    await contractAccount.functionCall({
      contractId: marketId,
      methodName: "offer",
      args: {
        nft_contract_id: contractId,
        token_id: token_id,
      },
      gas,
      attachedDeposit: parseNearAmount("1"),
    });

    const bobBalanceAfter = (await getAccountBalance(bobId)).total;

    assert.strictEqual(
      new BN(bobBalanceAfter).sub(new BN(bobBalanceBefore)).toString(),
      parseNearAmount("0.1")
    );
    const { owner_id } = await contractAccount.viewFunction(
      contractId,
      "nft_token",
      { token_id }
    );
    assert.strictEqual(owner_id, contractId);
  });

  it("should return payout object on call of nft_payout", async function () {
    const balanceInt = 1;
    const balance = parseNearAmount(balanceInt.toString());

    const res = await contractAccount.viewFunction(contractId, "nft_payout", {
      token_id,
      balance,
      max_len_payout: 9,
    });
    const bobExpected = (BOB_ROYALTY * balanceInt) / 10000;
    const contractAcctExpected = balanceInt - bobExpected;
    const expected = {
      [bobId]: bobExpected.toString(),
      [contractId]: contractAcctExpected.toString(),
    };
    for (let key in res.payout) {
      res.payout[key] = formatNearAmount(res.payout[key]);
    }
    assert.deepEqual(res.payout, expected);
  });
});
