import * as anchor from "@project-serum/anchor";
import { expect } from "chai";
import nacl from 'tweetnacl';
import * as fs from "fs";
import { 
  bufferU64BE, getRandomInt, median, prepareMessage, relayTxn
} from "./utils";
import { DapiClient } from "./client";

const delay = ms => new Promise(resolve => setTimeout(resolve, ms))

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
  const timestamp1 = Math.floor(Date.now() / 1000);    
  const data1 = getRandomInt(10000000);

  const templateId2 = 2;
  const timestamp2 = Math.floor(Date.now() / 1000) - getRandomInt(500);
  const data2 = getRandomInt(10000000);

  const templateId3 = 3;
  let timestamp3 = Math.floor(Date.now() / 1000) - getRandomInt(500);
  let data3 = getRandomInt(10000000);

  const templateId4 = 4;
  const timestamp4 = Math.floor(Date.now() / 1000) - getRandomInt(500);
  const data4 = getRandomInt(10000000);

  const name = bufferU64BE(123);
  
  const dapiClient = new DapiClient(program, provider);

  before(async () => {
    // fund the accounts one shot
    await provider.connection.confirmTransaction(await provider.connection.requestAirdrop(airnode1.publicKey, anchor.web3.LAMPORTS_PER_SOL));
    await provider.connection.confirmTransaction(await provider.connection.requestAirdrop(airnode2.publicKey, anchor.web3.LAMPORTS_PER_SOL));
    await provider.connection.confirmTransaction(await provider.connection.requestAirdrop(airnode3.publicKey, anchor.web3.LAMPORTS_PER_SOL));
    await provider.connection.confirmTransaction(await provider.connection.requestAirdrop(airnode4.publicKey, anchor.web3.LAMPORTS_PER_SOL));
    await provider.connection.confirmTransaction(await provider.connection.requestAirdrop(messageRelayer.publicKey, anchor.web3.LAMPORTS_PER_SOL));
  }) 

  describe("updateBeaconWithSignedData", () => {
    it("works", async () => {
      // 1. Airnode create the txn    
      const [airnodeSignature, airnodeTxn] = await dapiClient.newUpdateBeaconWithSignedDataTxn(
        templateId3,
        timestamp3,
        data3,
        airnode3,
        messageRelayer.publicKey
      );
  
      // 2. Relay the transaction
      const offlineTxn = await relayTxn(airnodeTxn, airnodeSignature, airnode3.publicKey, messageRelayer);
  
      // 3. Send transaction
      await provider.connection.sendRawTransaction(offlineTxn);
  
      // wait a bit for the transaction to take effect
      await delay(1000);
  
      // Check test response
      const beaconId = dapiClient.deriveBeaconId(airnode3.publicKey.toBytes(), templateId3);
      const datapoint = await dapiClient.readWithDataPointId(beaconId);
  
      // construct expected
      expect(datapoint.value).to.deep.eq(data3);
      expect(datapoint.timestamp).to.deep.eq(timestamp3);
    });

    it("overwrite old data", async () => {
      // 1. Airnode create the txn
      timestamp3 = Math.floor(Date.now() / 1000) + getRandomInt(100);
      data3 = getRandomInt(10000000);
      const [airnodeSignature, airnodeTxn] = await dapiClient.newUpdateBeaconWithSignedDataTxn(
        templateId3,
        timestamp3,
        data3,
        airnode3,
        messageRelayer.publicKey
      );
  
      // 2. Relay the transaction
      const offlineTxn = await relayTxn(airnodeTxn, airnodeSignature, airnode3.publicKey, messageRelayer);
  
      // 3. Send transaction
      await provider.connection.sendRawTransaction(offlineTxn);
  
      // wait a bit for the transaction to take effect
      await delay(1000);
  
      // Check test response
      const beaconId = dapiClient.deriveBeaconId(airnode3.publicKey.toBytes(), templateId3);
      const datapoint = await dapiClient.readWithDataPointId(beaconId);
  
      // construct expected
      expect(datapoint.value).to.deep.eq(data3);
      expect(datapoint.timestamp).to.deep.eq(timestamp3);
    });

    it("invalid signature", async () => {
      // 1. Airnode create the txn    
      const [_, airnodeTxn] = await dapiClient.newUpdateBeaconWithSignedDataTxn(
        templateId3,
        timestamp3,
        data3,
        airnode3,
        messageRelayer.publicKey
      );
  
      // 2. Relay the transaction with wrong signature
      try {
        await relayTxn(airnodeTxn, Buffer.allocUnsafe(32), airnode3.publicKey, messageRelayer);
        expect(false, "should not make it here");
      } catch (e) {
        expect(true, "should fail");
      }
    });
  });
  
  describe("updateBeaconWithSignedData", () => {
    it("works", async () => {
      // Create the datapoint 4 
      const [airnodeSignature, airnodeTxn] =  await dapiClient.newUpdateBeaconWithSignedDataTxn(
        templateId4,
        timestamp4,
        data4,
        airnode4,
        messageRelayer.publicKey
      );
      const offlineTxn = await relayTxn(airnodeTxn, airnodeSignature, airnode4.publicKey, messageRelayer);
      await provider.connection.sendRawTransaction(offlineTxn);
  
      // wait a bit for the transaction to take effect
      await delay(1000);

      const beaconId4 = dapiClient.deriveBeaconId(airnode4.publicKey.toBytes(), templateId4);
      const datapoint = await dapiClient.readWithDataPointId(beaconId4);
      expect(datapoint.value).to.deep.eq(data4);
      expect(datapoint.timestamp).to.deep.eq(timestamp4);

      // now test updateDapiWithBeacons
      const beaconId3 = dapiClient.deriveBeaconId(airnode3.publicKey.toBytes(), templateId3);
      const beaconIds = [beaconId3, beaconId4];
      const dataPointId = dapiClient.deriveDapiId(beaconIds);
      
      await dapiClient.updateDapiWithBeacons(beaconIds, messageRelayer);
      const dapi = await dapiClient.readWithDataPointId(dataPointId);
      expect(dapi.timestamp).to.eq(Math.floor((timestamp3 + timestamp4) / 2));
      expect(dapi.value).to.eq(Math.floor((data3 + data4) / 2));
    });
  });

  describe("updateDapiWithSignedData", async () => {
    it("works", async () => {
      // Step 1. Airnode1 create the data
      const message1 = prepareMessage(templateId1, timestamp1, data1);
      const sig1 = nacl.sign.detached(message1, airnode1.secretKey);
  
      // Step 2. Airnode2 create the data
      const message2 = prepareMessage(templateId2, timestamp2, data2);
      const sig2 = nacl.sign.detached(message2, airnode2.secretKey);
      
      const newTimestamp = timestamp3 + 10000;
      const newData = data3 + 10;

      await dapiClient.updateDapiWithSignedData(
        [airnode1.publicKey, airnode2.publicKey, airnode3.publicKey],
        [templateId1, templateId2, templateId3],
        [timestamp1, timestamp2, newTimestamp],
        [data1, data2, newData],
        [
          { publicKey: airnode1.publicKey.toBytes(), message: message1, signature: sig1 },
          { publicKey: airnode2.publicKey.toBytes(), message: message2, signature: sig2 }
        ],
        messageRelayer
      );
  
      // wait a bit for the transaction to take effect
      await delay(1000);

      const beaconId1 = dapiClient.deriveBeaconId(airnode1.publicKey.toBytes(), templateId1);
      const beaconId2 = dapiClient.deriveBeaconId(airnode2.publicKey.toBytes(), templateId2);
      const beaconId3 = dapiClient.deriveBeaconId(airnode3.publicKey.toBytes(), templateId3);
      const beaconIds = [beaconId1, beaconId2, beaconId3];
      const dapi = dapiClient.deriveDapiId(beaconIds);
      
      const datapoint = await dapiClient.readWithDataPointId(dapi);
      // newData and newTimestamp are not reflected here
      expect(datapoint.value).to.eq(median([data1, data2, data3]));
      expect(datapoint.timestamp).to.eq(Math.floor((timestamp1 + timestamp2 + timestamp3) / 3));
    });

    it("wrong signature", async () => {
      // Step 1. Airnode1 create the data
      const message1 = prepareMessage(templateId1, timestamp1, data1);
  
      // Step 2. Airnode2 create the data
      const message2 = prepareMessage(templateId2, timestamp2, data2);
      const sig2 = nacl.sign.detached(message2, airnode2.secretKey);
            
      try {
        await dapiClient.updateDapiWithSignedData(
          [airnode1.publicKey, airnode2.publicKey, airnode3.publicKey],
          [templateId1, templateId2, templateId3],
          [timestamp1, timestamp2, timestamp3],
          [data1, data2, data1],
          [
            { publicKey: airnode1.publicKey.toBytes(), message: message1, signature: Buffer.allocUnsafe(sig2.length) },
            { publicKey: airnode2.publicKey.toBytes(), message: message2, signature: sig2 }
          ],
          messageRelayer
        );
        expect(false, "should not be here");
      } catch (_) {
        expect(true, "should fail");
      }
    });
  });

  describe("setName", () => {
    it("works", async () => {
      const beaconId = dapiClient.deriveBeaconId(airnode3.publicKey.toBytes(), templateId3);

      await dapiClient.setName(name, beaconId, provider.wallet.publicKey);
  
      await delay(1000);
  
      const datapointId = await dapiClient.nameToDataPointId(name);
      expect([...beaconId]).to.deep.eq([...datapointId]);

      const datapoint = await dapiClient.readWithName(name);
      expect(datapoint.timestamp).to.eq(timestamp3);
      expect(datapoint.value).to.eq(data3);
    });
  });

  describe("deriveDApiId", () => {
    it("works", async () => {
      const publicKey1 = [
        6,  78, 207,   9,  71, 108, 155, 104,
        161, 183, 128,  28, 210, 228,  71, 204,
        102, 174, 178, 254, 233,  61,  63,  16,
        76, 210,  51, 189,  74,  91,  76, 129
      ];
      const publicKey2 = [
        5,  78, 207,   9,  71, 108, 155, 104,
        3, 183, 4,  28, 12, 33,  71, 204,
        102, 174, 178, 53, 25,  61,  63,  16,
        76, 210,  51, 189,  74,  91,  76, 129
      ];
      const beaconId1 = dapiClient.deriveBeaconId(Buffer.from(publicKey1), 1);
      const beaconId2 = dapiClient.deriveBeaconId(Buffer.from(publicKey2), 1);
      const dapiId = dapiClient.deriveDapiId([beaconId1, beaconId2]);
      const expected = [
        110,  19,  32, 193, 151,  76, 132,
        243, 209, 249, 115,  27, 118, 162,
         62, 179, 168, 118, 111, 106, 159,
        213, 112, 120, 131, 222, 247,  10,
         29,  52, 167,  73
      ];
      expect([...dapiId]).to.deep.eq(expected);
    });
  });

  describe("deriveBeaconId", () => {
    it("works", async () => {
      const publicKey = [
        6,  78, 207,   9,  71, 108, 155, 104,
        161, 183, 128,  28, 210, 228,  71, 204,
        102, 174, 178, 254, 233,  61,  63,  16,
        76, 210,  51, 189,  74,  91,  76, 129
      ];
      const beaconId = dapiClient.deriveBeaconId(Buffer.from(publicKey), 1);
      const expected = [
        29, 240, 119, 252, 172, 145, 146, 209,
       118, 116, 187,   7,  80, 114, 122,  33,
        67, 238, 197, 149,  19, 223, 191, 129,
        96,  40, 222,   6, 132,  43,  31, 189
      ];
      expect([...beaconId]).to.deep.eq(expected);
    });
  });
});
