const { deriveDApiId, ensure, array_equals, encodeData } = require("../util");

async function updatesBeaconSet(client, beaconIds, expectedValue, expectedTimestamp, readerClient) {
    await client.updateBeaconSetWithBeacons(beaconIds);
    const expectedId = deriveDApiId(beaconIds);

    const beacon = await readerClient.readDataFeedWithId(expectedId);
    ensure(
      array_equals(beacon.value, [...encodeData(expectedValue)])
    );
    ensure(beacon.timestamp === expectedTimestamp);
}

async function updatedValueOutdated(client, beaconIds, expectedId) {
  // NOTE: seems not possible to test this case in near devnet automatically, skip for now
}

async function lessThanTwoBeacons(client) {
    try {
      await client.updateBeaconSetWithBeacons([]);
    } catch (e) {
      ensure(e.toString().includes("LessThanTwoBeacons"));
    }
}

module.exports = { 
  updatesBeaconSet, updatedValueOutdated, lessThanTwoBeacons
};