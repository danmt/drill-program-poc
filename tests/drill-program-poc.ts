import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { DrillProgramPoc } from "../target/types/drill_program_poc";
import { createAssociatedTokenAccount, createFundedWallet, createMint, mintTo } from "./utils";
import { PublicKey, Keypair } from '@solana/web3.js';
import { BN } from "bn.js";
import { assert } from "chai";
import { getAccount } from "@solana/spl-token";

describe("drill-program-poc", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.DrillProgramPoc as Program<DrillProgramPoc>;

  const boardId = 1;
  const bountyNumber = 2;
  let boardPublicKey: PublicKey;
  let bountyPublicKey: PublicKey;
  let bountyVaultPublicKey: PublicKey;
  let acceptedMintPublicKey: PublicKey;
  let user1Keypair: Keypair;
  let user1AssociatedTokenAccount: PublicKey;
  const user1Balance = 500;
  const user2Name = 'user2';
  let user2Keypair: Keypair;
  let user2AssociatedTokenAccount: PublicKey;
  const user2Balance = 25;
  const bountyTotal = new BN(100);

  before(async () => {
    [boardPublicKey] = await PublicKey.findProgramAddress([
      Buffer.from('board', 'utf8'),
      new BN(boardId).toArrayLike(Buffer, "le", 4),
    ], program.programId);
    [bountyPublicKey] = await PublicKey.findProgramAddress([
      Buffer.from('bounty', 'utf8'),
      boardPublicKey.toBuffer(),
      new BN(bountyNumber).toArrayLike(Buffer, "le", 4),
    ], program.programId);
    [bountyVaultPublicKey] = await PublicKey.findProgramAddress([
      Buffer.from('bounty_vault', 'utf8'),
      bountyPublicKey.toBuffer(),
    ], program.programId);

    acceptedMintPublicKey = await createMint(provider);
    user1Keypair = await createFundedWallet(provider);
    user2Keypair = await createFundedWallet(provider);
    
    user1AssociatedTokenAccount = await createAssociatedTokenAccount(provider, acceptedMintPublicKey, user1Keypair);
    user2AssociatedTokenAccount = await createAssociatedTokenAccount(provider, acceptedMintPublicKey, user2Keypair);

    await mintTo(provider, acceptedMintPublicKey, user1Balance, user1AssociatedTokenAccount);
    await mintTo(provider, acceptedMintPublicKey, user2Balance, user2AssociatedTokenAccount);     
  })

  it("should initialize board", async () => {
    // act
    await program.methods
      .initializeBoard(boardId)
      .accounts({
        acceptedMint: acceptedMintPublicKey,
        authority: user1Keypair.publicKey,
      })
      .signers([user1Keypair])
      .rpc();
    // assert
    const boardAccount = await program.account.board.fetchNullable(boardPublicKey);
    assert.notEqual(boardAccount, null);
  });

  it("should initialize bounty", async () => {
    // act
    await program.methods
      .initializeBounty(boardId, bountyNumber)
      .accounts({
        acceptedMint: acceptedMintPublicKey,
        authority: user1Keypair.publicKey,
      })
      .signers([user1Keypair])
      .rpc();
    // assert
    const bountyAccount = await program.account.bounty.fetchNullable(bountyPublicKey);
    assert.notEqual(bountyAccount, null);
    assert.equal(bountyAccount.isClosed, false);
  });

  it("should deposit into bounty", async () => {
    // act
    await program.methods
      .deposit(boardId, bountyNumber, bountyTotal)
      .accounts({
        authority: user1Keypair.publicKey,
        sponsorVault: user1AssociatedTokenAccount,
      })
      .signers([user1Keypair])
      .rpc();
    // assert
    const bountyVaultAccount = await getAccount(provider.connection, bountyVaultPublicKey);
    const sponsorVaultAccount = await getAccount(provider.connection, user1AssociatedTokenAccount);
    assert.equal(bountyVaultAccount.amount, BigInt(`0x${bountyTotal.toString('hex')}`));
    assert.equal(sponsorVaultAccount.amount, BigInt(user1Balance) - BigInt(`0x${bountyTotal.toString('hex')}`));
  });

  it("should close bounty", async () => {
    // act
    await program.methods
      .closeBounty(boardId, bountyNumber, user2Name)
      .accounts({ authority: user1Keypair.publicKey })
      .signers([user1Keypair])
      .rpc();
    // assert
    const bountyAccount = await program.account.bounty.fetchNullable(bountyPublicKey);
    assert.equal(bountyAccount.isClosed, true);
    assert.notEqual(bountyAccount.bountyHunter, null);
    assert.equal(bountyAccount.bountyHunter, user2Name);
  });

  it("should send bounty", async () => {
    // act
    await program.methods
      .sendBounty(boardId, bountyNumber, user2Name)
      .accounts({ 
        authority: user2Keypair.publicKey,
        userVault: user2AssociatedTokenAccount,
        boardAuthority: user1Keypair.publicKey,
      })
      .signers([user2Keypair])
      .rpc();
    // assert
    const bountyAccount = await program.account.bounty.fetchNullable(bountyPublicKey);
    const userVaultAccount = await getAccount(provider.connection, user2AssociatedTokenAccount);
    const bountyVaultAccount = await provider.connection.getAccountInfo(bountyVaultPublicKey);
    assert.equal(bountyAccount, null);
    assert.equal(bountyVaultAccount, null);
    assert.equal(userVaultAccount.amount, BigInt(user2Balance) + BigInt(`0x${bountyTotal.toString('hex')}`));
  });
});
