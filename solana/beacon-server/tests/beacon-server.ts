import * as anchor from "@project-serum/anchor";
import { assert, expect } from "chai";
import nacl from 'tweetnacl';
import * as fs from "fs";
import { deriveBeaconId, encodeData } from "./utils";
import { createInstructionWithPublicKey } from "./sig";

const delay = ms => new Promise(resolve => setTimeout(resolve, ms))

/**
 * Create a new offline UpdateBeaconWithSignedData transasction
 * 
 * @param beaconID 
 * @param templateID 
 * @param timestamp 
 * @param data 
 * @param program 
 * @param beaconIdPDA 
 * @param storageFunderKey 
 * @param txnRelayerKey 
 * @returns The serialized offline transaction buffer
 */
async function newUpdateBeaconWithSignedDataTxn(
  beaconID: Buffer,
  templateID: Buffer,
  timestamp: number,
  data: number,
  program: anchor.Program,
  beaconIdPDA: anchor.web3.PublicKey,
  storageFunder: anchor.web3.Keypair,
  txnRelayerKey: anchor.web3.PublicKey,
): Promise<[Uint8Array, Buffer]> {
  const bufferedTimestamp = Buffer.allocUnsafe(32);
  bufferedTimestamp.writeBigInt64BE(BigInt(0), 0);
  bufferedTimestamp.writeBigInt64BE(BigInt(0), 8);
  bufferedTimestamp.writeBigInt64BE(BigInt(0), 16);
  bufferedTimestamp.writeBigInt64BE(BigInt(timestamp), 24);
  
  const encodedData = encodeData(data);

  const method = program.instruction.updateBeaconWithSignedData(
    beaconID,
    templateID,
    bufferedTimestamp,
    encodedData,
    {
      accounts: {
        datapoint: beaconIdPDA,
        user: storageFunder.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      }
    }
  );
  
  const tx = new anchor.web3.Transaction().add(method);
  tx.recentBlockhash = (await program.provider.connection.getLatestBlockhash()).blockhash;
  tx.feePayer = txnRelayerKey;

  const rawTxn = tx.serializeMessage();
  const signature = nacl.sign.detached(rawTxn, storageFunder.secretKey);
  return [signature, rawTxn];
}

async function relayTxn(
  rawTxn: Buffer,
  storageSignature: Uint8Array,
  storageFunderKey: anchor.web3.PublicKey,
  relayer: anchor.web3.Keypair,
): Promise<Buffer> {
  const relayerSignature = nacl.sign.detached(rawTxn, relayer.secretKey);
  let recoverTx = anchor.web3.Transaction.populate(anchor.web3.Message.from(rawTxn));
  recoverTx.addSignature(relayer.publicKey, Buffer.from(relayerSignature));
  recoverTx.addSignature(storageFunderKey, Buffer.from(storageSignature));

  return recoverTx.serialize();
}

