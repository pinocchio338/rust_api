const { generateRandomBytes32, delay } = require("../../src/util");


class WithIndefiniteWhitelisterSetterRole {
    static async setup(client, userAccount, userClient) {
        const indefiniteWhitelisterRole = await client.indefiniteWhitelisterRole();
        await client.revokeRole(indefiniteWhitelisterRole, userAccount);
        await delay(5000);
        await WithIndefiniteWhitelisterSetterRole.cannotSetIndefiniteWhitelistStatus(userClient);
        await client.grantRole(indefiniteWhitelisterRole, userAccount);
        await delay(3000);
    }

    static async tearDown(client, userAccount) {
        const indefiniteWhitelisterRole = await client.indefiniteWhitelisterRole();
        await client.revokeRole(indefiniteWhitelisterRole, userAccount);
        await delay(3000);
    }

    static async setIndefiniteWhitelistStatus(client, listerAccount) {
        const reader = generateRandomBytes32().toString();
        const beaconId = [...generateRandomBytes32()];
        await client.setIndefiniteWhitelistStatus(beaconId, reader, true);
        const r = await client.dataFeedIdToReaderToWhitelistStatus(
            beaconId,
            reader
        );
        const expected = Buffer.alloc(32, 0);
        expected.writeUint8(1, 31);

        expect(r[0]).toEqual(0)
        expect(r[1]).toEqual([...expected])

        const s = await client.dataFeedIdToReaderToSetterToIndefiniteWhitelistStatus(
            beaconId,
            reader,
            listerAccount
        );
        expect(s).toBe(true)
    }

    static async cannotSetIndefiniteWhitelistStatus(client) {
        const reader = generateRandomBytes32().toString();
        const beaconId = [...generateRandomBytes32()];
        await expect(client.setIndefiniteWhitelistStatus(beaconId, reader, true)).rejects.toThrow("AccessDenied")
    }

    static async readerZeroAddress(client) {
        const beaconId = [...generateRandomBytes32()];
        await expect(client.setIndefiniteWhitelistStatus(beaconId, "", true)).rejects.toThrow("UserAddressZero")
    }

    static async dataFeedIdZero(client) {
        await expect(client.setIndefiniteWhitelistStatus([...Buffer.alloc(32, 0)], generateRandomBytes32().toString(), true)).rejects.toThrow("ServiceIdZero")
    }
}

module.exports = {
    WithIndefiniteWhitelisterSetterRole
};