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
const { updatesBeaconSetWithSignedData, updatedSetValueOutdated, lengthNotCorrect, notAllSignaturesValid, notAllTimestampValid, parameterLengthMismatch } = require("./tests/updateBeaconSetWithSignedData");
const { readerZeroAddress, readerNotWhitelisted, readerWhitelisted, readerUnlimitedReaderRole } = require("./tests/readerCanReadDataFeed");
const { dataFeedIdToReaderToWhitelistStatus, dataFeedIdToReaderToSetterToIndefiniteWhitelistStatus } = require("./tests/whitelist");
const { revokeRole, renounceRole } = require("./tests/role");
const homedir = require("os").homedir();
const CREDENTIALS_DIR = ".near-credentials";
const credentialsPath = path.join(homedir, CREDENTIALS_DIR);
const keyStore = new keyStores.UnencryptedFileSystemKeyStore(credentialsPath);

const contractAccount = process.env.CONTRACT_ACCOUNT;
const adminAccount = process.env.ADMIN_ACCOUNT;
const userAccount = process.env.USER_ACCOUNT;
const crossContractAccount = process.env.CROSS_CONTRACT_ACCOUNT;
// If you are running the first time, ensure this is false
// else make this true
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
        'derive_beacon_set_id',
        'reader_can_read_data_point',
        'data_feed_id_to_whitelist_status',
        'data_feed_id_to_reader_to_setter_to_indefinite_whitelist_status',
      ],
      changeMethods: [
        'initialize',
        'grant_role',
        'renounce_role',
        'revoke_role',
        'update_beacon_with_signed_data',
        'update_dapi_with_beacons',
        'update_dapi_with_signed_data',
        'get_data_point',
        'set_name',
        'set_indefinite_whitelist_status',
        'set_whitelist_expiration',
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

  describe('updateBeaconSetWithSignedData', function () {
    // it('updatesBeaconSetWithSignedData', async function () {
    //   await updatesBeaconSetWithSignedData(client, keyPair, keyPair.getPublicKey().data, beaconSetTemplateIds);
    // });

    // it('updatedSetValueOutdated', async function () {
    //   await updatedSetValueOutdated(client, keyPair, keyPair.getPublicKey().data, beaconSetTemplateIds);
    // });

    // it('dataValueExceedingRange', async function () {
    //   // TODO: we are using U256 internally, not sure if this is still needed  
    // });

    // it('lengthNotCorrect', async function () {
    //   // TODO: debugging
    //   // await lengthNotCorrect(client, keyPair, keyPair.getPublicKey().data, beaconSetTemplateIds);
    // });

    // it('notAllSignaturesValid', async function () {
    //   await notAllSignaturesValid(client, keyPair, keyPair.getPublicKey().data, beaconSetTemplateIds);
    // });

    // it('notAllTimestampValid', async function () {
    //   // TODO
    //   await notAllTimestampValid(client, keyPair, keyPair.getPublicKey().data, beaconSetTemplateIds);
    // });

    // it('lessThanTwoBeacons', async function () {
    //   await lessThanTwoBeacons(client);
    // });

    // it('parameterLengthMismatch', async function () {
    //   await parameterLengthMismatch(client, keyPair.getPublicKey().data, beaconSetTemplateIds);
    // });
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
    //   await client.revokeRole(
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

  describe('readerCanReadDataFeed', function () {
    // it('readerZeroAddress', async function () {
    //   await readerZeroAddress(client);
    // });

    // it('readerNotWhitelisted', async function () {
    //   await readerNotWhitelisted(client, keyPair.getPublicKey().data);
    // });

    // it('readerWhitelisted', async function () {
    //   await readerWhitelisted(client, generateRandomBytes32(), keyPair.getPublicKey().data);
    // });

    // it('readerUnlimitedReaderRole', async function () {
    //   const roles = await contract.roles();
    //   await readerUnlimitedReaderRole(client, keyPair.getPublicKey().data, roles[0]);
    // });
  });

  describe('role', function () {
    // it('revokeRole', async function () {
    //   await revokeRole(client, [...generateRandomBytes32()]);
    // });

    it('renounceRole', async function () {
      await renounceRole(client, [...generateRandomBytes32()], [...Buffer.concat([Buffer.from(adminAccount, 'ascii')], 32)]);
    });
    // it('dataFeedIdToReaderToSetterToIndefiniteWhitelistStatus', async function () {
    //   await dataFeedIdToReaderToSetterToIndefiniteWhitelistStatus(client, [...Buffer.concat([Buffer.from(adminAccount, 'ascii')], 32)]);
    // });
  });

  describe('whitelist', function () {
    // it('dataFeedIdToReaderToWhitelistStatus', async function () {
    //   await dataFeedIdToReaderToWhitelistStatus(client);
    // });

    // it('dataFeedIdToReaderToSetterToIndefiniteWhitelistStatus', async function () {
    //   await dataFeedIdToReaderToSetterToIndefiniteWhitelistStatus(client, [...Buffer.concat([Buffer.from(adminAccount, 'ascii')], 32)]);
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
