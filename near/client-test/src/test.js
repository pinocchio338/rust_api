const { DapiServer } = require("./client");
const { 
  updateBeacon, dataNotFresherThanBeacon, dataLengthNotCorrect,
  timestampNotValid, signatureNotValid
} = require("./tests/updateBeaconWithSignedData");
const { 
  updatesBeaconSet, updatedValueOutdated, lessThanTwoBeacons

} = require("./tests/updateBeaconSetWithBeacons");
const { 
  senderNotNameSetter, setsDAPIName, dAPINameZero
} = require("./tests/setName");
const { 
  derivesBeaconId, templateIdZero, airnodeZero, derivesBeaconSetId,
} = require("./tests/derive");
const { generateRandomBytes32, bufferU64BE, toBuffer, currentTimestamp, deriveBeaconId, deriveDApiId } = require("./util");
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
const crossContractAccount = "cross-call.testnet";
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
  let crossContract;

  const templateId = generateRandomBytes32();
  const beaconSetTemplateIds = [
    generateRandomBytes32(),
    generateRandomBytes32(),
    generateRandomBytes32(),
  ];

  let beaconSetId;

  // define all the data
  const templateId1 = 1;  
  const data1 = 121;

  const templateId2 = 2;
  const data2 = 122;

  const templateId3 = 3;
  const data3 = 123;

  let client;

  let keyPair;
  beforeAll(async function () {
    near = await nearAPI.connect(config);
    contract = await near.loadContract(contractAccount, {
      viewMethods: [
        'has_role',
        'read_with_data_point_id',
        'roles',
        'name_to_data_point_id',
        'derive_beacon_id',
        'derive_beacon_set_id'
      ],
      changeMethods: [
        'initialize',
        'grant_role',
        'renounce_role',
        'update_beacon_with_signed_data',
        'update_dapi_with_beacons',
        'update_dapi_with_signed_data',
        'get_data_point',
        'set_name',
      ],
      sender: adminAccount
    });
    client = new DapiServer(contract);

    crossContract = await near.loadContract(crossContractAccount, {
      viewMethods: ['hello_world', 'my_callback'],
      changeMethods: ['get_datapoint'],
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

    beaconSetId = deriveDApiId(beaconSetTemplateIds.map(r => deriveBeaconId(keyPair.getPublicKey().data, r)));

    if (!isInitialized) {
      await contract.initialize(
        {
          args: { }
        }
      );
    }
  });

  describe('Access', function () {
    // it('has role', async function () {
    //   const newKey = KeyPair.fromRandom("ed25519");
    //   const newPubKey = toBuffer(newKey.getPublicKey().data);
    //   const r = await contract.has_role(
    //     {
    //       role: [...bufferU64BE(0)],
    //       who: [...Buffer.concat([Buffer.from(adminAccount, 'ascii')], 32)]
    //     }
    //   );
    //   console.log("what's r", r);
    //   expect(r).toEqual(false);

    //   expect(
    //     await contract.has_role(
    //       {
    //         role: [...bufferU64BE(0)],
    //         who: [...newPubKey]
    //       }
    //     )
    //   ).toEqual(false);
    // });

  
    // it('renounce role', async function () {
    //   const roles = await contract.roles();
    //   const readerRole = roles[0];

    //   console.log("reader role", readerRole);
    //   const who = [...Buffer.concat([Buffer.from(crossContractAccount, 'ascii')], 32)]
      
    //   expect(await contract.has_role({role: readerRole, who})).toEqual(true);

    //   await contract.renounce_role({
    //     args: {
    //       role: readerRole,
    //       who
    //     }
    //   });

    //   expect(await contract.has_role({role: readerRole, who})).toEqual(false);
    // });

  });

  describe('updateBeaconWithSignedData', function () {
    // it('updateBeacon', async function () {
    //   const timestamp = currentTimestamp() + 1;
    //   await updateBeacon(client, keyPair, keyPair.getPublicKey().data, templateId, 123, timestamp);
    // });

    // it('dataNotFresherThanBeacon', async function () {
    //   await dataNotFresherThanBeacon(client, keyPair, keyPair.getPublicKey().data, templateId);
    // });

    // it('dataLengthNotCorrect', async function () {
    //   await dataLengthNotCorrect(client, keyPair, keyPair.getPublicKey().data, templateId);
    // });

    // it('timestampNotValid', async function () {
    //   await timestampNotValid(client, keyPair, keyPair.getPublicKey().data, templateId);
    // });

    // it('signatureNotValid', async function () {
    //   await signatureNotValid(client, keyPair, keyPair.getPublicKey().data, templateId);
    // });
  });

  describe('updateBeaconSetWithBeacons', function () {
    // it('updatesBeaconSet', async function () {
    //   const beaconIds = [];
    //   const beaconData = [123, 456, 789];
    //   let timestamp = currentTimestamp();
    //   for (let ind = 0; ind < beaconData.length; ind++) {
    //     timestamp++;
    //     // await updateBeacon(client, keyPair, keyPair.getPublicKey().data, beaconSetTemplateIds[ind], beaconData[ind], timestamp);
    //     beaconIds.push([...deriveBeaconId(keyPair.getPublicKey().data, beaconSetTemplateIds[ind])]);
    //   }
    //   await updatesBeaconSet(client, beaconIds);
    // });

    // it('lessThanTwoBeacons', async function () {
    //   await lessThanTwoBeacons(client);
    // });
  });

  describe('updateDapiWithSignedData', function () {
  });
  
  describe('setName', function () {
    // it('setsDAPIName', async function () {
    //   const roles = await contract.roles();
    //   await client.grantRole(
    //     roles[1],
    //     [...Buffer.concat([Buffer.from(adminAccount, 'ascii')], 32)]
    //   );
    //   const dapiName = Buffer.from(ethers.utils.formatBytes32String('My dAPI').substring(2), "hex");
    //   await setsDAPIName(client, dapiName, beaconSetId);
    // });

    // it('senderNotNameSetter', async function () {
    //   const roles = await contract.roles();
    //   await client.renounceRole(
    //     roles[1],
    //     [...Buffer.concat([Buffer.from(adminAccount, 'ascii')], 32)]
    //   );
    //   const dapiName = Buffer.from(ethers.utils.formatBytes32String('My dAPI').substring(2), "hex");
    //   await senderNotNameSetter(client, dapiName, beaconSetId);
    // });

    // it('dAPINameZero', async function () {
    //   await dAPINameZero(client);
    // });
  });

  describe('deriveBeaconId', function () {
    // it('derivesBeaconId', async function () {
    //   await derivesBeaconId(client, keyPair.getPublicKey().data, templateId);
    // });

    // it('templateIdZero', async function () {
    //   await templateIdZero(client, keyPair.getPublicKey().data);
    // });

    // it('airnodeZero', async function () {
    //   await airnodeZero(client, templateId);
    // });
  });

  describe('deriveBeaconSetId', function () {
    // it('derivesBeaconSetId', async function () {
    //   await derivesBeaconSetId(client, [[...generateRandomBytes32()], [...generateRandomBytes32()]]);
    // });
  });

  // describe('readWithDataPointId - cross contract call', function () {
  //   it('works', async function () {
  //     const pubKeyBuf = toBuffer(keyPair.getPublicKey().data);
  //     const beaconId1 = deriveBeaconId(pubKeyBuf, templateId1);
  //     const beaconId2 = deriveBeaconId(pubKeyBuf, templateId2);
  //     const beaconId3 = deriveBeaconId(pubKeyBuf, templateId3);
  //     const beaconIds = [beaconId1, beaconId2, beaconId3];
  //     const dataPointId = deriveDApiId(beaconIds);
  //     await crossContract.get_datapoint(
  //       {
  //         args: {
  //           datapoint_id: [...dataPointId]
  //         }
  //       }
  //     );
  //   });
  // });
});
