const { ensure, generateRandomBytes32, delay } = require("../util");

async function revokesIndefiniteWhitelistStatus(client, listerClient, listerAccount, randomClient) {
    const reader = generateRandomBytes32().toString();
    const beaconId = [...generateRandomBytes32()];

    const indefiniteWhitelisterRole = await client.indefiniteWhitelisterRole();
    
    await client.grantRole(indefiniteWhitelisterRole, listerAccount);
    await delay(1000);

    await listerClient.setIndefiniteWhitelistStatus(beaconId, reader, true);
    await client.revokeRole(indefiniteWhitelisterRole, listerAccount);
    await delay(1000);

    await randomClient.revokeIndefiniteWhitelistStatus(
        beaconId,
        reader,
        listerAccount
    );

    const r = await client.dataFeedIdToReaderToWhitelistStatus(
        beaconId,
        reader
    );
    ensure(r[0] === 0);
    expect(r[1] === [...Buffer.alloc(32, 0)]);

    const s = await client.dataFeedIdToReaderToSetterToIndefiniteWhitelistStatus(
        beaconId,
        reader,
        listerAccount
    );
    ensure(!s);

    await randomClient.revokeIndefiniteWhitelistStatus(
        beaconId,
        reader,
        listerAccount
    );
}

async function setterHasIndefiniteWhitelisterRole(client, listerClient, listerAccount) {
    const reader = generateRandomBytes32().toString();
    const beaconId = [...generateRandomBytes32()];

    const indefiniteWhitelisterRole = await client.indefiniteWhitelisterRole();
    
    await client.grantRole(indefiniteWhitelisterRole, listerAccount);
    await delay(1000);
    
    try {
        await listerClient.revokeIndefiniteWhitelistStatus(
            beaconId,
            reader,
            listerAccount
        );
        ensure(false);
    } catch(e) {
        ensure(e.toString().includes("SetterCanSetIndefiniteStatus"));
    }
}

module.exports = {
    revokesIndefiniteWhitelistStatus, setterHasIndefiniteWhitelisterRole
};