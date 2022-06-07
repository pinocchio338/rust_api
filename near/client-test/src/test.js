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
const { generateRandomBytes32, bufferU64BE, toBuffer, currentTimestamp, deriveBeaconId, deriveDApiId, delay } = require("./util");
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
const { WithExtenderRole } = require("./tests/extendWhitelistExpiration");
const { WithSetterRole } = require("./tests/setWhitelistExpiration");
const { WithIndefiniteWhitelisterSetterRole } = require("./tests/setIndefiniteWhitelistStatus");
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

  // This is the actual dapi client
  let client;
  // This is just a test util contract with propoer reading access, for data checking purposes
  let userClient;

  let keyPair;
  beforeAll(async function () {
    near = await nearAPI.connect(config);
    contract = await near.loadContract(contractAccount, {
      viewMethods: [
        'has_role',
        // 'read_with_data_point_id',
        'roles',
        'name_to_data_point_id',
        'derive_beacon_id',
        'derive_beacon_set_id',
        'reader_can_read_data_point',
        'data_feed_id_to_whitelist_status',
        'data_feed_id_to_reader_to_setter_to_indefinite_whitelist_status',
        'whitelist_expiration_setter_role',
        'whitelist_expiration_extender_role',
        'indefinite_whitelister_role'
      ],
      changeMethods: [
        'read_with_data_point_id',
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
        'extend_whitelist_expiration',
      ],
      sender: adminAccount
    });
    client = new DapiServer(contract);

    userContract = await near.loadContract(contractAccount, {
      viewMethods: [
        'roles',
        'name_to_data_point_id',
        'derive_beacon_id',
        'derive_beacon_set_id',
        'reader_can_read_data_point',
        'data_feed_id_to_whitelist_status',
        'data_feed_id_to_reader_to_setter_to_indefinite_whitelist_status',
        'whitelist_expiration_setter_role',
        'whitelist_expiration_extender_role',
        'indefinite_whitelister_role'
      ],
      changeMethods: [
        'read_with_data_point_id',
        'set_indefinite_whitelist_status',
        'set_whitelist_expiration',
        'extend_whitelist_expiration',
      ],
      sender: userAccount
    });
    userClient = new DapiServer(userContract);

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
      console.log("initialized contract");
      // just wait a bit for the effects to take place
      await delay(1000);

      const reader = [...Buffer.concat([Buffer.from(userAccount, 'ascii')], 32)];
      const unlimitedReaderRole = (await contract.roles())[0];
      await client.grantRole([...unlimitedReaderRole], reader);
      await delay(1000);
    }
  });

  describe('updateBeaconWithSignedData', function () {
    // it('updateBeacon', async function () {
    //   const timestamp = currentTimestamp() + 1;
    //   await updateBeacon(client, keyPair, keyPair.getPublicKey().data, templateId, 123, timestamp, userClient);
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

    // it('renounceRole', async function () {
    //   await renounceRole(client, [...generateRandomBytes32()], [...Buffer.concat([Buffer.from(adminAccount, 'ascii')], 32)]);
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

  describe('extendWhitelistExpiration', function () {
    // describe('Sender has whitelist expiration extender role', function () {
    //   beforeAll(async function () {
    //     await WithExtenderRole.setup(client, [...Buffer.concat([Buffer.from(userAccount, 'ascii')], 32)], userClient);
    //     console.log("setup done for extendWhitelistExpiration");
    //   });

    //   it('extendsWhitelistExpiration', async function () {
    //     await WithExtenderRole.extendsWhitelistExpiration(userClient);
    //   });

    //   it('doesNotExtendExpiration', async function () {
    //     await WithExtenderRole.doesNotExtendExpiration(userClient);
    //   });

    //   it('readerZeroAddress', async function () {
    //     await WithExtenderRole.readerZeroAddress(userClient);
    //   });

    //   it('dataFeedIdZero', async function () {
    //     await WithExtenderRole.dataFeedIdZero(userClient);
    //   });

    //   afterAll(async function () {
    //     await WithExtenderRole.tearDown(client, [...Buffer.concat([Buffer.from(userAccount, 'ascii')], 32)]);
    //     console.log("tear down done for extendWhitelistExpiration");
    //   });
    // });

    // describe('Sender is the manager', function () {
    //   it('extendsWhitelistExpiration', async function () {
    //     await WithExtenderRole.extendsWhitelistExpiration(client);
    //   });

    //   it('doesNotExtendExpiration', async function () {
    //     await WithExtenderRole.doesNotExtendExpiration(client);
    //   });

    //   it('readerZeroAddress', async function () {
    //     await WithExtenderRole.readerZeroAddress(client);
    //   });

    //   it('dataFeedIdZero', async function () {
    //     await WithExtenderRole.dataFeedIdZero(client);
    //   });
    // });
  });

  describe('setWhitelistExpiration', function () {
    describe('Sender has whitelist expiration setter role', function () {
      // beforeAll(async function () {
      //   await WithSetterRole.setup(client, [...Buffer.concat([Buffer.from(userAccount, 'ascii')], 32)], userClient);
      //   console.log("setup done for setWhitelistExpiration");
      // });

      // it('setsWhitelistExpiration', async function () {
      //   await WithSetterRole.setsWhitelistExpiration(userClient);
      // });

      // it('readerZeroAddress', async function () {
      //   await WithSetterRole.readerZeroAddress(userClient);
      // });

      // it('dataFeedIdZero', async function () {
      //   await WithSetterRole.dataFeedIdZero(userClient);
      // });

      // afterAll(async function () {
      //   await WithSetterRole.tearDown(client, [...Buffer.concat([Buffer.from(userAccount, 'ascii')], 32)]);
      //   console.log("tear down done for extendWhitelistExpiration");
      // });
    });

    describe('Sender is the manager', function () {
      // it('setsWhitelistExpiration', async function () {
      //   await WithSetterRole.setsWhitelistExpiration(client);
      // });

      // it('readerZeroAddress', async function () {
      //   await WithSetterRole.readerZeroAddress(client);
      // });

      // it('dataFeedIdZero', async function () {
      //   await WithSetterRole.dataFeedIdZero(client);
      // });
    });
  });

  describe('setIndefiniteWhitelistStatus', function () {
    describe('Sender has whitelist expiration setter role', function () {
      beforeAll(async function () {
        await WithIndefiniteWhitelisterSetterRole.setup(client, [...Buffer.concat([Buffer.from(userAccount, 'ascii')], 32)], userClient);
        console.log("setup done for setIndefiniteWhitelistStatus");
      });

      it('setIndefiniteWhitelistStatus', async function () {
        await WithIndefiniteWhitelisterSetterRole.setIndefiniteWhitelistStatus(
          userClient,
          [...Buffer.concat([Buffer.from(userAccount, 'ascii')], 32)]
        );
      });

      it('readerZeroAddress', async function () {
        await WithIndefiniteWhitelisterSetterRole.readerZeroAddress(userClient);
      });

      it('dataFeedIdZero', async function () {
        await WithIndefiniteWhitelisterSetterRole.dataFeedIdZero(userClient);
      });

      afterAll(async function () {
        await WithIndefiniteWhitelisterSetterRole.tearDown(client, [...Buffer.concat([Buffer.from(userAccount, 'ascii')], 32)]);
        console.log("tear down done for extendWhitelistExpiration");
      });
    });

    describe('Sender is the manager', function () {
      it('setIndefiniteWhitelistStatus', async function () {
        await WithIndefiniteWhitelisterSetterRole.setIndefiniteWhitelistStatus(
          client, [...Buffer.concat([Buffer.from(adminAccount, 'ascii')], 32)]
        );
      });

      it('readerZeroAddress', async function () {
        await WithIndefiniteWhitelisterSetterRole.readerZeroAddress(client);
      });

      it('dataFeedIdZero', async function () {
        await WithIndefiniteWhitelisterSetterRole.dataFeedIdZero(client);
      });
    });
  });
});
