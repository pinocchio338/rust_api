const {deriveBeaconId, deriveDApiId } = require("../../src/util");

async function derivesBeaconId(client, airnode, templateId) {
  const beaconId = await client.deriveBeaconId(airnode, templateId);
  const expected = deriveBeaconId(airnode, templateId);
  expect(beaconId).toEqual([...expected])
}

async function templateIdZero(client, airnode) {
  await expect(client.deriveBeaconId(airnode, [...Buffer.alloc(32, 0)])).rejects.toThrow("TemplateIdZero");
}

async function airnodeZero(client, templateId) {
  await expect(client.deriveBeaconId([...Buffer.alloc(32, 0)], templateId)).rejects.toThrow("AirnodeIdZero");
}

async function derivesBeaconSetId(client, beaconIds) {
  const id = await client.deriveBeaconSetId(beaconIds);
  const expected = deriveDApiId(beaconIds);
  expect(id).toEqual([...expected])
}

module.exports = { 
  derivesBeaconId, templateIdZero, airnodeZero, derivesBeaconSetId
};