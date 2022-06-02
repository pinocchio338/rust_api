const { deriveBeaconId, deriveDApiId, ensure } = require("../util");

async function updatesBeaconSet(client, beaconIds, expectedValue, expectedTimestamp) {
    const beaconId = await client.updateBeaconSetWithBeacons(beaconIds);
    const expectedId = deriveDApiId(beaconIds);

    // const beacon = await client.readDataFeedWithId(expectedId);
    // expect([...beacon.value]).to.equal(expectedValue);
    // expect(beacon.timestamp).to.equal(expectedTimestamp);
}

async function updatedValueOutdated(client, beaconIds, expectedId, expectedValue, expectedTimestamp) {
    // TODO
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