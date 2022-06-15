const { ensure, generateRandomBytes32, delay } = require("../util");

async function dataFeedIdToReaderToWhitelistStatus(client) {
    const reader = generateRandomBytes32().toString();
    const beaconId = [...generateRandomBytes32()];
    await client.setIndefiniteWhitelistStatus(beaconId, reader, true);
    await client.setWhitelistExpiration(beaconId, reader, 123456);
    const r = await client.dataFeedIdToReaderToWhitelistStatus(
      beaconId,
      reader
    );
    const expected = Buffer.alloc(32, 0);
    expected.writeUint8(1, 31);
    ensure(r[0] === 123456);
    expect(r[1] === [...expected]);
}

async function dataFeedIdToReaderToSetterToIndefiniteWhitelistStatus(client, setter) {
    const reader = generateRandomBytes32().toString();
    const beaconId = [...generateRandomBytes32()];

    let r = await client.dataFeedIdToReaderToSetterToIndefiniteWhitelistStatus(
      beaconId,
      reader,
      setter
    );
    ensure(!r);

    await client.setIndefiniteWhitelistStatus(beaconId, reader, true);
    await delay(1000);

    r = await client.dataFeedIdToReaderToSetterToIndefiniteWhitelistStatus(
      beaconId,
      reader,
      setter
    );
    ensure(r);
}

module.exports = { 
  dataFeedIdToReaderToWhitelistStatus, dataFeedIdToReaderToSetterToIndefiniteWhitelistStatus
};