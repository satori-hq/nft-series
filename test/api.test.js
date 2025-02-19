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

let COPIES_TO_MINT = 2;
const APPROVALS_TO_ATTEMPT = 2;
const TOKEN_DELIMETER = ":";
const CONTRACT_TOKEN_DELIMETER = "||";
const BOB_ROYALTY = 1000;

describe("NFT Series", function () {
  this.timeout(120000);

  const now = Date.now().toString();
  let token_type_title_non_gen = "dog-non-gen" + now;
  let token_type_title_semi_gen = "dog-semi-gen" + now;
  let token_type_title_fully_gen = "dog-fully-gen" + now;
  let token_type_title_fully_gen_single_filetype =
    "dog-fully-gen-single-filetype" + now;

  let token_id;

  let typeCopies = COPIES_TO_MINT * 2;

  let assets = [["some-asset-title.jpg", "1000", ""]];

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

  it("should allow the owner to update all fields of a contract's source_metadata", async function () {
    const updatedVersion = Date.now().toString();
    const updatedHash = "1".repeat(63);
    const updatedLink = "updatedLink";

    await contractAccount.functionCall({
      contractId,
      methodName: "patch_contract_source_metadata",
      args: {
        new_source_metadata: {
          version: updatedVersion,
          commit_sha: updatedHash,
          link: updatedLink,
        },
      },
      gas,
      attachedDeposit: parseNearAmount("0.1"),
    });

    const source_metadata_updated = await contractAccount.viewFunction(
      contractId,
      "contract_source_metadata"
    );

    assert.strictEqual(source_metadata_updated.version, updatedVersion);
    assert.strictEqual(source_metadata_updated.commit_sha, updatedHash);
    assert.strictEqual(source_metadata_updated.link, updatedLink);
  });

  it("should allow the owner to update a single field of a contract's source_metadata", async function () {
    const source_metadata_original = await contractAccount.viewFunction(
      contractId,
      "contract_source_metadata"
    );

    const updatedVersion = Date.now().toString();

    await contractAccount.functionCall({
      contractId,
      methodName: "patch_contract_source_metadata",
      args: {
        new_source_metadata: {
          version: updatedVersion,
        },
      },
      gas,
      attachedDeposit: parseNearAmount("0.1"),
    });

    const source_metadata_updated = await contractAccount.viewFunction(
      contractId,
      "contract_source_metadata"
    );

    assert.strictEqual(source_metadata_updated.version, updatedVersion);
    assert.strictEqual(
      source_metadata_updated.commit_sha,
      source_metadata_original.commit_sha
    );
    assert.strictEqual(
      source_metadata_updated.link,
      source_metadata_original.link
    );
  });

  // nft_update_metadata
  it("should allow the owner to update 'name' and 'base_uri' fields on contract's metadata", async function () {
    const current_metadata = await contractAccount.viewFunction(
        contractId,
        "nft_metadata"
    );
    const new_metadata = {
        ...current_metadata,
        name: "Sonar by Satori New Name",
        base_uri: "https://some-other-domain.io",
    };
    await contractAccount.functionCall({
      contractId,
      methodName: "nft_update_contract_metadata",
      args: {
        new_metadata: new_metadata,
      },
      gas,
      attachedDeposit: parseNearAmount("0.1"),
    });
    const updated_metadata = await contractAccount.viewFunction(
        contractId,
        "nft_metadata"
    );

    assert.strictEqual(updated_metadata.name, new_metadata.name);
    assert.strictEqual(updated_metadata.base_uri, new_metadata.base_uri);
  });

  it("should error if owner attempts to create a type with invalid arguments", async function () {
    typeCopies = 10;

    // no `metadata.title`
    invalidArgs = {
      metadata: {
        media: "bafkreibael4nenayqy45ijuvgcpkmyscbt3q35mtbzbeabopmugdwr5r64",
        copies: typeCopies,
      },
      assets: [["1.json", typeCopies.toString(), ""]],
      royalty: {
        [bobId]: BOB_ROYALTY,
      },
    };

    try {
      await testUtils.createType(
        contractAccount,
        contractId,
        invalidArgs,
        parseNearAmount("3")
      );
      assert(false);
    } catch {
      assert(true);
    }

    // invalid `metadata.title`
    invalidArgs = {
      ...invalidArgs,
      metadata: {
        title: 1,
        media: "bafkreibael4nenayqy45ijuvgcpkmyscbt3q35mtbzbeabopmugdwr5r64",
        copies: typeCopies,
      },
    };

    try {
      await testUtils.createType(
        contractAccount,
        contractId,
        invalidArgs,
        parseNearAmount("3")
      );
      assert(false);
    } catch {
      assert(true);
    }

    // no `metadata.media`
    invalidArgs = {
      ...invalidArgs,
      metadata: {
        title: token_type_title_non_gen,
        copies: typeCopies,
      },
    };

    try {
      await testUtils.createType(
        contractAccount,
        contractId,
        invalidArgs,
        parseNearAmount("3")
      );
      assert(false);
    } catch {
      assert(true);
    }

    // invalid `metadata.media`
    invalidArgs = {
      ...invalidArgs,
      metadata: {
        title: token_type_title_non_gen,
        media: 1,
        copies: typeCopies,
      },
    };

    try {
      await testUtils.createType(
        contractAccount,
        contractId,
        invalidArgs,
        parseNearAmount("3")
      );
      assert(false);
    } catch {
      assert(true);
    }

    // no `metadata.copies`
    invalidArgs = {
      ...invalidArgs,
      metadata: {
        title: token_type_title_non_gen,
        media: "bafkreibael4nenayqy45ijuvgcpkmyscbt3q35mtbzbeabopmugdwr5r64",
      },
    };

    try {
      await testUtils.createType(
        contractAccount,
        contractId,
        invalidArgs,
        parseNearAmount("3")
      );
      assert(false);
    } catch {
      assert(true);
    }

    // invalid `metadata.copies`
    invalidArgs = {
      ...invalidArgs,
      metadata: {
        title: token_type_title_non_gen,
        media: "bafkreibael4nenayqy45ijuvgcpkmyscbt3q35mtbzbeabopmugdwr5r64",
        copies: typeCopies.toString(),
      },
    };

    try {
      await testUtils.createType(
        contractAccount,
        contractId,
        invalidArgs,
        parseNearAmount("3")
      );
      assert(false);
    } catch {
      assert(true);
    }

    // second elements (`supply_remaining`) of all asset_distribution elements (sub-arrays) must add up to `metadata.copies`
    asset_filetypes = ["jpg", "png"];

    invalidArgs = {
      ...invalidArgs,
      metadata: {
        ...invalidArgs.metadata,
        copies: typeCopies,
      },
      assets: [
        ["cat", (typeCopies / 2).toString(), ""],
        ["dog", (typeCopies / 2 + 1).toString(), ""],
      ],
    };

    try {
      await testUtils.createType(
        contractAccount,
        contractId,
        invalidArgs,
        parseNearAmount("3")
      );
      assert(false);
    } catch {
      assert(true);
    }

    // invalid asset_id (`null`)
    invalidArgs = {
      ...invalidArgs,
      assets: [
        [null, (typeCopies / 2).toString(), ""],
        ["2", (typeCopies / 2).toString(), ""],
      ],
    };

    try {
      await testUtils.createType(
        contractAccount,
        contractId,
        invalidArgs,
        parseNearAmount("3")
      );
      assert(false);
    } catch {
      assert(true);
    }
  });

  it("should allow owner to create a non-generative type", async function () {
    typeCopies = 1000;
    asset_filetypes = ["jpg"];
    await contractAccount.functionCall({
      contractId,
      methodName: "nft_create_type",
      args: {
        metadata: {
          title: token_type_title_non_gen,
          media: "bafkreibael4nenayqy45ijuvgcpkmyscbt3q35mtbzbeabopmugdwr5r64",
          copies: typeCopies,
        },
        assets,
        royalty: {
          [bobId]: BOB_ROYALTY,
        },
        cover_asset: assets[0][0],
      },
      gas,
      attachedDeposit: parseNearAmount("0.1"),
    });

    const token_type = await contractAccount.viewFunction(
      contractId,
      "nft_get_type",
      {
        token_type_title: token_type_title_non_gen,
      }
    );
    // console.log("non-generative token type: ", token_type);

    assert.strictEqual(token_type.owner_id, contractId);
    assert.strictEqual(token_type.metadata.copies, typeCopies);
    assert.strictEqual(token_type.royalty[bobId], 1000);
  });

  it("should allow the owner to mint correctly formatted tokens of a non-generative type", async function () {
    COPIES_TO_MINT = 5;
    for (let i = 0; i < COPIES_TO_MINT; i++) {
      await contractAccount.functionCall({
        contractId,
        methodName: "nft_mint_type",
        args: {
          token_type_title: token_type_title_non_gen,
          receiver_id: contractId,
        },
        gas,
        attachedDeposit: parseNearAmount("0.1"),
      });
    }

    const supply_for_type = await contractAccount.viewFunction(
      contractId,
      "nft_supply_for_type",
      {
        token_type_title: token_type_title_non_gen,
      }
    );

    assert.strictEqual(parseInt(supply_for_type, 10), COPIES_TO_MINT);

    const tokens = await contractAccount.viewFunction(
      contractId,
      "nft_tokens_by_type",
      {
        token_type_title: token_type_title_non_gen,
      }
    );

    // console.log("non-gen tokens: ", tokens);

    const [TOKEN_DELIMETER, TITLE_DELIMETER, EDITION_DELIMETER] =
      await contractAccount.viewFunction(contractId, "nft_get_type_format");

    const {
      token_id: _token_id,
      owner_id,
      metadata: { title, copies },
    } = tokens[tokens.length - 1];

    // check for correct owner
    assert.strictEqual(owner_id, contractId);
    token_id = _token_id;
    const formattedTitle = `${token_type_title_non_gen}${TITLE_DELIMETER}${
      token_id.split(TOKEN_DELIMETER)[1]
    }${EDITION_DELIMETER}${copies}`;

    // check for correctly formatted title
    assert.strictEqual(title, formattedTitle);

    // check that all tokens have correct filetypes for `media` & `extra`
    tokens.forEach((token) => {
      if (!token.metadata.media.endsWith(assets[0][0])) assert(false);
    });
  });

  it("should allow owner to create a semi-generative type", async function () {
    typeCopies = 6;
    assets = [
      ["silver.jpg", "4", "silver.json"],
      ["gold.png", "2", "gold.json"],
    ];
    await contractAccount.functionCall({
      contractId,
      methodName: "nft_create_type",
      args: {
        metadata: {
          title: token_type_title_semi_gen,
          media: "bafkreibael4nenayqy45ijuvgcpkmyscbt3q35mtbzbeabopmugdwr5r64",
          copies: typeCopies,
        },
        assets,
        royalty: {
          [bobId]: BOB_ROYALTY,
        },
        cover_asset: assets[0][0],
      },
      gas,
      attachedDeposit: parseNearAmount("3"), // need 2.5+ N to store these large arrays on the type
    });

    const token_type = await contractAccount.viewFunction(
      contractId,
      "nft_get_type",
      {
        token_type_title: token_type_title_semi_gen,
      }
    );
    // console.log("token type semi-gen: ", token_type);

    assert.strictEqual(token_type.owner_id, contractId);
    assert.strictEqual(token_type.metadata.copies, typeCopies);
    assert.strictEqual(token_type.royalty[bobId], 1000);
  });

  it("should allow the owner to mint correctly formatted tokens of a semi-generative type", async function () {
    COPIES_TO_MINT = typeCopies;

    for (let i = 0; i < COPIES_TO_MINT; i++) {
      await contractAccount.functionCall({
        contractId,
        methodName: "nft_mint_type",
        args: {
          token_type_title: token_type_title_semi_gen,
          receiver_id: contractId,
        },
        gas,
        attachedDeposit: parseNearAmount("0.1"),
      });
    }

    const supply_for_type = await contractAccount.viewFunction(
      contractId,
      "nft_supply_for_type",
      {
        token_type_title: token_type_title_semi_gen,
      }
    );

    assert.strictEqual(parseInt(supply_for_type, 10), COPIES_TO_MINT);

    const tokens = await contractAccount.viewFunction(
      contractId,
      "nft_tokens_by_type",
      {
        token_type_title: token_type_title_semi_gen,
      }
    );

    console.log("semi-gen tokens: ", tokens);

    // check for expected quantity of each filetype
    let distrCount1 = 0;
    let distrCount2 = 0;
    tokens.forEach((token) => {
      if (token.metadata.media.endsWith(assets[0][0])) distrCount1++;
      else if (token.metadata.media.endsWith(assets[1][0])) distrCount2++;
    });

    if (distrCount1 !== parseInt(assets[0][1], 10)) assert(false);
    if (distrCount2 !== parseInt(assets[1][1], 10)) assert(false);
  });

  it("should allow owner to create a fully-generative type", async function () {
    // typeCopies = 5;
    // assets = [
    //   ["koala.png", "1", "koala.json"],
    //   ["platypus.jpg", "1", "platypus.json"],
    //   ["echidna.mp4", "1", "echidna.json"],
    //   ["kangaroo.webm", "1", "kangaroo.json"],
    //   ["wombat.jpg", "1", "wombat.json"],
    // ];
    typeCopies = 10_000;
    assets = [];
    for (let i = 1; i <= 10_000; i++) {
      assets.push([`#${i}.png`, "1", `#${i}.json`]);
    }
    try {
      await contractAccount.functionCall({
        contractId,
        methodName: "nft_create_type",
        args: {
          metadata: {
            title: token_type_title_fully_gen,
            media:
              "bafkreibael4nenayqy45ijuvgcpkmyscbt3q35mtbzbeabopmugdwr5r64",
            copies: typeCopies,
          },
          assets,
          royalty: {
            [bobId]: BOB_ROYALTY,
          },
          cover_asset: assets[0][0],
        },
        gas,
        attachedDeposit: parseNearAmount("5"),
      });

      const token_type = await contractAccount.viewFunction(
        contractId,
        "nft_get_type",
        {
          token_type_title: token_type_title_fully_gen,
        }
      );

      // console.log("fully gen token type: ", token_type);

      assert.strictEqual(token_type.owner_id, contractId);
      assert.strictEqual(token_type.metadata.copies, typeCopies);
      assert.strictEqual(token_type.royalty[bobId], 1000);
    } catch (e) {
      console.log("error creating type: ", e);
    }
  });

  it("should allow the owner to mint correctly formatted tokens of a fully-generative type", async function () {
    // COPIES_TO_MINT = typeCopies;
    COPIES_TO_MINT = 5;
    for (let i = 0; i < COPIES_TO_MINT; i++) {
      await contractAccount.functionCall({
        contractId,
        methodName: "nft_mint_type",
        args: {
          token_type_title: token_type_title_fully_gen,
          receiver_id: contractId,
        },
        gas,
        attachedDeposit: parseNearAmount("0.1"),
      });
    }

    const supply_for_type = await contractAccount.viewFunction(
      contractId,
      "nft_supply_for_type",
      {
        token_type_title: token_type_title_fully_gen,
      }
    );

    assert.strictEqual(parseInt(supply_for_type, 10), COPIES_TO_MINT);

    const tokens = await contractAccount.viewFunction(
      contractId,
      "nft_tokens_by_type",
      {
        token_type_title: token_type_title_fully_gen,
      }
    );

    console.log("fully-gen tokens: ", tokens);

    // check that each token has expected media asset
    let foundCount = 0;
    for (let i = 0; i < tokens.length; i++) {
      const token = tokens[i];
      for (let j = 0; j < assets.length; j++) {
        const asset = assets[j];
        if (token.metadata.media.endsWith(asset[0])) {
          foundCount++;
          break;
        }
      }
    }
    if (foundCount !== tokens.length) assert(false);
    else assert(true);
  });

  it("should allow the owner to update any type metadata fields EXCEPT for `media` and `copies`", async function () {
    const updatedTitle = token_type_title_semi_gen + " - updated";
    const updatedDescription = "Updated description";
    const updatedMedia =
      "bafybeiasgveflayov5ux6rwbkymt6mcmnq4rpzxjnbies5za3urezaykny";
    const updatedCopies = COPIES_TO_MINT * 100;

    let token_type_original = await contractAccount.viewFunction(
      contractId,
      "nft_get_type",
      {
        token_type_title: token_type_title_semi_gen,
      }
    );
    console.log("token_type_original: ", token_type_original);

    await contractAccount.functionCall({
      contractId,
      methodName: "nft_update_type",
      args: {
        token_type_title: token_type_title_semi_gen,
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
        token_type_title: updatedTitle,
      }
    );
    console.log("token_type_updated: ", token_type_updated);

    assert.strictEqual(token_type_updated.metadata.title, updatedTitle);
    assert.strictEqual(
      token_type_updated.metadata.description,
      updatedDescription
    );
    assert.strictEqual(
      token_type_updated.metadata.media,
      token_type_original.metadata.media
    );
    assert.strictEqual(
      token_type_updated.metadata.copies,
      token_type_original.metadata.copies
    );

    // revert to original values for future tests
    await contractAccount.functionCall({
      contractId,
      methodName: "nft_update_type",
      args: {
        token_type_title: updatedTitle,
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
        token_type_title: token_type_original.metadata.title,
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
        token_type_title: token_type_title_semi_gen,
      }
    );

    const updatedRoyalties = {
      [bobId]: 2000,
    };

    await contractAccount.functionCall({
      contractId,
      methodName: "nft_update_type",
      args: {
        token_type_title: token_type_title_semi_gen,
        royalty: updatedRoyalties,
      },
      gas,
      attachedDeposit: parseNearAmount("0.1"),
    });

    const token_type_updated = await contractAccount.viewFunction(
      contractId,
      "nft_get_type",
      {
        token_type_title: token_type_title_semi_gen,
      }
    );

    assert.deepEqual(token_type_updated.royalty, updatedRoyalties);

    // revert to original value for future tests
    await contractAccount.functionCall({
      contractId,
      methodName: "nft_update_type",
      args: {
        token_type_title: token_type_title_semi_gen,
        royalty: token_type_original.royalty,
      },
      gas,
      attachedDeposit: parseNearAmount("0.1"),
    });

    const token_type_reverted = await contractAccount.viewFunction(
      contractId,
      "nft_get_type",
      {
        token_type_title: token_type_title_semi_gen,
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
          token_type_title_semi_gen,
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

  // nft_batch_mint_type
  it("should NOT allow more than 10 receiver", async function () {
    let receiver_ids = [];
    for (let i = 0; i < 1000; i++) {
        receiver_ids.push(`${i}_${contractId}`);
    }
    try {
      const response = await contractAccount.functionCall({
        contractId,
        methodName: "nft_batch_mint_type",
        args: {
          token_type_title: token_type_title_non_gen,
          receiver_ids: receiver_ids,
        },
        gas,
        attachedDeposit: parseNearAmount("0.1"),
      });
      console.log(response)
      assert(false);
    } catch (e) {
      assert(true);
    }
  });

  it("should allow the owner cap the copies to whatever is already minted", async function () {
    const supply = await contractAccount.viewFunction(
      contractId,
      "nft_supply_for_type",
      {
        token_type_title: token_type_title_semi_gen,
      }
    );

    await contractAccount.functionCall({
      contractId,
      methodName: "nft_cap_copies",
      args: {
        token_type_title: token_type_title_semi_gen,
      },
      gas,
    });

    const token_type = await contractAccount.viewFunction(
      contractId,
      "nft_get_type",
      {
        token_type_title: token_type_title_semi_gen,
      }
    );

    assert.strictEqual(token_type.metadata.copies, parseInt(supply, 10));
  });

  it("should NOT allow the owner to mint more than copies", async function () {
    try {
      await contractAccount.functionCall({
        contractId,
        methodName: "nft_mint_type",
        args: {
          token_type_title: token_type_title_semi_gen,
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

  it("should NOT allow the owner to delete a series that contains tokens", async function () {
    try {
      await contractAccount.functionCall({
        contractId,
        methodName: "nft_delete_type",
        args: {
          token_type_title: token_type_title_non_gen,
        },
        gas,
        attachedDeposit: parseNearAmount("0.1"),
      });
      assert(false);
    } catch (e) {
      assert(true);
    }
  });

  it("should allow the owner to delete a series that contains no tokens", async function () {
    try {
      typeCopies = 10;
      let title = "series-to-be-deleted" + Date.now();
      let assets = [["some-asset-title.jpg", "10", ""]];

      let args = {
        metadata: {
          title,
          media: "bafkreibael4nenayqy45ijuvgcpkmyscbt3q35mtbzbeabopmugdwr5r64",
          copies: typeCopies,
        },
        assets: assets,
        royalty: {
          [bobId]: BOB_ROYALTY,
        },
        cover_asset: assets[0][0],
      };

      await testUtils.createType(
        contractAccount,
        contractId,
        args,
        parseNearAmount("1")
      );

      await contractAccount.functionCall({
        contractId,
        methodName: "nft_delete_type",
        args: {
          token_type_title: title,
        },
        gas,
        attachedDeposit: parseNearAmount("0.1"),
      });

      try {
        await contractAccount.viewFunction(contractId, "nft_get_type", {
          token_type_title: token_type_title_fully_gen_single_filetype,
        });
        assert(false);
      } catch (e) {
        assert(true);
      }
    } catch (e) {
      assert(false);
    }
  });

  it("should allow the owner to transfer the nft", async function () {
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
        const nftApproveRes = await alice.functionCall({
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

    const res = await contractAccount.functionCall({
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
