const { fail } = require("assert");
const { ethers } = require("ethers");
const { 
  currentTimestamp, encodeAndSignData, encodeData, toBuffer, deriveBeaconId,
  prepareMessage,
  generateRandomBytes32,
  bufferU64BE,
  keccak256Packed,
  delay,
 } = require("../../src/util");

async function updateBeacon(client, signer, airnodeAddress, templateId, value, timestamp, readerClient) {
    const [data, signature] = await encodeAndSignData(value, templateId, timestamp, signer);
    await client.updateBeaconWithSignedData(airnodeAddress, templateId, timestamp, data, signature);

    const beaconId = deriveBeaconId(
      toBuffer(airnodeAddress),
      templateId
    );
    const beacon = await readerClient.readDataFeedWithId(beaconId);
    expect(beacon.value).toEqual([...encodeData(value)])
    expect(beacon.timestamp).toEqual(timestamp)

}

async function dataNotFresherThanBeacon(client, signer, airnodeAddress, templateId) {
    const timestamp = currentTimestamp() - 1000;
    const [data, signature] = await encodeAndSignData(123, templateId, timestamp, signer);
    await expect(client.updateBeaconWithSignedData(airnodeAddress, templateId, timestamp, data, signature)).rejects.toThrow("FulfillmentOlderThanBeacon")
}

async function dataLengthNotCorrect(client, signer, airnodeAddress, templateId) {
    const timestamp = currentTimestamp();
    const data = ethers.utils.randomBytes(30);

    // prepare signature
    const bufferedTemplate = toBuffer(Buffer.from(templateId, 'hex'));
    const bufferedTimestamp = bufferU64BE(timestamp);
    const message = keccak256Packed(
        ["bytes32", "uint256", "bytes"],
        [bufferedTemplate, bufferedTimestamp, data]
    );
    const {signature} = await signer.sign(message);

    await expect(client.updateBeaconWithSignedData(airnodeAddress, templateId, timestamp, data, signature)).rejects.toThrow("InvalidDataLength")
}

async function timestampNotValid(client, signer, airnodeAddress) {
  // we are using a randon templated id for now
  const templateId = generateRandomBytes32();

  // we update once first, ensure there is some data.
  let timestamp = currentTimestamp();
  let [data, signature] = await encodeAndSignData(1234, templateId, timestamp, signer);
  await client.updateBeaconWithSignedData(airnodeAddress, templateId, timestamp, data, signature);

  // mimic some other operation
  await delay(1000);

  // now update with an older timestamp
  timestamp = timestamp - 1;
  [data, signature] = await encodeAndSignData(123, templateId, timestamp, signer);
  await expect(client.updateBeaconWithSignedData(airnodeAddress, templateId, timestamp, data, signature)).rejects.toThrow("FulfillmentOlderThanBeacon")
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