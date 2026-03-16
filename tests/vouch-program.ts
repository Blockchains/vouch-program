import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { VouchProgram } from "../target/types/vouch_program";
import { PublicKey, Keypair } from "@solana/web3.js";

describe("vouch-program", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.VouchProgram as Program<VouchProgram>;
  const wallet = anchor.getProvider().publicKey;
  const agentWallet = Keypair.generate();

  it("registers an agent", async () => {
    const [agentPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("agent"), agentWallet.publicKey.toBuffer()],
      program.programId
    );

    const tx = await program.methods
      .registerAgent("Truth Terminal", "Semi-autonomous AI agent", "social-media, trading")
      .accounts({
        agent: agentPda,
        wallet: agentWallet.publicKey,
        authority: wallet,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("Registered agent:", tx);

    const agent = await program.account.agentAccount.fetch(agentPda);
    console.log("Name:", agent.name);
    console.log("Score:", agent.reputationScore);
    console.log("Wallet:", agent.wallet.toBase58());
  });

  it("updates score", async () => {
    const [agentPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("agent"), agentWallet.publicKey.toBuffer()],
      program.programId
    );

    const tx = await program.methods
      .updateScore(45, 75, 0, 1, 5)
      .accounts({
        agent: agentPda,
        scorer: wallet,
      })
      .rpc();

    console.log("Updated score:", tx);

    const agent = await program.account.agentAccount.fetch(agentPda);
    console.log("Reputation:", agent.reputationScore);
    console.log("On-chain:", agent.onchainScore);
    console.log("Flags:", agent.anomalyFlags);
  });

  it("checks reputation", async () => {
    const [agentPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("agent"), agentWallet.publicKey.toBuffer()],
      program.programId
    );

    const tx = await program.methods
      .checkReputation()
      .accounts({
        agent: agentPda,
        checker: wallet,
      })
      .rpc();

    console.log("Checked reputation:", tx);
  });
});
