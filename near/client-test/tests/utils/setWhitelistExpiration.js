const { generateRandomBytes32, currentTimestamp } = require("../../src/util");


class WithSetterRole {
    static async setup(client, userAccount, userClient) {
        const whitelistExpirationSetterRole = await client.whitelistExpirationSetterRole();
        await expect(client.hasRole(whitelistExpirationSetterRole, userAccount)).resolves.toBe(false);
        await WithSetterRole.cannotSetWhitelistExpiration(userClient);
        await client.grantRole(whitelistExpirationSetterRole, userAccount);
    }

    static async tearDown(client, userAccount) {
        const whitelistExpirationSetterRole = await client.whitelistExpirationSetterRole();
        await client.revokeRole(whitelistExpirationSetterRole, userAccount);
    }

    static async setsWhitelistExpiration(client) {
        const timestamp = currentTimestamp();
        const reader = generateRandomBytes32().toString();
        const beaconId = [...generateRandomBytes32()];
        await client.setWhitelistExpiration(beaconId, reader, timestamp);
        const r = await client.dataFeedIdToReaderToWhitelistStatus(
            beaconId,
            reader
        );
        const expected = Buffer.alloc(32, 0);
        expected.writeUint8(1, 31);
        expect(r[0]).toEqual(timestamp)
        expect(r[1]).toEqual([...expected])
    }

    static async cannotSetWhitelistExpiration(client) {
        const timestamp = currentTimestamp();
        const reader = generateRandomBytes32().toString();
        const beaconId = [...generateRandomBytes32()];
        await expect(client.setWhitelistExpiration(beaconId, reader, timestamp)).rejects.toThrow("AccessDenied")
    }

    static async readerZeroAddress(client) {
        const timestamp = currentTimestamp();
        const beaconId = [...generateRandomBytes32()];
        await expect(client.setWhitelistExpiration(beaconId, "", timestamp)).rejects.toThrow("UserAddressZero")
    }

    static async dataFeedIdZero(client) {
        const timestamp = currentTimestamp();
        await expect(client.setWhitelistExpiration([...Buffer.alloc(32, 0)], generateRandomBytes32().toString(), timestamp)).rejects.toThrow("ServiceIdZero")
    }
}

module.exports = {
    WithSetterRole
};