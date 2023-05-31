import * as anchor from "@coral-xyz/anchor"
import { BN } from "@coral-xyz/anchor"
import { AnchorEscrow2023Timed, IDL } from "../target/types/anchor_escrow_2023_timed"
import { PublicKey, Commitment, Keypair, SystemProgram } from "@solana/web3.js"
import { ASSOCIATED_TOKEN_PROGRAM_ID as associatedTokenProgram, TOKEN_PROGRAM_ID as tokenProgram, createMint, createAccount, mintTo, getAssociatedTokenAddress } from "@solana/spl-token"
import { randomBytes } from "crypto"
import { assert } from "chai"

const commitment: Commitment = "confirmed"

describe("anchor-escrow-2023-timed", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const commitment: Commitment = "confirmed"; // processed, confirmed, finalized

  const programId = new PublicKey("8RifdHyuiX5yDUYJRGBHHpY1K8tyoQuUhR69TstPh2p1");
  const program = new anchor.Program<AnchorEscrow2023Timed>(IDL, programId, anchor.getProvider());

  // Set up our keys
  const [maker, taker] = [new Keypair(), new Keypair()];

  // Random seed
  const seed = new BN(randomBytes(8));
  
  // PDAs
  const auth = PublicKey.findProgramAddressSync([Buffer.from("auth")], program.programId)[0];
  const escrow = PublicKey.findProgramAddressSync([Buffer.from("escrow"), maker.publicKey.toBytes(), seed.toBuffer().reverse()], program.programId)[0];
  const vault = PublicKey.findProgramAddressSync([Buffer.from("vault"), escrow.toBuffer()], program.programId)[0];

  // Mints
  let maker_token: PublicKey;
  let taker_token: PublicKey;

  // ATAs
  let maker_ata: PublicKey; // Maker + maker token
  let taker_ata: PublicKey; // Taker + taker token
  let maker_receive_ata: PublicKey; // Maker + taker token
  let taker_receive_ata: PublicKey; // Taker + maker
  
  it("Airdrop", async () => {
    await Promise.all([maker, taker].map(async (k) => {
      return await anchor.getProvider().connection.requestAirdrop(k.publicKey, 100 * anchor.web3.LAMPORTS_PER_SOL)
    })).then(confirmTxs);
  });

  it("Mint maker/taker tokens", async () => {
    // Create mints and ATAs
    let [ m, t ] = await Promise.all([maker, taker].map(async(a) => { return await newMintToAta(anchor.getProvider().connection, a) }))
    maker_token = m.mint;
    taker_token = t.mint;
    maker_ata = m.ata;
    taker_ata = t.ata;
    // Create take ATAs
    maker_receive_ata = await getAssociatedTokenAddress(taker_token, maker.publicKey, false, tokenProgram);
    taker_receive_ata = await getAssociatedTokenAddress(maker_token, taker.publicKey, false, tokenProgram);
  })

  it("Make an escrow with an invalid slot time", async () => {
    try {
      const signature = await program.methods
      .make(
        seed,
        new anchor.BN(10 * 1e6),
        new anchor.BN(20 * 1e6),
        new anchor.BN(100_000),
      )
      .accounts({
        maker: maker.publicKey,
        makerAta: maker_ata,
        makerToken: maker_token,
        takerToken: taker_token,
        auth,
        escrow,
        vault,
        tokenProgram,
        associatedTokenProgram,
        systemProgram: SystemProgram.programId
      })
      .signers(
        [
          maker
        ]
      )
      .rpc()
      await(confirmTx);
    } catch(e) {
      const err = e as anchor.AnchorError;
      assert(err.error.errorCode.code === 'MaxExpiryExceeded');
    }
  });

  it("Make an escrow", async () => {
    const signature = await program.methods
    .make(
      seed,
      new anchor.BN(10 * 1e6),
      new anchor.BN(20 * 1e6),
      new anchor.BN(10_000),
    )
    .accounts({
      maker: maker.publicKey,
      makerAta: maker_ata,
      makerToken: maker_token,
      takerToken: taker_token,
      auth,
      escrow,
      vault,
      tokenProgram,
      associatedTokenProgram,
      systemProgram: SystemProgram.programId
    })
    .signers(
      [
        maker
      ]
    )
    .rpc()
    await(confirmTx);
  });

  it("Remake an existing escrow", async () => {
    let error = null;
    try {
      const signature = await program.methods
      .make(
        seed,
        new anchor.BN(10 * 1e6),
        new anchor.BN(20 * 1e6),
        new anchor.BN(10_000)
      )
      .accounts({
        maker: maker.publicKey,
        makerAta: maker_ata,
        makerToken: maker_token,
        takerToken: taker_token,
        auth,
        escrow,
        vault,
        tokenProgram,
        associatedTokenProgram,
        systemProgram: SystemProgram.programId
      })
      .signers(
        [
          maker
        ]
      )
      .rpc()
      await(confirmTx);
    } catch(e: any) {
      error = e;
    }
    assert(error.logs[3].startsWith("Allocate: account Address") && error.logs[3].endsWith("already in use"));
  });

  it("Refund an escrow", async () => {
    const signature = await program.methods
    .refund()
    .accounts({
      maker: maker.publicKey,
      makerAta: maker_ata,
      makerToken: maker_token,
      auth,
      escrow,
      vault,
      tokenProgram,
      associatedTokenProgram,
      systemProgram: SystemProgram.programId
    })
    .signers(
      [
        maker
      ]
    )
    .rpc()
    await(confirmTx);
  });

  it("Remake an escrow", async () => {
    const signature = await program.methods
    .make(
      seed,
      new anchor.BN(10 * 1e6),
      new anchor.BN(20 * 1e6),
      new anchor.BN(10_000)
    )
    .accounts({
      maker: maker.publicKey,
      makerAta: maker_ata,
      makerToken: maker_token,
      takerToken: taker_token,
      auth,
      escrow,
      vault,
      tokenProgram,
      associatedTokenProgram,
      systemProgram: SystemProgram.programId
    })
    .signers(
      [
        maker
      ]
    )
    .rpc()
    await(confirmTx);
  });

  it("Update an escrow", async () => {
    const signature = await program.methods
    .update(
      new anchor.BN(12 * 1e6),
      new anchor.BN(1),
    )
    .accounts({
      maker: maker.publicKey,
      newTakerToken: taker_token,
      escrow,
      systemProgram: SystemProgram.programId
    })
    .signers(
      [maker]
    )
    .rpc()
    await(confirmTx);
    setTimeout(() => {}, 2000);
  });

  it("Take an expired escrow", async () => {
    try {
      const signature = await program.methods
      .take()
      .accounts({
        taker: taker.publicKey,
        takerAta: taker_ata,
        takerReceiveAta: taker_receive_ata,
        takerToken: taker_token,
        maker: maker.publicKey,
        makerReceiveAta: maker_receive_ata,
        makerToken: maker_token,
        auth,
        escrow,
        vault,
        tokenProgram,
        associatedTokenProgram,
        systemProgram: SystemProgram.programId
      })
      .signers(
        [
          taker
        ]
      )
      .rpc()
      await(confirmTx);
    } catch(e) {
      const err = e as anchor.ProgramError;
      assert(err.msg === 'Escrow has expired');
    }
  });

  it("Update an escrow to no longer be expired", async () => {
    const signature = await program.methods
    .update(
      new anchor.BN(12 * 1e6),
      new anchor.BN(1000),
    )
    .accounts({
      maker: maker.publicKey,
      newTakerToken: taker_token,
      escrow,
      systemProgram: SystemProgram.programId
    })
    .signers(
      [maker]
    )
    .rpc()
    await(confirmTx);
  });

  it("Take escrow", async () => {
    try {
      const signature = await program.methods
      .take()
      .accounts({
        taker: taker.publicKey,
        takerAta: taker_ata,
        takerReceiveAta: taker_receive_ata,
        takerToken: taker_token,
        maker: maker.publicKey,
        makerReceiveAta: maker_receive_ata,
        makerToken: maker_token,
        auth,
        escrow,
        vault,
        tokenProgram,
        associatedTokenProgram,
        systemProgram: SystemProgram.programId
      })
      .signers(
        [
          taker
        ]
      )
      .rpc()
      await(confirmTx);
    } catch(e) {
      throw(e)
    }
  });
});



// Helpers

const confirmTx = async (signature: string) => {
  const latestBlockhash = await anchor.getProvider().connection.getLatestBlockhash();
  await anchor.getProvider().connection.confirmTransaction(
    {
      signature,
      ...latestBlockhash,
    },
    commitment
  )
}

const confirmTxs = async (signatures: string[]) => {
  await Promise.all(signatures.map(confirmTx))
}

const newMintToAta = async (connection, minter: Keypair): Promise<{ mint: PublicKey, ata: PublicKey }> => { 
  const mint = await createMint(connection, minter, minter.publicKey, null, 6)
  // await getAccount(connection, mint, commitment)
  const ata = await createAccount(connection, minter, mint, minter.publicKey)
  const signature = await mintTo(connection, minter, mint, ata, minter, 21e8)
  await confirmTx(signature)
  return {
    mint,
    ata
  }
}