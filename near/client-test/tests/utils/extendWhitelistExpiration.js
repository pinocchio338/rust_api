const { execPath } = require("process");
const { generateRandomBytes32, currentTimestamp, delay } = require("../../src/util");


class WithExtenderRole {
    static async setup(client, userAccount) {
        const whitelistExpirationExtenderRole = await client.whitelistExpirationExtenderRole();
        const hasRole = await client.hasRole(whitelistExpirationExtenderRole, userAccount);
        if (!hasRole) {
            await client.grantRole(whitelistExpirationExtenderRole, userAccount);
        }
    }

    static async tearDown(client, userAccount) {
        const whitelistExpirationExtenderRole = await client.whitelistExpirationExtenderRole();
        await client.revokeRole(whitelistExpirationExtenderRole, userAccount);
    }

    static async cannotExtendWhitelistExpiration(client, userAccount, userClient) {
        const whitelistExpirationExtenderRole = await client.whitelistExpirationExtenderRole();
        const hasRole = await client.hasRole(whitelistExpirationExtenderRole, userAccount);

        await client.revokeRole(whitelistExpirationExtenderRole, userAccount);
        await delay(1000);

        const timestamp = currentTimestamp();
        const reader = generateRandomBytes32().toString();
        const beaconId = [...generateRandomBytes32()];
        await expect(userClient.extendWhitelistExpiration(beaconId, reader, timestamp)).rejects.toThrow("AccessDenied")

        // recover to its starting condition
        if (hasRole) {
            await client.grantRole(whitelistExpirationExtenderRole, userAccount);
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
        expect(r[0]).toEqual(timestamp);
        expect(r[1]).toEqual([...expected]);
    }

    static async doesNotExtendExpiration(client) {
        const reader = generateRandomBytes32().toString();
        const beaconId = [...generateRandomBytes32()];

        await expect(client.extendWhitelistExpiration(beaconId, reader, 0)).rejects.toThrow("DoesNotExtendExpiration")
    }

    static async readerZeroAddress(client) {
        const timestamp = currentTimestamp();
        const beaconId = [...generateRandomBytes32()];
        await expect(client.extendWhitelistExpiration(beaconId, "", timestamp)).rejects.toThrow("UserAddressZero")
    }

    static async dataFeedIdZero(client) {
        const timestamp = currentTimestamp();
        await expect(client.extendWhitelistExpiration([...Buffer.alloc(32, 0)], generateRandomBytes32().toString(), timestamp)).rejects.toThrow("ServiceIdZero")
    }
}

module.exports = {
    WithExtenderRole
};