async function dAPINameZero(client) {
  await expect(client.setDapiName(Buffer.alloc(32, 0), Buffer.alloc(32, 0))).rejects.toThrow("InvalidData")
}

async function setsDAPIName(client, name, datapointId) {
  await client.setDapiName(name, datapointId);
  const id = await client.dapiNameToDataFeedId(name);
  expect([...id]).toEqual([...datapointId])
}

async function senderNotNameSetter(client, name, beaconId) {
  await expect(client.setDapiName(name, beaconId)).rejects.toThrow("AccessDenied")
}

module.exports = { 
  senderNotNameSetter, setsDAPIName, dAPINameZero
};