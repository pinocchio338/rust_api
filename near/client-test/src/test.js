const fs = require("fs");
const ethers = require("ethers");
const nearAPI = require("near-api-js");
const { connect, KeyPair, keyStores, providers } = require("near-api-js");
const path = require("path");
const { base64 } = require("ethers/lib/utils");
const { assert } = require("console");
const homedir = require("os").homedir();
const CREDENTIALS_DIR = ".near-credentials";
const credentialsPath = path.join(homedir, CREDENTIALS_DIR);
const keyStore = new keyStores.UnencryptedFileSystemKeyStore(credentialsPath);

// DEFINE THESE
const adminAccount = "mocha-test.testnet";
const userAccount = "user-test.testnet";
const contractAccount = "test-api3.testnet";
const isInitialized = true;

const config = {
  keyStore,
  networkId: "testnet",
  nodeUrl: "https://rpc.testnet.near.org",
};

describe('Token', function () {
  let contract;
  let userContract;
  let near;

  // define all the data
  const templateId1 = 1;  
  const data1 = 121;

  const templateId2 = 2;
  const data2 = 122;

  const templateId3 = 3;
  const data3 = 123;

  let keyPair;
  beforeAll(async function () {
    near = await nearAPI.connect(config);
    contract = await near.loadContract(contractAccount, {
      viewMethods: ['has_role', 'read_with_data_point_id'],
      changeMethods: [
        'initialize',
        'grant_role',
        'update_beacon_with_signed_data',
        'update_dapi_with_beacons',
        'update_dapi_with_signed_data',
        'get_data_point'
      ],
      sender: adminAccount
    });

    userContract = await near.loadContract(contractAccount, {
      viewMethods: [],
      changeMethods: ['get_data_point'],
      sender: userAccount
    });

    const account = await near.account(adminAccount);
    const key = `${account.connection.signer.keyStore.keyDir}/testnet/${account.accountId}.json`;
    const data = JSON.parse(fs.readFileSync(key));
    keyPair = nearAPI.KeyPair.fromString(data.private_key);

    if (!isInitialized) {
      await contract.initialize(
        {
          args: { }
        }
      );
    }
  });

  describe('Access', function () {
    it('has role', async function () {
      const pubKeyBuf = toBuffer(keyPair.getPublicKey().data);
      const newKey = KeyPair.fromRandom("ed25519");
      const newPubKey = toBuffer(newKey.getPublicKey().data);
      // await contract.grant_role(
      //   {
      //     args: {

      //     }
      //   }
      // );
      const r = await contract.has_role(
        {
          role: [...bufferU64BE(0)],
          who: [...pubKeyBuf]
        }
      );
      expect(r).toEqual(true);

      expect(
        await contract.has_role(
          {
            role: [...bufferU64BE(0)],
            who: [...newPubKey]
          }
        )
      ).toEqual(false);
    });
  });

  describe('updateBeaconWithSignedData', function () {
    it('works', async function () {

      const timestamp = Math.floor(Date.now() / 1000);
      const message1 = prepareMessage(templateId1, timestamp, data1);
      const sig1 = keyPair.sign(message1);

      const pubKeyBuf = toBuffer(keyPair.getPublicKey().data);
      const bufferedTimestamp = bufferU64BE(timestamp);
      const bufferedTemplateId = bufferU64BE(templateId1);
      const encodedData = encodeData(data1);
      const buf = toBuffer(sig1.signature);

      await contract.update_beacon_with_signed_data(
        {
          args: {
            airnode: [...pubKeyBuf],
            template_id: [...bufferedTemplateId],
            timestamp: [...bufferedTimestamp],
            data: [...encodedData],
            signature: [...buf]
          }
        }
      );

      let data = await contract.read_with_data_point_id(
        {
          data_point_id: [...deriveBeaconId(pubKeyBuf, templateId1)]
        }
      );
      expect(data[0]).toEqual([...encodedData]);
      expect(data[1]).toEqual(timestamp);

      const timestamp2 = Math.floor(Date.now() / 1000);
      const message2 = prepareMessage(templateId2, timestamp2, data2);
      const sig2 = keyPair.sign(message2);

      const pubKeyBuf2 = toBuffer(keyPair.getPublicKey().data);
      const bufferedTimestamp2 = bufferU64BE(timestamp2);
      const bufferedTemplateId2 = bufferU64BE(templateId2);
      const encodedData2 = encodeData(data2);
      const buf2 = toBuffer(sig2.signature);

      await contract.update_beacon_with_signed_data(
        {
          args: {
            airnode: [...pubKeyBuf2],
            template_id: [...bufferedTemplateId2],
            timestamp: [...bufferedTimestamp2],
            data: [...encodedData2],
            signature: [...buf2]
          }
        }
      );

      data = await contract.read_with_data_point_id(
        {
          data_point_id: [...deriveBeaconId(pubKeyBuf2, templateId2)]
        }
      );
      expect(data[0]).toEqual([...encodedData2]);
      expect(data[1]).toEqual(timestamp2);
    });
  });

  describe('updateDapiWithBeacons', function () {
    it('works', async function () {
      await contract.update_dapi_with_beacons(
        {
          args: {
            beacon_ids: [
              [...deriveBeaconId(toBuffer(keyPair.getPublicKey().data), templateId1)],
              [...deriveBeaconId(toBuffer(keyPair.getPublicKey().data), templateId2)]
            ]
          }
        }
      );
    });
  });

  describe('updateDapiWithSignedData', function () {
    it('works', async function () {
      const timestamp = Math.floor(Date.now() / 1000);
      const message3 = prepareMessage(templateId3, timestamp, data3);
      const sig3 = keyPair.sign(message3);

      const pubKeyBuf = toBuffer(keyPair.getPublicKey().data);
      const t1 = Math.floor(Date.now() / 1000);
      const t2 = Math.floor(Date.now() / 1000);
      await contract.update_dapi_with_signed_data(
        {
          args: {
            airnodes: [
              [...pubKeyBuf],
              [...pubKeyBuf],
              [...pubKeyBuf]
            ],
            template_ids: [
              [...bufferU64BE(templateId1)],
              [...bufferU64BE(templateId2)],
              [...bufferU64BE(templateId3)]
            ],
            timestamps: [
              [...bufferU64BE(t1)],
              [...bufferU64BE(t2)],
              [...bufferU64BE(timestamp)],
            ],
            data: [
              [...encodeData(data1)],
              [...encodeData(data2)],
              [...encodeData(data3)]
            ],
            signatures: [
              [],
              [],
              [...toBuffer(sig3.signature)]
            ],
          }
        }
      );

      const beaconId1 = deriveBeaconId(pubKeyBuf, templateId1);
      const beaconId2 = deriveBeaconId(pubKeyBuf, templateId2);
      const beaconId3 = deriveBeaconId(pubKeyBuf, templateId3);
      const beaconIds = [beaconId1, beaconId2, beaconId3];
      const dataPointId = deriveDApiId(beaconIds);
      let data = await contract.read_with_data_point_id(
        {
          data_point_id: [...dataPointId]
        }
      );
      expect(data[0]).toEqual([...bufferU64BE((data1 + data2 + data3) / 3)]);
    });
  });

  // describe('setName', function () {
  //   it('works', async function () {
  //     const timestamp = Math.floor(Date.now() / 1000);
  //     const message3 = prepareMessage(templateId3, timestamp, data3);
  //     const sig3 = keyPair.sign(message3);

  //     const pubKeyBuf = toBuffer(keyPair.getPublicKey().data);
  //     await contract.read_with_data_point_id(
  //       {
  //         data_point_id: []
  //       }
  //     );
  //   });
  // });
});

