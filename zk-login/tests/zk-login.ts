import * as anchor from "@coral-xyz/anchor";
import { Program, web3 } from "@coral-xyz/anchor";
import { ZkLogin } from "../target/types/zk_login";
import { expect } from "chai";
import { buildPoseidon } from "circomlibjs";
import { groth16 } from "snarkjs";
import { utils, buildBn128 } from "ffjavascript";

async function poseidonHash(inputs) {
  const poseidon = await buildPoseidon();
  const poseidonHash = poseidon.F.toString(poseidon(inputs));
  return poseidonHash;
}

function g1Uncompressed(curve, p1Raw) {
  let p1 = curve.G1.fromObject(p1Raw);

  let buff = new Uint8Array(64); // 64 bytes for G1 uncompressed
  curve.G1.toRprUncompressed(buff, 0, p1);

  return Buffer.from(buff);
}

function g2Uncompressed(curve, p2Raw) {
  let p2 = curve.G2.fromObject(p2Raw);

  let buff = new Uint8Array(128); // 128 bytes for G2 uncompressed
  curve.G2.toRprUncompressed(buff, 0, p2);

  return Buffer.from(buff);
}

function reverseEndianness(buffer) {
  return Buffer.from(buffer.reverse());
}

function to32ByteBuffer(bigInt) {
  const hexString = bigInt.toString(16).padStart(64, '0'); // Pad to 64 hex characters (32 bytes)
  const buffer = Buffer.from(hexString, "hex");
  return buffer; 
}

async function negateAndSerializeG1(curve, reversedP1Uncompressed) {
  if (!reversedP1Uncompressed || !(reversedP1Uncompressed instanceof Uint8Array || Buffer.isBuffer(reversedP1Uncompressed))) {
    console.error('Invalid input to negateAndSerializeG1:', reversedP1Uncompressed);
    throw new Error('Invalid input to negateAndSerializeG1');
  }
  // Negate the G1 point
  let p1 = curve.G1.toAffine(curve.G1.fromRprUncompressed(reversedP1Uncompressed, 0));
  let negatedP1 = curve.G1.neg(p1);

  // Serialize the negated point
  // The serialization method depends on your specific library
  let serializedNegatedP1 = new Uint8Array(64); // 32 bytes for x and 32 bytes for y
  curve.G1.toRprUncompressed(serializedNegatedP1, 0, negatedP1);
  // curve.G1.toRprUncompressed(serializedNegatedP1, 32, negatedP1.y);
  console.log(serializedNegatedP1)

  // Change endianness if necessary
  let proof_a = reverseEndianness(serializedNegatedP1);

  return proof_a;
}

describe("zk-login", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const provider = anchor.getProvider();

  const program = anchor.workspace.ZkLogin as Program<ZkLogin>;
  const embeddingHash = 1;
  const microchipId = 1;


  const [userDataPda] = web3.PublicKey.findProgramAddressSync(
    [Buffer.from('user_data'), provider.publicKey.toBuffer()],
    program.programId
);

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
  
  it("user sign up", async () => {
    const hash = await poseidonHash([embeddingHash, microchipId])
    const tx = await program.methods.userSignUp(hash).rpc();

    const account = await program.account.userData.fetch(userDataPda);
    console.log("user hash", account.hash)

    expect(hash === account.hash)
  })

  it("user sign in", async () => {
    const hash = await poseidonHash([embeddingHash, microchipId])
    let { proof, publicSignals } = await groth16.fullProve(
      {
        embeddingHash,
        microchipId,
        hash
      },
      "circom/embedding_hash_proof.wasm",
      "circom/EmbeddingHashProof.zkey"
    )
    const curve = await buildBn128();

    const { unstringifyBigInts } = utils;
    let proofProc = unstringifyBigInts(proof);
    publicSignals = unstringifyBigInts(publicSignals);

    let pi_a = g1Uncompressed(curve, proofProc.pi_a);
    console.log("pi_a", Buffer.from(pi_a))
    pi_a = reverseEndianness(pi_a);
    pi_a = await negateAndSerializeG1(curve, pi_a);
    let pi_a_u8_array = Array.from(pi_a);
    console.log("proof", pi_a_u8_array);

    const pi_b = g2Uncompressed(curve, proofProc.pi_b);
    let pi_b_0_u8_array = Array.from(pi_b);
    console.log("pi_b_0_u8_array.slice(0, 64)", pi_b_0_u8_array.slice(0, 64));
    console.log("pi_b_0_u8_array.slice(64, 128)", pi_b_0_u8_array.slice(64, 128));

    const pi_c = g1Uncompressed(curve, proofProc.pi_c);
    let pi_c_0_u8_array = Array.from(pi_c);
    console.log("pi_c_0_u8_array", pi_c_0_u8_array);

    const publicSignalsBuffer = to32ByteBuffer(BigInt(publicSignals[0]));
    let public_signal_0_u8_array = Array.from(publicSignalsBuffer);
    console.log("public_signal_0_u8_array", public_signal_0_u8_array);

    const serializedData = Buffer.concat([
      pi_a,
      pi_b,
      pi_c,
      publicSignalsBuffer
    ]);

    const tx = await program.methods.userSignIn(serializedData).rpc();
  });
});
