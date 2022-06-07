const { ensure, generateRandomBytes32, currentTimestamp } = require("../util");


class WithSetterRole {
    static async setup(client, userAccount, userClient) {
        const whitelistExpirationSetterRole = await client.whitelistExpirationSetterRole();
        ensure(!(await client.hasRole(whitelistExpirationSetterRole, userAccount)));
        await WithSetterRole.cannotSetWhitelistExpiration(userClient);
        await client.grantRole(whitelistExpirationSetterRole, userAccount);
    }

    static async tearDown(client, userAccount) {
        const whitelistExpirationSetterRole = await client.whitelistExpirationSetterRole();
        await client.revokeRole(whitelistExpirationSetterRole, userAccount);
    }

    static async setsWhitelistExpiration(client) {
        const timestamp = currentTimestamp();
        const reader = [...generateRandomBytes32()];
        const beaconId = [...generateRandomBytes32()];
        await client.setWhitelistExpiration(beaconId, reader, timestamp);
        const r = await client.dataFeedIdToReaderToWhitelistStatus(
            beaconId,
            reader
        );
        const expected = Buffer.alloc(32, 0);
        expected.writeUint8(1, 31);
        ensure(r[0] === timestamp);
        expect(r[1] === [...expected]);
    }

    static async cannotSetWhitelistExpiration(client) {
        const timestamp = currentTimestamp();
        const reader = [...generateRandomBytes32()];
        const beaconId = [...generateRandomBytes32()];
        try {
            await client.setWhitelistExpiration(beaconId, reader, timestamp);
            ensure(false);
        } catch (e) {
            ensure(e.toString().includes("AccessDenied"));
        }
    }

    static async readerZeroAddress(client) {
        const timestamp = currentTimestamp();
        const beaconId = [...generateRandomBytes32()];
        try {
            await client.setWhitelistExpiration(beaconId, [...Buffer.alloc(32, 0)], timestamp);
            ensure(false);
        } catch(e) {
            ensure(e.toString().includes("UserAddressZero"));
        }
    }

    static async dataFeedIdZero(client) {
        const timestamp = currentTimestamp();
        try {
            await client.setWhitelistExpiration([...Buffer.alloc(32, 0)], [...generateRandomBytes32()], timestamp);
            ensure(false);
        } catch(e) {
            ensure(e.toString().includes("ServiceIdZero"));
        }
    }
}

module.exports = {
    WithSetterRole
};