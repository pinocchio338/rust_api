const { ensure, generateRandomBytes32, currentTimestamp } = require("../util");


class WithExtenderRole {
    static async setup(client, userAccount, userClient) {
        const whitelistExpirationExtenderRole = await client.whitelistExpirationExtenderRole();
        ensure(!(await client.hasRole(whitelistExpirationExtenderRole, userAccount)));
        await WithExtenderRole.cannotExtendWhitelistExpiration(userClient);
        await client.grantRole(whitelistExpirationExtenderRole, userAccount);
    }

    static async tearDown(client, userAccount) {
        const whitelistExpirationExtenderRole = await client.whitelistExpirationExtenderRole();
        await client.revokeRole(whitelistExpirationExtenderRole, userAccount);
    }

    static async cannotExtendWhitelistExpiration(client) {
        const timestamp = currentTimestamp();
        const reader = generateRandomBytes32().toString();
        const beaconId = [...generateRandomBytes32()];
        try {
            await client.extendWhitelistExpiration(beaconId, reader, timestamp);
            ensure(false);
        } catch (e) {
            ensure(e.toString().includes("AccessDenied"));
        }
    }

    static async extendsWhitelistExpiration(client) {
        const timestamp = currentTimestamp();
        const reader = generateRandomBytes32().toString();
        const beaconId = [...generateRandomBytes32()];
        await client.extendWhitelistExpiration(beaconId, reader, timestamp);
        const r = await client.dataFeedIdToReaderToWhitelistStatus(
            beaconId,
            reader
        );
        const expected = Buffer.alloc(32, 0);
        expected.writeUint8(1, 31);
        ensure(r[0] === timestamp);
        expect(r[1] === [...expected]);
    }

    static async doesNotExtendExpiration(client) {
        const reader = generateRandomBytes32().toString();
        const beaconId = [...generateRandomBytes32()];

        try {
            await client.extendWhitelistExpiration(beaconId, reader, 0);
            ensure(false);
        } catch(e) {
            ensure(e.toString().includes("DoesNotExtendExpiration"));
        }
    }

    static async readerZeroAddress(client) {
        const timestamp = currentTimestamp();
        const beaconId = [...generateRandomBytes32()];
        try {
            await client.extendWhitelistExpiration(beaconId, "", timestamp);
            ensure(false);
        } catch(e) {
            ensure(e.toString().includes("UserAddressZero"));
        }
    }

    static async dataFeedIdZero(client) {
        const timestamp = currentTimestamp();
        try {
            await client.extendWhitelistExpiration([...Buffer.alloc(32, 0)], generateRandomBytes32().toString(), timestamp);
            ensure(false);
        } catch(e) {
            ensure(e.toString().includes("ServiceIdZero"));
        }
    }
}

module.exports = {
    WithExtenderRole
};