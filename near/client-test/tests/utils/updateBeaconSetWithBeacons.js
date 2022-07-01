const { deriveDApiId, encodeData } = require("../../src/util");

async function updatesBeaconSet(client, beaconIds, expectedValue, expectedTimestamp, readerClient) {
    await client.updateBeaconSetWithBeacons(beaconIds);
    const expectedId = deriveDApiId(beaconIds);

    const beacon = await readerClient.readDataFeedWithId(expectedId);
    expect(beacon.timestamp).toEqual(expectedTimestamp)
    expect(beacon.value).toEqual([...encodeData(expectedValue)])
}

async function lessThanTwoBeacons(client) {
  await expect(client.updateBeaconSetWithBeacons([])).rejects.toThrow("LessThanTwoBeacons")
}

module.exports = { 
  updatesBeaconSet, lessThanTwoBeacons
};