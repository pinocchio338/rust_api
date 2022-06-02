const { ensure, array_equals } = require("../util");

async function dAPINameZero(client) {
  try {
    await client.setDapiName(Buffer.alloc(32, 0), Buffer.alloc(32, 0));
  } catch(e) {
    ensure(e.toString().includes("InvalidData"));
  }
}

async function setsDAPIName(client, name, datapointId) {
  await client.setDapiName(name, datapointId);
  const id = await client.dapiNameToDataFeedId(name);
  ensure(
    array_equals(
      [...id],
      [...datapointId]
    )
  );
}

async function senderNotNameSetter(client, name, beaconId) {
  try {
    await client.setDapiName(name, beaconId);
  } catch(e) {
    ensure(e.toString().includes("AccessDenied"));
  }
}

module.exports = { 
  senderNotNameSetter, setsDAPIName, dAPINameZero
};