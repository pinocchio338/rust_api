const { 
  currentTimestamp, encodeAndSignData, encodeData, toBuffer, deriveBeaconId,
  prepareMessage
 } = require("../util");

async function updateBeacon(client, signer, airnodeAddress, templateId, value, timestamp) {
    const [data, signature] = await encodeAndSignData(value, templateId, timestamp, signer);
    await client.updateBeaconWithSignedData(airnodeAddress, templateId, timestamp, data, signature);

    const beaconId = deriveBeaconId(
      toBuffer(airnodeAddress),
      templateId
    );
    // const beacon = await client.readDataFeedWithId(beaconId);
    // expect(beacon.value).to.equal([...encodeData(123)]);
    // expect(beacon.timestamp).to.equal(timestamp);
}

async function dataNotFresherThanBeacon(client, signer, airnodeAddress, templateId) {
    const timestamp = currentTimestamp() - 1000;
    const [data, signature] = await encodeAndSignData(123, templateId, timestamp, signer);
    try {
      await client.updateBeaconWithSignedData(airnodeAddress, templateId, timestamp, data, signature);
    } catch (e) {
      expect(e.toString().includes("FulfillmentOlderThanBeacon"))
    }
}

async function dataLengthNotCorrect(client, signer, airnodeAddress, templateId) {
    const timestamp = currentTimestamp() - 1000;
    const data = encodeData(123);
    data.writeUint16BE(0);
    const signature = await signer.sign(prepareMessage(templateId, timestamp, data));
    try {
        await client.updateBeaconWithSignedData(airnodeAddress, templateId, timestamp, data, signature);
    } catch (e) {
        expect(e.toString().includes("InvalidDataLength"))
    }
}

async function timestampNotValid(client, signer, airnodeAddress, templateId) {
  const timestamp = currentTimestamp() - 1000;
  const [data, signature] = await encodeAndSignData(123, templateId, timestamp, signer);
  try {
    await client.updateBeaconWithSignedData(airnodeAddress, templateId, timestamp, data, signature);
  } catch (e) {
    expect(e.toString().includes("InvalidTimestamp"))
  }
}

async function signatureNotValid(client, signer, airnodeAddress, templateId) {
  const timestamp = currentTimestamp();
  const [data, ] = await encodeAndSignData(123, templateId, timestamp, signer);
  try {
    await client.updateBeaconWithSignedData(airnodeAddress, templateId, timestamp, data, Buffer.allocUnsafe(32));
  } catch (e) {
    expect(e.toString().includes("InvalidSignature"))
  }
}

module.exports = { 
  updateBeacon, dataNotFresherThanBeacon, dataLengthNotCorrect, timestampNotValid,
  signatureNotValid
};