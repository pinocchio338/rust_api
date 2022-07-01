const { DapiServer } = require("../src/client");
const { 
  updateBeacon, dataNotFresherThanBeacon, dataLengthNotCorrect,
  timestampNotValid, signatureNotValid
} = require("./utils/updateBeaconWithSignedData");
const { 
  updatesBeaconSet, lessThanTwoBeacons

} = require("./utils/updateBeaconSetWithBeacons");
const { 
  senderNotNameSetter, setsDAPIName, dAPINameZero
} = require("./utils/setName");
const {
  derivesBeaconId, templateIdZero, airnodeZero, derivesBeaconSetId,
} = require("./utils/derive");
const { generateRandomBytes32, toBuffer, currentTimestamp, deriveBeaconId, deriveDApiId, delay, encodeAndSignData, encodeData } = require("../src/util");
const fs = require("fs");
const ethers = require("ethers");
const nearAPI = require("near-api-js");
const { keyStores } = require("near-api-js");
const path = require("path");
const { updatesBeaconSetWithSignedData, updatedSetValueOutdated, lengthNotCorrect, notAllSignaturesValid, parameterLengthMismatch } = require("./utils/updateBeaconSetWithSignedData");
const { readerZeroAddress, readerNotWhitelisted, readerWhitelisted, readerUnlimitedReaderRole } = require("./utils/readerCanReadDataFeed");
const { dataFeedIdToReaderToWhitelistStatus, dataFeedIdToReaderToSetterToIndefiniteWhitelistStatus } = require("./utils/whitelist");
const { revokeRole, renounceRole } = require("./utils/role");
const { WithExtenderRole } = require("./utils/extendWhitelistExpiration");
const { WithSetterRole } = require("./utils/setWhitelistExpiration");
const { WithIndefiniteWhitelisterSetterRole } = require("./utils/setIndefiniteWhitelistStatus");
const { revokesIndefiniteWhitelistStatus, setterHasIndefiniteWhitelisterRole } = require("./utils/revokeIndefiniteWhitelistStatus");
const { readerNotPermitted, readerUnlimitedReaderReads, readerWhitelistedReads } = require("./utils/readDataFeedWithId");
const { readerWhitelistedReadsByName, unlimitedReaderReadsWithName, readerNotPermittedWithName } = require("./utils/readDataFeedWithDapiName");
const homedir = require("os").homedir();
const CREDENTIALS_DIR = ".near-credentials";
const credentialsPath = path.join(homedir, CREDENTIALS_DIR);
const keyStore = new keyStores.UnencryptedFileSystemKeyStore(credentialsPath);

const contractAccount = process.env.CONTRACT_ACCOUNT;
const adminAccount = process.env.ADMIN_ACCOUNT;
const userAccount = process.env.USER_ACCOUNT;

// If you are running the first time, ensure this is false
// else make this true
const isInitialized = true;

const config = {
  keyStore,
  networkId: "testnet",
  nodeUrl: "https://rpc.testnet.near.org",
};

// Network can get unpredictable, set timeout to a large value just in case
jest.setTimeout(120_000);

