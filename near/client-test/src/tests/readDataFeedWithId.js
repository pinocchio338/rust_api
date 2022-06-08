const { ensure, generateRandomBytes32, delay } = require("../util");

async function readerWhitelistedReads(client, reader, userClient) {
    const datapoint = generateRandomBytes32();

    await client.setIndefiniteWhitelistStatus(datapoint, reader, true);
    await delay(1000);

    // we are testing the access here, dont care about the return results
    // other tests should have this covered already
    await userClient.readDataFeedWithId([...datapoint]);

    await client.setIndefiniteWhitelistStatus(datapoint, reader, true);
}

async function readerUnlimitedReaderReads(client, reader, role, userClient) {
    await client.grantRole(role, reader);
    await delay(1000);

    const datapoint = generateRandomBytes32();

    // we are testing the access here, dont care about the return results
    // other tests should have this covered already
    await userClient.readDataFeedWithId([...datapoint]);

    await client.revokeRole(role, reader);
}

async function readerNotPermitted(client, userClient, role, userAccount) {
    await client.revokeRole(role, userAccount);
    const datapoint = generateRandomBytes32();
    await client.setIndefiniteWhitelistStatus(datapoint, userAccount, false);
    await delay(1000);
    try {
        await userClient.readDataFeedWithId([...datapoint]);
        ensure(false);
    } catch (e) {
        ensure(e.toString().includes("AccessDenied"));
    }
}

module.exports = { 
    readerWhitelistedReads, readerNotPermitted, readerUnlimitedReaderReads
};