describe("beacon-server", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.Provider.env();
  anchor.setProvider(provider);

  const idl = JSON.parse(
    fs.readFileSync("./target/idl/beacon_server.json", "utf8")
  );
  const programId = new anchor.web3.PublicKey("FRoo7m8Sf6ZAirGgnn3KopQymDtujWx818kcnRxzi23b");
  const program = new anchor.Program(idl, programId);

  const airnode = anchor.web3.Keypair.generate();
  const messageRelayer = anchor.web3.Keypair.generate();

  before(async () => {
    // fund the accounts one shot
    await provider.connection.confirmTransaction(await provider.connection.requestAirdrop(airnode.publicKey, anchor.web3.LAMPORTS_PER_SOL));
    await provider.connection.confirmTransaction(await provider.connection.requestAirdrop(messageRelayer.publicKey, anchor.web3.LAMPORTS_PER_SOL));
  })
  
  it("updateBeaconWithSignedData", async () => {
    // 1. Airnode create the txn
    const templateID = Buffer.allocUnsafe(32);

    const timestamp = 1649133996;    
    const data = 123;

    const beaconId = deriveBeaconId(airnode.publicKey.toBytes(), templateID);
    console.log("raw beaconId with length", beaconId.length, "and value", beaconId.toString("hex"));

    const [beaconIdPDA] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(anchor.utils.bytes.utf8.encode("datapoint")),
        beaconId
      ],
      program.programId
    );

    const [airnodeSignature, airnodeTxn] = await newUpdateBeaconWithSignedDataTxn(
      beaconId,
      templateID,
      timestamp,
      data,
      program,
      beaconIdPDA,
      airnode,
      messageRelayer.publicKey
    );
    
    // 2. Relay the transaction
    const offlineTxn = await relayTxn(airnodeTxn, airnodeSignature, airnode.publicKey, messageRelayer);

    // 3. Send transaction
    await provider.connection.sendRawTransaction(offlineTxn);

    // wait a bit for the transaction to take effect
    await delay(1000);

    const wrappedDataPoint = await program.account.wrappedDataPoint.fetch(beaconIdPDA);

    // construct expected
    const expected = Buffer.allocUnsafe(36);
    expected.writeBigInt64BE(BigInt(0), 0);
    expected.writeBigInt64BE(BigInt(0), 8);
    expected.writeBigInt64BE(BigInt(0), 16);
    expected.writeBigInt64BE(BigInt(data), 24);
    expected.writeUInt32BE(timestamp, 32);

    expect(wrappedDataPoint.rawDatapoint).to.deep.eq(expected);
  });

  // it("updateDapiWithBeacons", async () => {
  //   const [beaconIdPDA] = await anchor.web3.PublicKey.findProgramAddress(
  //     [
  //       Buffer.from(anchor.utils.bytes.utf8.encode("datapoint")),
  //       beaconID
  //     ],
  //     program.programId
  //   )

  //   const tempDAPIId = Buffer.from("1".padEnd(64, "0"), "hex");
  //   const [dapiPDA] = await anchor.web3.PublicKey.findProgramAddress(
  //     [
  //       Buffer.from(anchor.utils.bytes.utf8.encode("datapoint")),
  //       tempDAPIId
  //     ],
  //     program.programId
  //   )

  //   const tx = await program.rpc.updateDapiWithBeacons(
  //     tempDAPIId,
  //     [beaconID],
  //     {
  //       accounts: {
  //         dapi: dapiPDA,
  //         user: anchor.getProvider().wallet.publicKey,
  //         systemProgram: anchor.web3.SystemProgram.programId,
  //       },
  //       remainingAccounts: [
  //         { isSigner: false, isWritable: false, pubkey: beaconIdPDA }
  //       ],
  //     }
  //   );

  //   const wrappedDataPoint = await program.account.wrappedDataPoint.fetch(dapiPDA);
  //   console.log(JSON.stringify(wrappedDataPoint));
  //   // expect(wrappedDataPoint.rawDatapoint).to.deep.eq(data);
  // });

  // it("updateDapiWithSignedData", async () => {
  //   const dataPointId = Buffer.allocUnsafe(32);
  //   const beaconID = Buffer.allocUnsafe(32);
  //   const tempDAPIId = Buffer.from("1".padEnd(64, "0"), "hex");
  //   const timestamp = Buffer.allocUnsafe(32);
  //   const data = Buffer.allocUnsafe(32);
  //   const sigs = Buffer.allocUnsafe(32);
    
  //   const msgStr = "The quick brown fox jumps over the lazy dog";
  //   const messageBytes = Buffer.from(msgStr, "ascii");

  //   const instruction = createInstructionWithPublicKey(
  //     [
  //       {
  //         publicKey: airnode.publicKey.toBytes(),
  //         message: messageBytes,
  //         signature: nacl.sign.detached(messageBytes, airnode.secretKey),
  //       },
  //       {
  //         publicKey: messageRelayer.publicKey.toBytes(),
  //         message: messageBytes,
  //         signature: nacl.sign.detached(messageBytes, messageRelayer.secretKey),
  //       }
  //     ],
  //     0
  //   );
  //   // expect(instruction.data).to.deep.eq(sigVerifInstruction1.data);

  //   const [dapiPDA] = await anchor.web3.PublicKey.findProgramAddress(
  //     [
  //       Buffer.from(anchor.utils.bytes.utf8.encode("datapoint")),
  //       dataPointId
  //     ],
  //     program.programId
  //   )

  //   const actual = program.instruction.updateDapiWithSignedData(
  //     dataPointId,
  //     [beaconID],
  //     [tempDAPIId],
  //     [timestamp],
  //     [data],
  //     {
  //       accounts: {
  //         datapoint: dapiPDA,
  //         user: messageRelayer.publicKey,
  //         systemProgram: anchor.web3.SystemProgram.programId,
  //       },
  //       remainingAccounts: [
  //         { isSigner: false, isWritable: false, pubkey: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY }
  //       ],
  //     }
  //   );

  //   const tx = new anchor.web3.Transaction();
  //   tx.add(instruction);
  //   tx.add(actual);

  //   await anchor.web3.sendAndConfirmTransaction(
  //     provider.connection,
  //     tx,
  //     [messageRelayer],
  //   );
  // });
});
