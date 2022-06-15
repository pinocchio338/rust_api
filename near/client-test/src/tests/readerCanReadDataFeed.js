const { ensure, generateRandomBytes32 } = require("../util");

async function readerZeroAddress(client) {
  const r = await client.readerCanReadDataFeed([...Buffer.alloc(32, 0)], "");;
  ensure(r);
}

async function readerWhitelisted(client, datapoint, reader) {
  await client.setIndefiniteWhitelistStatus(datapoint, reader, false);
  let canRead = await client.readerCanReadDataFeed([...datapoint], reader);
  ensure(!canRead);

  await client.setIndefiniteWhitelistStatus(datapoint, reader, true);
  canRead = await client.readerCanReadDataFeed([...datapoint], reader);
  ensure(canRead);
}

async function readerUnlimitedReaderRole(client, reader, role) {
    await client.grantRole(role, reader);
    const datapoint = generateRandomBytes32();

    let canRead = await client.readerCanReadDataFeed([...datapoint], reader);
    ensure(canRead);

    await client.revokeRole(role, reader);
    canRead = await client.readerCanReadDataFeed([...datapoint], reader);
    ensure(!canRead);
}

async function readerNotWhitelisted(client, reader) {
  const r = await client.readerCanReadDataFeed([...generateRandomBytes32()], reader);
  ensure(!r);
}

module.exports = { 
    readerZeroAddress, readerWhitelisted, readerNotWhitelisted, readerUnlimitedReaderRole
};