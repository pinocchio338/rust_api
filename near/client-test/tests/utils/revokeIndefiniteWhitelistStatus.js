const { ensure, generateRandomBytes32, delay } = require("../../src/util");

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
    expect(r[0]).toEqual(0)
    expect(r[1]).toEqual([...Buffer.alloc(32, 0)])

    const s = await client.dataFeedIdToReaderToSetterToIndefiniteWhitelistStatus(
        beaconId,
        reader,
        listerAccount
    );
    expect(s).toBe(false)

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
    
    await expect(listerClient.revokeIndefiniteWhitelistStatus(
        beaconId,
        reader,
        listerAccount
    )).rejects.toThrow("SetterCanSetIndefiniteStatus")
}

module.exports = {
    revokesIndefiniteWhitelistStatus, setterHasIndefiniteWhitelisterRole
};