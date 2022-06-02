const { ensure, array_equals, deriveBeaconId, deriveDApiId } = require("../util");

async function derivesBeaconId(client, airnode, templateId) {
  const beaconId = await client.deriveBeaconId(airnode, templateId);
  const expected = deriveBeaconId(airnode, templateId);
  ensure(array_equals(beaconId, [...expected]))
}

async function templateIdZero(client, airnode) {
  try {
    const beaconId = await client.deriveBeaconId(airnode, [...Buffer.alloc(32, 0)]);
  } catch (e) {
    ensure(e.toString().includes("TempalteIdZero"));
  }
}

async function airnodeZero(client, templateId) {
  try {
    const beaconId = await client.deriveBeaconId([...Buffer.alloc(32, 0)], templateId);
  } catch (e) {
    ensure(e.toString().includes("AirnodeIdZero"));
  }
}

async function derivesBeaconSetId(client, beaconIds) {
  const id = await client.deriveBeaconSetId(beaconIds);
  const expected = deriveDApiId(beaconIds);
  ensure(array_equals(id, [...expected]));
}

module.exports = { 
  derivesBeaconId, templateIdZero, airnodeZero, derivesBeaconSetId
};