function createRawDatapointBuffer(data, timestamp) {
    const expected = Buffer.allocUnsafe(36);
    expected.writeBigInt64BE(BigInt(0), 0);
    expected.writeBigInt64BE(BigInt(0), 8);
    expected.writeBigInt64BE(BigInt(0), 16);
    expected.writeBigInt64BE(BigInt(data), 24);
    expected.writeUInt32BE(timestamp, 32);
    return expected;
}

function prepareMessage(
    templateId,
    timestamp,
    data,
) {
    const bufferedTemplate = bufferU64BE(templateId);
    const bufferedTimestamp = bufferU64BE(timestamp);
    const encodedData = encodeData(data);
    return keccak256Packed(
        ["bytes32", "uint256", "bytes"],
        [bufferedTemplate, bufferedTimestamp, encodedData]
    )
}

function keccak256Packed(types, data) {
    let hex = ethers.utils.solidityPack(types, data).substr(2); // remove starting "0x"
    const buf = Buffer.from(hex, "hex");
    hex = ethers.utils.keccak256(buf).substr(2); // remove starting "0x"
    return Buffer.from(hex, "hex");
}

function deriveBeaconId(airnodeKey, templateId) {
    const bufferedTemplate = bufferU64BE(templateId);
    return keccak256Packed(["bytes", "bytes32"], [airnodeKey, bufferedTemplate]);
}

function deriveDApiId(beaconIds) {
    const types = beaconIds.map(_ => "bytes32");
    return keccak256Packed(types, beaconIds);
}

function encodeData(decodedData) {
    const hex = ethers.utils.defaultAbiCoder.encode(['int256'], [decodedData]);
    return Buffer.from(hex.substr(2), "hex");
}

function bufferU64BE(value) {
    const buffer = Buffer.alloc(32);
    buffer.writeBigUInt64BE(BigInt(value), 24);
    return buffer;
}

function toBuffer(ab) {
  const buf = Buffer.alloc(ab.byteLength);
  const view = new Uint8Array(ab);
  for (let i = 0; i < buf.length; ++i) {
      buf[i] = view[i];
  }
  return buf;
}