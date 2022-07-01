const { keccak256Packed } = require("../../src/util");

async function readerWhitelistedReadsByName(client, name, reader, userClient, expected) {
    const datapoint = keccak256Packed(['bytes32'], [name]);
    await client.setIndefiniteWhitelistStatus(datapoint, reader, true);

    const r = await userClient.readDataFeedWithDapiName([...name]);
    expect(r.value).toEqual(expected.value);
    expect(r.timestamp).toEqual(expected.timestamp);

    await client.setIndefiniteWhitelistStatus(datapoint, reader, true);
}

async function unlimitedReaderReadsWithName(client, name, reader, role, userClient, expected) {
    await client.grantRole(role, reader);

    const r = await userClient.readDataFeedWithDapiName([...name]);
    expect(r.value).toEqual(expected.value);
    expect(r.timestamp).toEqual(expected.timestamp);

    await client.revokeRole(role, reader);
}

async function readerNotPermittedWithName(client, name, userClient, role, userAccount) {
    await client.revokeRole(role, userAccount);
    const datapoint = keccak256Packed(['bytes32'], [name]);
    await client.setIndefiniteWhitelistStatus(datapoint, userAccount, false);
    await expect(userClient.readDataFeedWithDapiName([...datapoint])).rejects.toThrow("AccessDenied")
}

module.exports = { 
  readerWhitelistedReadsByName, readerNotPermittedWithName, unlimitedReaderReadsWithName
};