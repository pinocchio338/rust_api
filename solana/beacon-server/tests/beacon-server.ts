import * as anchor from "@project-serum/anchor";
import { assert, expect } from "chai";
import nacl from 'tweetnacl';
import * as fs from "fs";
import { bufferU64BE, createRawDatapointBuffer, deriveBeaconId, deriveDApiId, deriveDatapointPDA, encodeData, prepareMessage } from "./utils";
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
  templateID: number,
  timestamp: number,
  data: number,
  program: anchor.Program,
  beaconIdPDA: anchor.web3.PublicKey,
  storageFunder: anchor.web3.Keypair,
  txnRelayerKey: anchor.web3.PublicKey,
): Promise<[Uint8Array, Buffer]> {
  // const bufferedTimestamp = Buffer.allocUnsafe(32);
  // bufferedTimestamp.writeBigInt64BE(BigInt(0), 0);
  // bufferedTimestamp.writeBigInt64BE(BigInt(0), 8);
  // bufferedTimestamp.writeBigInt64BE(BigInt(0), 16);
  // bufferedTimestamp.writeBigInt64BE(BigInt(timestamp), 24);
  const bufferedTimestamp = bufferU64BE(timestamp);
  const encodedData = encodeData(data);

  const method = program.instruction.updateBeaconWithSignedData(
    beaconID,
    bufferU64BE(templateID),
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

  const airnode1 = anchor.web3.Keypair.generate();
  const airnode2 = anchor.web3.Keypair.generate();
  const airnode3 = anchor.web3.Keypair.generate();
  const airnode4 = anchor.web3.Keypair.generate();
  const messageRelayer = anchor.web3.Keypair.generate();

  // define all the data
  const templateId1 = 1;
  const timestamp1 = 1649133996;    
  const data1 = 121;

  const templateId2 = 1;
  const timestamp2 = 1649133997;    
  const data2 = 122;

  const templateID3 = 1;
  const timestamp3 = 1649133998;
  const data3 = 123;

  const templateID4 = 1;
  const timestamp4 = 1649134000;
  const data4 = 125;

  before(async () => {
    // fund the accounts one shot
    await provider.connection.confirmTransaction(await provider.connection.requestAirdrop(airnode1.publicKey, anchor.web3.LAMPORTS_PER_SOL));
    await provider.connection.confirmTransaction(await provider.connection.requestAirdrop(airnode2.publicKey, anchor.web3.LAMPORTS_PER_SOL));
    await provider.connection.confirmTransaction(await provider.connection.requestAirdrop(airnode3.publicKey, anchor.web3.LAMPORTS_PER_SOL));
    await provider.connection.confirmTransaction(await provider.connection.requestAirdrop(airnode4.publicKey, anchor.web3.LAMPORTS_PER_SOL));
    await provider.connection.confirmTransaction(await provider.connection.requestAirdrop(messageRelayer.publicKey, anchor.web3.LAMPORTS_PER_SOL));
  })

  it("updateBeaconWithSignedData", async () => {
    // 1. Airnode create the txn
    const beaconId = deriveBeaconId(airnode3.publicKey.toBytes(), templateID3);
    const beaconIdPDA = await deriveDatapointPDA(beaconId, program.programId);
    // console.log("raw beaconId with length", beaconId.length, "and value", beaconId.toString("hex"), "pda", beaconIdPDA.toString(), program.programId);

    const [airnodeSignature, airnodeTxn] = await newUpdateBeaconWithSignedDataTxn(
      beaconId,
      templateID3,
      timestamp3,
      data3,
      program,
      beaconIdPDA,
      airnode3,
      messageRelayer.publicKey
    );

    // 2. Relay the transaction
    const offlineTxn = await relayTxn(airnodeTxn, airnodeSignature, airnode3.publicKey, messageRelayer);

    // 3. Send transaction
    await provider.connection.sendRawTransaction(offlineTxn);

    // wait a bit for the transaction to take effect
    await delay(1000);

    const wrappedDataPoint = await program.account.wrappedDataPoint.fetch(beaconIdPDA);

    // construct expected
    const expected = createRawDatapointBuffer(data3, timestamp3);
    expect(wrappedDataPoint.rawDatapoint).to.deep.eq(expected);
  });

  it("updateDapiWithBeacons", async () => {
    // Create the datapoint 4
    const beaconId4 = deriveBeaconId(airnode4.publicKey.toBytes(), templateID4);
    const beaconIdPDA4 = await deriveDatapointPDA(beaconId4, program.programId);
    // console.log("raw beaconId with length", beaconId.length, "and value", beaconId.toString("hex"), "pda", beaconIdPDA.toString(), program.programId);

    const [airnodeSignature, airnodeTxn] = await newUpdateBeaconWithSignedDataTxn(
      beaconId4,
      templateID4,
      timestamp4,
      data4,
      program,
      beaconIdPDA4,
      airnode4,
      messageRelayer.publicKey
    );
    const offlineTxn = await relayTxn(airnodeTxn, airnodeSignature, airnode4.publicKey, messageRelayer);
    await provider.connection.sendRawTransaction(offlineTxn);

    // wait a bit for the transaction to take effect
    await delay(1000);

    const wrappedDataPoint4 = await program.account.wrappedDataPoint.fetch(beaconIdPDA4);
    const expected = createRawDatapointBuffer(data4, timestamp4);
    expect(wrappedDataPoint4.rawDatapoint).to.deep.eq(expected);

    // now test updateDapiWithBeacons
    const beaconId3 = deriveBeaconId(airnode3.publicKey.toBytes(), templateID3);
    const beaconIds = [beaconId3, beaconId4];
    const dataPointId = deriveDApiId(beaconIds);
    const dapiPDA = await deriveDatapointPDA(dataPointId, program.programId);

    const updateInstruction = await program.instruction.updateDapiWithBeacons(
      dataPointId,
      beaconIds,
      {
        accounts: {
          datapoint: dapiPDA,
          user: messageRelayer.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        },
        remainingAccounts: [
          { isSigner: false, isWritable: false, pubkey: await deriveDatapointPDA(beaconId3, program.programId) },
          { isSigner: false, isWritable: false, pubkey: await deriveDatapointPDA(beaconId4, program.programId) },
        ]
      }
    );

    const tx = new anchor.web3.Transaction();
    tx.add(updateInstruction);
    await anchor.web3.sendAndConfirmTransaction(
      provider.connection,
      tx,
      [messageRelayer],
    );

    let wrappedDataPoint = await program.account.wrappedDataPoint.fetch(dapiPDA);
    expect(wrappedDataPoint.rawDatapoint).to.deep.eq(createRawDatapointBuffer(124, 1649133999));
  });

  it("updateDapiWithSignedData", async () => {
    // Step 1. Airnode1 create the data
    const message1 = prepareMessage(templateId1, timestamp1, data1);
    const sig1 = nacl.sign.detached(message1, airnode1.secretKey);

    // Step 2. Airnode2 create the data
    const message2 = prepareMessage(templateId2, timestamp2, data2);
    const sig2 = nacl.sign.detached(message2, airnode2.secretKey);
    
    // Step 3. Airnode3 no data
    const templateId3 = 1;
    const timestamp3 = 1649133997; 

    // Step 4. Create the transaction call
    const beaconId1 = deriveBeaconId(airnode1.publicKey.toBytes(), templateId1);
    const beaconId2 = deriveBeaconId(airnode2.publicKey.toBytes(), templateId2);
    const beaconId3 = deriveBeaconId(airnode3.publicKey.toBytes(), templateId3);
    const beaconIds = [beaconId1, beaconId2, beaconId3];
    // console.log("beaconId3", [beaconId1, beaconId2, beaconId3].map(b => b.toString("hex")));

    const dataPointId = deriveDApiId(beaconIds);

    const sigVerify = createInstructionWithPublicKey(
      [
        { publicKey: airnode1.publicKey.toBytes(), message: message1, signature: sig1 },
        { publicKey: airnode2.publicKey.toBytes(), message: message2, signature: sig2 }
      ],
      0
    );

    const dapiPDA = await deriveDatapointPDA(dataPointId, program.programId);
    const remainingAccounts = [{ isSigner: false, isWritable: false, pubkey: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY }];
    for (const id of [beaconId3]) {
      const pda = await deriveDatapointPDA(id, program.programId);
      const wrappedDataPoint = await program.account.wrappedDataPoint.fetch(pda);
      expect(wrappedDataPoint.rawDatapoint.length > 0).to.eq(true);
      remainingAccounts.push({ isSigner: false, isWritable: false, pubkey: pda });
    }

    const updateInstruction = program.instruction.updateDapiWithSignedData(
      dataPointId,
      [airnode1, airnode2, airnode3].map(t => t.publicKey.toBytes()),
      beaconIds,
      [templateId1, templateId2, templateId3].map(t => bufferU64BE(t)),
      [timestamp1, timestamp2, timestamp3].map(t => bufferU64BE(t)),
      [data1, data2, data1].map(t => encodeData(t)),
      {
        accounts: {
          datapoint: dapiPDA,
          user: messageRelayer.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        },
        remainingAccounts,
      }
    );

    const tx = new anchor.web3.Transaction();
    tx.add(sigVerify);
    tx.add(updateInstruction);

    await anchor.web3.sendAndConfirmTransaction(
      provider.connection,
      tx,
      [messageRelayer],
    );

    // wait a bit for the transaction to take effect
    await delay(1000);

    const wrappedDataPoint = await program.account.wrappedDataPoint.fetch(dapiPDA);
    const expected = createRawDatapointBuffer(data2, timestamp2);
    expect(wrappedDataPoint.rawDatapoint).to.deep.eq(expected);
  });
});
