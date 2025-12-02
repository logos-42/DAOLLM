import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Daollm } from "../target/types/daollm";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import { expect } from "chai";

describe("daollm", () => {
  // Configure the client
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Daollm as Program<Daollm>;
  
  const submitter = Keypair.generate();
  const nodeOwner = Keypair.generate();
  
  let proposalId: string;
  let proposalPda: PublicKey;
  let nodePda: PublicKey;

  before(async () => {
    // Airdrop SOL to test accounts
    await provider.connection.requestAirdrop(
      submitter.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.requestAirdrop(
      nodeOwner.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL
    );
    
    // Wait for airdrop confirmation
    await new Promise(resolve => setTimeout(resolve, 1000));
  });

  it("Submits a proposal", async () => {
    proposalId = `proposal-${Date.now()}`;
    const ipfsHash = "QmTestHash123456789";
    
    const [proposal] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("proposal"),
        Buffer.from(proposalId),
      ],
      program.programId
    );
    proposalPda = proposal;

    const tx = await program.methods
      .submitProposal(proposalId, ipfsHash)
      .accounts({
        submitter: submitter.publicKey,
        proposal: proposalPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([submitter])
      .rpc();

    console.log("Proposal submission transaction:", tx);

    const proposalAccount = await program.account.proposal.fetch(proposalPda);
    expect(proposalAccount.proposalId).to.equal(proposalId);
    expect(proposalAccount.ipfsHash).to.equal(ipfsHash);
    expect(proposalAccount.submitter.toString()).to.equal(submitter.publicKey.toString());
  });

  it("Registers an inference node", async () => {
    const stakeAmount = new anchor.BN(1000 * anchor.web3.LAMPORTS_PER_SOL);
    
    const [node] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("node"),
        nodeOwner.publicKey.toBuffer(),
      ],
      program.programId
    );
    nodePda = node;

    const tx = await program.methods
      .registerNode(stakeAmount)
      .accounts({
        owner: nodeOwner.publicKey,
        node: nodePda,
        systemProgram: SystemProgram.programId,
      })
      .signers([nodeOwner])
      .rpc();

    console.log("Node registration transaction:", tx);

    const nodeAccount = await program.account.inferenceNode.fetch(nodePda);
    expect(nodeAccount.owner.toString()).to.equal(nodeOwner.publicKey.toString());
    expect(nodeAccount.stakeAmount.toNumber()).to.equal(stakeAmount.toNumber());
    expect(nodeAccount.isActive).to.be.true;
  });

  it("Submits an inference result", async () => {
    const resultHash = "QmResultHash987654321";
    const confidence = 85;

    const [inferenceResult] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("inference"),
        proposalPda.toBuffer(),
        nodePda.toBuffer(),
      ],
      program.programId
    );

    const tx = await program.methods
      .submitInference(proposalId, resultHash, confidence)
      .accounts({
        node: nodeOwner.publicKey,
        nodeAccount: nodePda,
        owner: nodeOwner.publicKey,
        proposal: proposalPda,
        inferenceResult: inferenceResult,
        systemProgram: SystemProgram.programId,
      })
      .signers([nodeOwner])
      .rpc();

    console.log("Inference submission transaction:", tx);

    const inferenceAccount = await program.account.inferenceResult.fetch(inferenceResult);
    expect(inferenceAccount.proposalId).to.equal(proposalId);
    expect(inferenceAccount.resultHash).to.equal(resultHash);
    expect(inferenceAccount.confidence).to.equal(confidence);
  });

  it("Aggregates inference results", async () => {
    const aggregator = Keypair.generate();
    await provider.connection.requestAirdrop(
      aggregator.publicKey,
      1 * anchor.web3.LAMPORTS_PER_SOL
    );
    await new Promise(resolve => setTimeout(resolve, 1000));

    const tx = await program.methods
      .aggregateResults(proposalId)
      .accounts({
        aggregator: aggregator.publicKey,
        proposal: proposalPda,
      })
      .signers([aggregator])
      .rpc();

    console.log("Aggregation transaction:", tx);

    const proposalAccount = await program.account.proposal.fetch(proposalPda);
    // Check that status is Completed (enum value 2)
    expect(proposalAccount.status).to.have.property("completed");
  });

  it("Rates a node", async () => {
    const rater = Keypair.generate();
    await provider.connection.requestAirdrop(
      rater.publicKey,
      1 * anchor.web3.LAMPORTS_PER_SOL
    );
    await new Promise(resolve => setTimeout(resolve, 1000));

    const score = 90;

    const tx = await program.methods
      .rateNode(nodeOwner.publicKey, score)
      .accounts({
        rater: rater.publicKey,
        node: nodePda,
        nodeAddress: nodeOwner.publicKey,
      })
      .signers([rater])
      .rpc();

    console.log("Node rating transaction:", tx);

    const nodeAccount = await program.account.inferenceNode.fetch(nodePda);
    // Initial score is 50, new score should be (50 + 90) / 2 = 70
    expect(nodeAccount.reputationScore).to.be.greaterThan(50);
  });
});

