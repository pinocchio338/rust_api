const { ensure, generateRandomBytes32, currentTimestamp } = require("../util");


class WithIndefiniteWhitelisterSetterRole {
    static async setup(client, userAccount, userClient) {
        const indefiniteWhitelisterRole = await client.indefiniteWhitelisterRole();
        ensure(!(await client.hasRole(indefiniteWhitelisterRole, userAccount)));
        await WithIndefiniteWhitelisterSetterRole.cannotSetIndefiniteWhitelistStatus(userClient);
        await client.grantRole(indefiniteWhitelisterRole, userAccount);
    }

    static async tearDown(client, userAccount) {
        const indefiniteWhitelisterRole = await client.indefiniteWhitelisterRole();
        await client.revokeRole(indefiniteWhitelisterRole, userAccount);
    }

    static async setIndefiniteWhitelistStatus(client, listerAccount) {
        const reader = [...generateRandomBytes32()];
        const beaconId = [...generateRandomBytes32()];
        await client.setIndefiniteWhitelistStatus(beaconId, reader, true);
        const r = await client.dataFeedIdToReaderToWhitelistStatus(
            beaconId,
            reader
        );
        const expected = Buffer.alloc(32, 0);
        expected.writeUint8(1, 31);
        ensure(r[0] === 0);
        expect(r[1] === [...expected]);

        const s = await client.dataFeedIdToReaderToSetterToIndefiniteWhitelistStatus(
            beaconId,
            reader,
            listerAccount
        );
        ensure(s);
    }

    static async cannotSetIndefiniteWhitelistStatus(client) {
        const reader = [...generateRandomBytes32()];
        const beaconId = [...generateRandomBytes32()];
        try {
            await client.setIndefiniteWhitelistStatus(beaconId, reader, true);
            ensure(false);
        } catch (e) {
            ensure(e.toString().includes("AccessDenied"));
        }
    }

    static async readerZeroAddress(client) {
        const beaconId = [...generateRandomBytes32()];
        try {
            await client.setIndefiniteWhitelistStatus(beaconId, [...Buffer.alloc(32, 0)], true);
            ensure(false);
        } catch(e) {
            ensure(e.toString().includes("UserAddressZero"));
        }
    }

    static async dataFeedIdZero(client) {
        const timestamp = currentTimestamp();
        try {
            await client.setIndefiniteWhitelistStatus([...Buffer.alloc(32, 0)], [...generateRandomBytes32()], true);
            ensure(false);
        } catch(e) {
            ensure(e.toString().includes("ServiceIdZero"));
        }
    }
}

module.exports = {
    WithIndefiniteWhitelisterSetterRole
};