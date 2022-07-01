const { fail } = require("assert");
const { 
  currentTimestamp, encodeAndSignData, encodeData, toBuffer, deriveBeaconId,
  prepareMessage,
 } = require("../../src/util");

async function updateBeacon(client, signer, airnodeAddress, templateId, value, timestamp, readerClient) {
    const [data, signature] = await encodeAndSignData(value, templateId, timestamp, signer);
    await client.updateBeaconWithSignedData(airnodeAddress, templateId, timestamp, data, signature);

    const beaconId = deriveBeaconId(
      toBuffer(airnodeAddress),
      templateId
    );
    return await readerClient.readDataFeedWithId(beaconId);

}

async function dataNotFresherThanBeacon(client, signer, airnodeAddress, templateId) {
    const timestamp = currentTimestamp() - 1000;
    const [data, signature] = await encodeAndSignData(123, templateId, timestamp, signer);
    await expect(client.updateBeaconWithSignedData(airnodeAddress, templateId, timestamp, data, signature)).rejects.toThrow("FulfillmentOlderThanBeacon")
}

async function dataLengthNotCorrect(client, signer, airnodeAddress, templateId) {
    const timestamp = currentTimestamp() - 1000;
    const data = encodeData(123);
    data.writeUint16BE(0);
    const {signature} = await signer.sign(prepareMessage(templateId, timestamp, data));
    await expect(client.updateBeaconWithSignedData(airnodeAddress, templateId, timestamp, data, signature)).rejects.toThrow("InvalidDataLength")
}

async function timestampNotValid(client, signer, airnodeAddress, templateId) {
  const timestamp = currentTimestamp() - 1000;
  const [data, signature] = await encodeAndSignData(123, templateId, timestamp, signer);
  await expect(client.updateBeaconWithSignedData(airnodeAddress, templateId, timestamp, data, signature)).rejects.toThrow("InvalidTimestamp")
}

async function signatureNotValid(client, signer, airnodeAddress, templateId) {
  const timestamp = currentTimestamp();
  const [data, ] = await encodeAndSignData(123, templateId, timestamp, signer);
  await expect(client.updateBeaconWithSignedData(airnodeAddress, templateId, timestamp, data, Buffer.allocUnsafe(64))).rejects.toThrow("InvalidSignature")
}

module.exports = { 
  updateBeacon, dataNotFresherThanBeacon, dataLengthNotCorrect, timestampNotValid,
  signatureNotValid
};