import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Voting } from "../target/types/voting";
import { PublicKey } from "@solana/web3.js";

describe("voting", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.voting as Program<Voting>;

  it("initializePoll", async () => {
    const [pollAddress] = PublicKey.findProgramAddressSync(
      [Buffer.from("poll"), new anchor.BN(1).toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    console.log(pollAddress.toBase58());

    const tx = await program.methods
      .initializePoll(
        new anchor.BN(1),
        new anchor.BN(1751972603),
        new anchor.BN(1752220223),
        "Akash Dalal",
        "description"
      )
      .rpc();

    console.log({ tx });
  });

  it("initialize candidate ", async () => {
    const pollIdBuffer = new anchor.BN(1).toArrayLike(Buffer, "le", 8);

    // console.log({ pollIdBuffer });

    const [pollAddress] = PublicKey.findProgramAddressSync(
      [Buffer.from("poll"), new anchor.BN(1).toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    const crunchyTx = await program.methods
      .initializeCandidate(new anchor.BN(1), "Akash DAlal")
      .accounts({
        pollAccount: pollAddress,
      })
      .rpc();

    console.log({ crunchyTx });

    const data = await program.account.pollAccount.fetch(pollAddress);
    console.log(data);

    const [candidateAddress] = PublicKey.findProgramAddressSync(
      [
        new anchor.BN(1).toArrayLike(Buffer, "le", 8),
        Buffer.from("Akash DAlal"),
      ],
      program.programId
    );

    console.log({ candidateAddress });
  });

  it("vote ", async () => {
    for (let i = 0; i < 40; i++) {
      const tx = await program.methods
        .vote(new anchor.BN(1), "Akash DAlal")
        .rpc();

      console.log({ tx });
    }

    const [pollAddress] = PublicKey.findProgramAddressSync(
      [Buffer.from("poll"), new anchor.BN(1).toArrayLike(Buffer, "le", 8)],
      program.programId
    );
    const data = await program.account.pollAccount.fetch(pollAddress);
    console.log(data);

    const [candidateAddress] = PublicKey.findProgramAddressSync(
      [
        new anchor.BN(1).toArrayLike(Buffer, "le", 8),
        Buffer.from("Akash DAlal"),
      ],
      program.programId
    );

    console.log({ candidateAddress });
  });
});