describe('Token', function () {
  let contract;
  let userContract;
  let near;

  const templateId = generateRandomBytes32();
  const beaconSetTemplateIds = [
    generateRandomBytes32(),
    generateRandomBytes32(),
    generateRandomBytes32(),
  ];

  let beaconSetId;

  // This is the actual dapi client
  let client;
  // This is just a test util contract with propoer reading access, for data checking purposes
  let userClient;

  let keyPair;
  beforeAll(async function () {
    near = await nearAPI.connect(config);
    const admin = await near.account(adminAccount);
    contract = new nearAPI.Contract(admin, contractAccount, {
      viewMethods: [
        'has_role',
        'roles',
        'name_to_data_point_id',
        'derive_beacon_id',
        'derive_beacon_set_id',
        'reader_can_read_data_point',
        'data_feed_id_to_whitelist_status',
        'data_feed_id_to_reader_to_setter_to_indefinite_whitelist_status',
        'whitelist_expiration_setter_role',
        'whitelist_expiration_extender_role',
        'indefinite_whitelister_role',
        'read_with_data_point_id',
        'read_with_name',
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
        'extend_whitelist_expiration',
        'revoke_indefinite_whitelist_status',
      ],
    });
    client = new DapiServer(contract);
    const user = await near.account(userAccount);
    userContract = new nearAPI.Contract(user, contractAccount, {
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
        'get_data_point',
        'read_with_data_point_id',
        'read_with_name',
        'set_indefinite_whitelist_status',
        'set_whitelist_expiration',
        'extend_whitelist_expiration',
        'revoke_indefinite_whitelist_status'
      ],
    });
    userClient = new DapiServer(userContract);

    const key = `${admin.connection.signer.keyStore.keyDir}/testnet/${admin.accountId}.json`;
    const data = JSON.parse(fs.readFileSync(key));
    keyPair = nearAPI.KeyPair.fromString(data.private_key);

    beaconSetId = deriveDApiId(beaconSetTemplateIds.map(r => deriveBeaconId(keyPair.getPublicKey().data, r)));

    if (!isInitialized) {
      await contract.initialize(
        {
          args: { }
        }
      );

      const reader = userAccount;
      const unlimitedReaderRole = (await contract.roles())[0];
      await client.grantRole([...unlimitedReaderRole], reader);
    }
  });

  describe('updateBeaconWithSignedData', function () {
    let roles;

    beforeAll(async () => {
      roles = await contract.roles();
      await client.grantRole(
        roles[0],
        userAccount
      );
    });

    afterAll(async () => {
      await client.revokeRole(
        roles[0],
        userAccount
      );
    })

    it('updateBeacon', async function () {
      const timestamp = currentTimestamp() + 1;
      const beacon = await updateBeacon(client, keyPair, keyPair.getPublicKey().data, templateId, 123, timestamp, userClient);
      ensure(
        array_equals(beacon.value, [...encodeData(value)])
      );
      ensure(beacon.timestamp === timestamp);
    });

    it('dataNotFresherThanBeacon', async function () {
      await dataNotFresherThanBeacon(client, keyPair, keyPair.getPublicKey().data, templateId);
    });

    it('dataLengthNotCorrect', async function () {
      await dataLengthNotCorrect(client, keyPair, keyPair.getPublicKey().data, templateId);
    });

    it('timestampNotValid', async function () {
      await timestampNotValid(client, keyPair, keyPair.getPublicKey().data, templateId);
    });

    it('signatureNotValid', async function () {
      await signatureNotValid(client, keyPair, keyPair.getPublicKey().data, templateId);
    });
  });

  describe('updateBeaconSetWithBeacons', function () {
    let roles;

    beforeAll(async () => {
      roles = await contract.roles();
      await client.grantRole(
        roles[0],
        userAccount
      );
    });

    afterAll(async () => {
      await client.revokeRole(
        roles[0],
        userAccount
      );
    })

    it('updatesBeaconSet', async function () {
      const beaconIds = [];
      const beaconData = [123, 456, 789];
      let expectedTimestamp = 0;
      for (let ind = 0; ind < beaconData.length; ind++) {
        const timestamp = currentTimestamp() + 1;
        await updateBeacon(
          client,
          keyPair,
          keyPair.getPublicKey().data,
          beaconSetTemplateIds[ind],
          beaconData[ind],
          timestamp,
          userClient
        );
        beaconIds.push([...deriveBeaconId(keyPair.getPublicKey().data, beaconSetTemplateIds[ind])]);
        expectedTimestamp += timestamp;
      }
      await updatesBeaconSet(client, beaconIds, 456, Math.floor(expectedTimestamp / beaconData.length), userClient);
    });

    it('lessThanTwoBeacons', async function () {
      await lessThanTwoBeacons(client);
    });
  });

  describe('updateBeaconSetWithSignedData', function () {
    let roles;

    beforeAll(async () => {
      roles = await contract.roles();
      await client.grantRole(
        roles[0],
        userAccount
      );
    });

    afterAll(async () => {
      await client.revokeRole(
        roles[0],
        userAccount
      );
    });

    it('updatesBeaconSetWithSignedData', async function () {
      await updatesBeaconSetWithSignedData(client, keyPair, keyPair.getPublicKey().data, beaconSetTemplateIds, userClient);
    });

    it('updatedSetValueOutdated', async function () {
      await updatedSetValueOutdated(client, keyPair, keyPair.getPublicKey().data, beaconSetTemplateIds);
    });

    it.skip('dataValueExceedingRange', async function () {
      // TODO: we are using U256 internally, not sure if this is still needed  
    });

    it('lengthNotCorrect', async function () {
      await lengthNotCorrect(client, keyPair, keyPair.getPublicKey().data, beaconSetTemplateIds);
    });

    it('notAllSignaturesValid', async function () {
      await notAllSignaturesValid(client, keyPair, keyPair.getPublicKey().data, beaconSetTemplateIds);
    });

    it('lessThanTwoBeacons', async function () {
      await lessThanTwoBeacons(client);
    });

    it('parameterLengthMismatch', async function () {
      await parameterLengthMismatch(client, keyPair.getPublicKey().data, beaconSetTemplateIds);
    });
  });
  
  describe('setName', function () {
    it('setsDAPIName', async function () {
      const roles = await contract.roles();
      await client.grantRole(
        roles[1],
        adminAccount
      );
      const dapiName = Buffer.from(ethers.utils.formatBytes32String('My dAPI').substring(2), "hex");
      await setsDAPIName(client, dapiName, beaconSetId);
    });

    it('senderNotNameSetter', async function () {
      const roles = await contract.roles();
      await client.revokeRole(
        roles[1],
        adminAccount
      );
      const dapiName = Buffer.from(ethers.utils.formatBytes32String('My dAPI').substring(2), "hex");
      await senderNotNameSetter(client, dapiName, beaconSetId);
    });

    it('dAPINameZero', async function () {
      await dAPINameZero(client);
    });
  });

  describe('deriveBeaconId', function () {
    it('derivesBeaconId', async function () {
      await derivesBeaconId(client, keyPair.getPublicKey().data, templateId);
    });

    it('templateIdZero', async function () {
      await templateIdZero(client, keyPair.getPublicKey().data);
    });

    it('airnodeZero', async function () {
      await airnodeZero(client, templateId);
    });
  });

  describe('deriveBeaconSetId', function () {
    it('derivesBeaconSetId', async function () {
      await derivesBeaconSetId(client, [[...generateRandomBytes32()], [...generateRandomBytes32()]]);
    });
  });

  describe('readerCanReadDataFeed', function () {
    it('readerZeroAddress', async function () {
      await readerZeroAddress(client);
    });

    it('readerNotWhitelisted', async function () {
      await readerNotWhitelisted(client, adminAccount);
    });

    it('readerWhitelisted', async function () {
      await readerWhitelisted(client, generateRandomBytes32(), adminAccount);
    });

    it('readerUnlimitedReaderRole', async function () {
      const roles = await contract.roles();
      await readerUnlimitedReaderRole(client, adminAccount, roles[0]);
    });
  });

  describe('role', function () {
    it('revokeRole', async function () {
      await revokeRole(client, [...generateRandomBytes32()]);
    });

    it('renounceRole', async function () {
      await renounceRole(client, [...generateRandomBytes32()], adminAccount);
    });
  });

  describe('whitelist', function () {
    it('dataFeedIdToReaderToWhitelistStatus', async function () {
      await dataFeedIdToReaderToWhitelistStatus(client);
    });

    it('dataFeedIdToReaderToSetterToIndefiniteWhitelistStatus', async function () {
      await dataFeedIdToReaderToSetterToIndefiniteWhitelistStatus(client, adminAccount);
    });
  });

  describe('extendWhitelistExpiration', function () {
    describe('Sender has whitelist expiration extender role', function () {
      beforeAll(async function () {
        await WithExtenderRole.setup(client, userAccount, userClient);
      });

      it('extendsWhitelistExpiration', async function () {
        await WithExtenderRole.extendsWhitelistExpiration(userClient);
      });

      it('doesNotExtendExpiration', async function () {
        await WithExtenderRole.doesNotExtendExpiration(userClient);
      });

      it('readerZeroAddress', async function () {
        await WithExtenderRole.readerZeroAddress(userClient);
      });

      it('dataFeedIdZero', async function () {
        await WithExtenderRole.dataFeedIdZero(userClient);
      });

      afterAll(async function () {
        await WithExtenderRole.tearDown(client, userAccount);
      });
    });

    describe('Sender is the manager', function () {
      it('extendsWhitelistExpiration', async function () {
        await WithExtenderRole.extendsWhitelistExpiration(client);
      });

      it('doesNotExtendExpiration', async function () {
        await WithExtenderRole.doesNotExtendExpiration(client);
      });

      it('readerZeroAddress', async function () {
        await WithExtenderRole.readerZeroAddress(client);
      });

      it('dataFeedIdZero', async function () {
        await WithExtenderRole.dataFeedIdZero(client);
      });
    });
  });

  describe('setWhitelistExpiration', function () {
    describe('Sender has whitelist expiration setter role', function () {
      beforeAll(async function () {
        await WithSetterRole.setup(client, userAccount, userClient);
      });

      it('setsWhitelistExpiration', async function () {
        await WithSetterRole.setsWhitelistExpiration(userClient);
      });

      it('readerZeroAddress', async function () {
        await WithSetterRole.readerZeroAddress(userClient);
      });

      it('dataFeedIdZero', async function () {
        await WithSetterRole.dataFeedIdZero(userClient);
      });

      afterAll(async function () {
        await WithSetterRole.tearDown(client, userAccount);
      });
    });

    describe('Sender is the manager', function () {
      it('setsWhitelistExpiration', async function () {
        await WithSetterRole.setsWhitelistExpiration(client);
      });

      it('readerZeroAddress', async function () {
        await WithSetterRole.readerZeroAddress(client);
      });

      it('dataFeedIdZero', async function () {
        await WithSetterRole.dataFeedIdZero(client);
      });
    });
  });

  describe('setIndefiniteWhitelistStatus', function () {
    describe('Sender has whitelist expiration setter role', function () {
      beforeAll(async function () {
        await WithIndefiniteWhitelisterSetterRole.setup(client, userAccount, userClient);
      });

      it('setIndefiniteWhitelistStatus', async function () {
        await WithIndefiniteWhitelisterSetterRole.setIndefiniteWhitelistStatus(
          userClient,
          userAccount
        );
      });

      it('readerZeroAddress', async function () {
        await WithIndefiniteWhitelisterSetterRole.readerZeroAddress(userClient);
      });

      it('dataFeedIdZero', async function () {
        await WithIndefiniteWhitelisterSetterRole.dataFeedIdZero(userClient);
      });

      afterAll(async function () {
        await WithIndefiniteWhitelisterSetterRole.tearDown(client, userAccount);
      });
    });

    describe('Sender is the manager', function () {
      it('setIndefiniteWhitelistStatus', async function () {
        await WithIndefiniteWhitelisterSetterRole.setIndefiniteWhitelistStatus(
          client, adminAccount
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

  describe('revokeIndefiniteWhitelistStatus', function () {
    it('setIndefiniteWhitelistStatus', async function () {
      await revokesIndefiniteWhitelistStatus(
        client,
        userClient,
        userAccount,
        client
      );
    });

    it('setterHasIndefiniteWhitelisterRole', async function () {
      await setterHasIndefiniteWhitelisterRole(
        client,
        userClient,
        userAccount,
      );
    });
  });

  describe('readDataFeedWithId', function () {
    let role;

    beforeAll(async () => {
      role = (await contract.roles())[0];
    });

    it('readerNotPermitted', async function () {
      await readerNotPermitted(
        client,
        userClient,
        role,
        userAccount,
      );
    });

    it('readerUnlimitedReaderReads', async function () {
      await readerUnlimitedReaderReads(
        client,
        userAccount,
        role,
        userClient,
      );
    });

    it('readerWhitelistedReads', async function () {
      await readerWhitelistedReads(
        client,
        userAccount,
        userClient,
      );
    });
  });

  describe('readDataFeedWithName', function () {
    let name;
    let expected;
    let roles;

    beforeAll(async () => {
      const value = 456;
      const template = generateRandomBytes32();
      const timestamp = currentTimestamp();
      const airnodeAddress = keyPair.getPublicKey().data;
      const [data, signature] = await encodeAndSignData(value, template, timestamp, keyPair);
      await client.updateBeaconWithSignedData(airnodeAddress, template, timestamp, data, signature);

      const beaconId = deriveBeaconId(
        toBuffer(airnodeAddress),
        template
      );

      expected = {
        value: [...encodeData(value)],
        timestamp 
      };

      roles = await contract.roles();
      await client.grantRole(
        roles[1],
        adminAccount
      );
      name = Buffer.from(ethers.utils.formatBytes32String('My dAPI 2').substring(2), "hex");
      await setsDAPIName(client, name, beaconId);
    });

    it('readerWhitelistedReadsByName', async function () {
      await readerWhitelistedReadsByName(
        client,
        name,
        userAccount,
        userClient,
        expected
      );
    });

    it('unlimitedReaderReadsWithName', async function () {
      await unlimitedReaderReadsWithName(
        client,
        name,
        userAccount,
        roles[0],
        userClient,
        expected
      );
    });

    it('readerNotPermittedWithName', async function () {
      await readerNotPermittedWithName(
        client,
        name,
        userClient,
        roles[0],
        userAccount,
      );
    });

    afterAll(async () => {
      await client.revokeRole(
        roles[1],
        adminAccount
      );
    });
  });
});
