const { generateRandomBytes32 } = require("../../src/util");

async function readerZeroAddress(client) {
  const r = await client.readerCanReadDataFeed([...Buffer.alloc(32, 0)], "");;
  expect(r).toBe(true);
}

async function readerWhitelisted(client, datapoint, reader) {
  await client.setIndefiniteWhitelistStatus(datapoint, reader, false);
  let canRead = await client.readerCanReadDataFeed([...datapoint], reader);
  expect(canRead).toBe(false);

  await client.setIndefiniteWhitelistStatus(datapoint, reader, true);
  canRead = await client.readerCanReadDataFeed([...datapoint], reader);
  expect(canRead).toBe(true)
}

async function readerUnlimitedReaderRole(client, reader, role) {
    await client.grantRole(role, reader);
    const datapoint = generateRandomBytes32();

    let canRead = await client.readerCanReadDataFeed([...datapoint], reader);
    expect(canRead).toBe(true)

    await client.revokeRole(role, reader);
    canRead = await client.readerCanReadDataFeed([...datapoint], reader);
    expect(canRead).toBe(false)
}

async function readerNotWhitelisted(client, reader) {
  const r = await client.readerCanReadDataFeed([...generateRandomBytes32()], reader);
  expect(r).toBe(false)
}

module.exports = { 
    readerZeroAddress, readerWhitelisted, readerNotWhitelisted, readerUnlimitedReaderRole
};