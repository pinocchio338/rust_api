const { ensure, generateRandomBytes32, delay, array_equals, keccak256Packed } = require("../util");

async function readerWhitelistedReadsByName(client, name, reader, userClient, expected) {
    const datapoint = keccak256Packed(['bytes32'], [name]);
    await client.setIndefiniteWhitelistStatus(datapoint, reader, true);
    await delay(1000);

    const r = await userClient.readDataFeedWithDapiName([...name]);
    ensure(array_equals(r.value, expected.value));
    ensure(r.timestamp, expected.timestamp);

    await client.setIndefiniteWhitelistStatus(datapoint, reader, true);
}

async function unlimitedReaderReadsWithName(client, name, reader, role, userClient, expected) {
    await client.grantRole(role, reader);
    await delay(1000);

    const r = await userClient.readDataFeedWithDapiName([...name]);
    ensure(array_equals(r.value, expected.value));
    ensure(r.timestamp, expected.timestamp);

    await client.revokeRole(role, reader);
}

async function readerNotPermittedWithName(client, name, userClient, role, userAccount) {
    await client.revokeRole(role, userAccount);
    const datapoint = keccak256Packed(['bytes32'], [name]);
    await client.setIndefiniteWhitelistStatus(datapoint, userAccount, false);
    await delay(1000);
    try {
        await userClient.readDataFeedWithDapiName([...datapoint]);
        ensure(false);
    } catch (e) {
        ensure(e.toString().includes("AccessDenied"));
    }
}

module.exports = { 
  readerWhitelistedReadsByName, readerNotPermittedWithName, unlimitedReaderReadsWithName
};