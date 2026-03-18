import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
  TOKEN_2022_PROGRAM_ID,
  getAccount,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";
import { Keypair, PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";

import { expect } from "chai";
import { Svs9 } from "../target/types/svs_9";

const ALLOCATOR_VAULT_SEED = Buffer.from("allocator_vault");
const CHILD_ALLOCATION_SEED = Buffer.from("child_allocation");
const SHARES_SEED = Buffer.from("shares");
const IDLE_VAULT_SEED = Buffer.from("idle_vault");

describe("svs-9 (Allocator Vault)", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.Svs9 as Program<Svs9>;
  const user = provider.wallet as anchor.Wallet;

  const VAULT_ID = new BN(1);
  const IDLE_BUFFER_BPS = 500; // 5%
  const DECIMALS_OFFSET = 3;

  let vaultPda: PublicKey;
  let sharesMint: PublicKey;
  let idleVault: PublicKey;
  let assetMint: PublicKey;
  let userAssetAta: PublicKey;
  let userSharesAta: PublicKey;

  before(async () => {
    [vaultPda] = PublicKey.findProgramAddressSync(
      [ALLOCATOR_VAULT_SEED, new Uint8Array(32), VAULT_ID.toArrayLike(Buffer, "le", 8)],
      program.programId
    );
  });

  it("creates asset mint", async () => {
    assetMint = await createMint(
      provider.connection, user.payer, user.publicKey, null, 6,
      undefined, undefined, TOKEN_PROGRAM_ID
    );

    [vaultPda] = PublicKey.findProgramAddressSync(
      [ALLOCATOR_VAULT_SEED, assetMint.toBuffer(), VAULT_ID.toArrayLike(Buffer, "le", 8)],
      program.programId
    );
    [sharesMint] = PublicKey.findProgramAddressSync(
      [SHARES_SEED, vaultPda.toBuffer()],
      program.programId
    );
    idleVault = getAssociatedTokenAddressSync(assetMint, vaultPda, true, TOKEN_PROGRAM_ID);

    expect(assetMint).to.not.be.null;
    console.log("assetMint:", assetMint.toBase58());
    console.log("vaultPda:", vaultPda.toBase58());
  });

  it("initializes allocator vault", async () => {
    await program.methods
      .initialize(VAULT_ID, IDLE_BUFFER_BPS, DECIMALS_OFFSET)
      .accounts({
        authority: user.publicKey,
        assetMint: assetMint,
        vault: vaultPda,
        sharesMint: sharesMint,
        idleVault: idleVault,
        assetTokenProgram: TOKEN_PROGRAM_ID,
        sharesTokenProgram: TOKEN_2022_PROGRAM_ID,
        associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .rpc();

    const vault = await program.account.allocatorVault.fetch(vaultPda);
    expect(vault.vaultId.toString()).to.equal(VAULT_ID.toString());
    expect(vault.paused).to.be.false;
    expect(vault.numChildren).to.equal(0);
    expect(vault.idleBufferBps).to.equal(IDLE_BUFFER_BPS);
    console.log("vault initialized:", vaultPda.toBase58());
  });

  it("mints tokens to user", async () => {
    const ata = await getOrCreateAssociatedTokenAccount(
      provider.connection, user.payer, assetMint, user.publicKey,
      false, undefined, undefined, TOKEN_PROGRAM_ID
    );
    userAssetAta = ata.address;

    await mintTo(
      provider.connection, user.payer, assetMint, userAssetAta,
      user.publicKey, 10_000_000, [], undefined, TOKEN_PROGRAM_ID
    );

    const bal = await getAccount(provider.connection, userAssetAta, undefined, TOKEN_PROGRAM_ID);
    expect(Number(bal.amount)).to.equal(10_000_000);
    console.log("minted 10 tokens to user");
  });

  it("deposits into allocator vault", async () => {
    const sharesAta = await getOrCreateAssociatedTokenAccount(
      provider.connection, user.payer, sharesMint, user.publicKey,
      false, undefined, undefined, TOKEN_2022_PROGRAM_ID
    );
    userSharesAta = sharesAta.address;

    await program.methods
      .deposit(new BN(1_000_000), new BN(0))
      .accounts({
        user: user.publicKey,
        vault: vaultPda,
        assetMint: assetMint,
        idleVault: idleVault,
        sharesMint: sharesMint,
        userAssetAccount: userAssetAta,
        userSharesAccount: userSharesAta,
        tokenProgram: TOKEN_PROGRAM_ID,
        sharesTokenProgram: TOKEN_2022_PROGRAM_ID,
        associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const vault = await program.account.allocatorVault.fetch(vaultPda);
    expect(vault.totalShares.toNumber()).to.be.greaterThan(0);
    console.log("total shares after deposit:", vault.totalShares.toString());
  });

  it("rejects deposit below minimum", async () => {
    try {
      await program.methods
        .deposit(new BN(10), new BN(0))
        .accounts({
          user: user.publicKey,
          vault: vaultPda,
          assetMint: assetMint,
          idleVault: idleVault,
          sharesMint: sharesMint,
          userAssetAccount: userAssetAta,
          userSharesAccount: userSharesAta,
          tokenProgram: TOKEN_PROGRAM_ID,
          sharesTokenProgram: TOKEN_2022_PROGRAM_ID,
          associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
      expect.fail("should have thrown");
    } catch (e) {
      expect(e.message).to.include("Error");
      console.log("correctly rejected small deposit");
    }
  });

  it("pauses vault and rejects deposit", async () => {
    await program.methods.pause()
      .accounts({ vault: vaultPda, authority: user.publicKey })
      .rpc();

    const vault = await program.account.allocatorVault.fetch(vaultPda);
    expect(vault.paused).to.be.true;

    try {
      await program.methods
        .deposit(new BN(1_000_000), new BN(0))
        .accounts({
          user: user.publicKey,
          vault: vaultPda,
          assetMint: assetMint,
          idleVault: idleVault,
          sharesMint: sharesMint,
          userAssetAccount: userAssetAta,
          userSharesAccount: userSharesAta,
          tokenProgram: TOKEN_PROGRAM_ID,
          sharesTokenProgram: TOKEN_2022_PROGRAM_ID,
          associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
      expect.fail("should have thrown");
    } catch (e) {
      expect(e.message).to.include("Error");
      console.log("correctly rejected deposit on paused vault");
    }
  });

  it("unpauses vault", async () => {
    await program.methods.unpause()
      .accounts({ vault: vaultPda, authority: user.publicKey })
      .rpc();

    const vault = await program.account.allocatorVault.fetch(vaultPda);
    expect(vault.paused).to.be.false;
  });

  it("sets curator", async () => {
    const newCurator = Keypair.generate();
    await program.methods.setCurator(newCurator.publicKey)
      .accounts({ vault: vaultPda, authority: user.publicKey })
      .rpc();

    const vault = await program.account.allocatorVault.fetch(vaultPda);
    expect(vault.curator.toBase58()).to.equal(newCurator.publicKey.toBase58());

    // Reset curator back
    await program.methods.setCurator(user.publicKey)
      .accounts({ vault: vaultPda, authority: user.publicKey })
      .rpc();
    console.log("curator set and reset");
  });

  it("redeems from allocator vault", async () => {
    const vaultBefore = await program.account.allocatorVault.fetch(vaultPda);
    const sharesToRedeem = vaultBefore.totalShares.divn(2);

    await program.methods
      .redeem(sharesToRedeem, new BN(0))
      .accounts({
        user: user.publicKey,
        vault: vaultPda,
        assetMint: assetMint,
        idleVault: idleVault,
        sharesMint: sharesMint,
        userAssetAccount: userAssetAta,
        userSharesAccount: userSharesAta,
        tokenProgram: TOKEN_PROGRAM_ID,
        sharesTokenProgram: TOKEN_2022_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const vaultAfter = await program.account.allocatorVault.fetch(vaultPda);
    expect(vaultAfter.totalShares.toNumber()).to.be.lessThan(vaultBefore.totalShares.toNumber());
    console.log("shares after redeem:", vaultAfter.totalShares.toString());
  });

  it("transfers authority", async () => {
    const newAuthority = Keypair.generate();
    await program.methods.transferAuthority(newAuthority.publicKey)
      .accounts({ vault: vaultPda, authority: user.publicKey })
      .rpc();

    const vault = await program.account.allocatorVault.fetch(vaultPda);
    expect(vault.authority.toBase58()).to.equal(newAuthority.publicKey.toBase58());

    await program.methods.transferAuthority(user.publicKey)
      .accounts({ vault: vaultPda, authority: newAuthority.publicKey })
      .signers([newAuthority])
      .rpc();

    console.log("authority transferred and reset");
  });
});
