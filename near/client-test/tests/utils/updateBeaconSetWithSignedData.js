const { 
  toBuffer, currentTimestamp, bufferU64BE, deriveBeaconId,
  encodeAndSignData, delay, encodeData, keccak256Packed, deriveDApiId
} = require("../../src/util");

async function updatesBeaconSetWithSignedData(client, signer, airnodeAddress, beaconSetTemplateIds, readerClient) {
    let timestamp = currentTimestamp();
    timestamp++;
    
    const beaconIds = [];

    const [d1, d2, d3] = [100, 101, 102];
    const [data, signature] = await encodeAndSignData(d1, beaconSetTemplateIds[0], timestamp, signer);
    beaconIds.push([...deriveBeaconId(airnodeAddress, beaconSetTemplateIds[0])]);
    await client.updateBeaconWithSignedData(airnodeAddress, beaconSetTemplateIds[0], timestamp, data, signature);
    await delay(1000);

    // Sign data for the next two beacons
    beaconIds.push([...deriveBeaconId(airnodeAddress, beaconSetTemplateIds[1])]);
    const [data1, signature1] = await encodeAndSignData(d2, beaconSetTemplateIds[1], timestamp, signer);
    beaconIds.push([...deriveBeaconId(airnodeAddress, beaconSetTemplateIds[2])]);
    const [data2, signature2] = await encodeAndSignData(d3, beaconSetTemplateIds[2], timestamp, signer);

    await client.updateBeaconSetWithSignedData(
      [airnodeAddress, airnodeAddress, airnodeAddress],
      beaconSetTemplateIds,
      [0, timestamp, timestamp],
      [[], [...data1], [...data2]],
      [[], [...signature1], [...signature2]]
    );

    const beacon = await readerClient.readDataFeedWithId([...deriveDApiId(beaconIds)]);
    expect(beacon.value).toEqual([...encodeData(d2)])
    expect(beacon.timestamp).toEqual(timestamp)
}

async function updatedSetValueOutdated(client, signer, airnodeAddress, beaconSetTemplateIds) {
    let timestamp = currentTimestamp();
    timestamp++;
    
    const [data, signature] = await encodeAndSignData(100, beaconSetTemplateIds[0], timestamp, signer);
    await client.updateBeaconWithSignedData(airnodeAddress, beaconSetTemplateIds[0], timestamp, data, signature);
    await delay(1000);

    // Sign data for the next two beacons
    let [data1, signature1] = await encodeAndSignData(105, beaconSetTemplateIds[1], timestamp, signer);
    let [data2, signature2] = await encodeAndSignData(110, beaconSetTemplateIds[2], timestamp, signer);

    await client.updateBeaconSetWithSignedData(
      [airnodeAddress, airnodeAddress, airnodeAddress],
      beaconSetTemplateIds,
      [0, timestamp, timestamp],
      [[], [...data1], [...data2]],
      [[], [...signature1], [...signature2]]
    );
    await delay(1000);

    [data1, signature1] = await encodeAndSignData(105, beaconSetTemplateIds[1], timestamp-5, signer);
    [data2, signature2] = await encodeAndSignData(110, beaconSetTemplateIds[2], timestamp-5, signer);
    await expect(client.updateBeaconSetWithSignedData(
      [airnodeAddress, airnodeAddress, airnodeAddress],
      beaconSetTemplateIds,
      [0, timestamp-5, timestamp-5],
      [[], [...data1], [...data2]],
      [[], [...signature1], [...signature2]]
    )).rejects.toThrow("UpdatedValueOutdated")
}

async function lengthNotCorrect(client, signer, airnodeAddress, beaconSetTemplateIds) {
  const timestamp = currentTimestamp();

  const data = Buffer.allocUnsafe(21);
  const bufferedTemplate = toBuffer(Buffer.from(beaconSetTemplateIds[1], 'hex'));
  const bufferedTimestamp = bufferU64BE(timestamp);
  const message = keccak256Packed(
    ["bytes32", "uint256", "bytes"],
    [bufferedTemplate, bufferedTimestamp, data]
  );
  const signature = await signer.sign(message);
  await expect(client.updateBeaconSetWithSignedData(
    [airnodeAddress, airnodeAddress],
    [beaconSetTemplateIds[0], beaconSetTemplateIds[1]],
    [0, timestamp],
    [[], [...data]],
    [[], [...signature.signature]]
  )).rejects.toThrow("InvalidDataLength")
}

async function notAllSignaturesValid(client, signer, airnodeAddress, beaconSetTemplateIds) {
  const timestamp = currentTimestamp();

  const data = Buffer.alloc(21, 0);
  await expect(client.updateBeaconSetWithSignedData(
    [airnodeAddress, airnodeAddress],
    [beaconSetTemplateIds[0], beaconSetTemplateIds[1]],
    [0, timestamp],
    [[], [...data]],
    [[], [...Buffer.alloc(64)]]
  )).rejects.toThrow("InvalidSignature") 
}

async function lessThanTwoBeacons(client) {
  await expect(client.updateBeaconSetWithSignedData(
    [airnodeAddress],
    [beaconSetTemplateIds[0]],
    [0],
    [[]],
    [[]]
  )).rejects.toThrow("LessThanTwoBeacons")
}

async function parameterLengthMismatch(client, airnodeAddress, beaconSetTemplateIds) {
  await expect(client.updateBeaconSetWithSignedData(
    [airnodeAddress],
    [beaconSetTemplateIds[0]],
    [0, 123],
    [[]],
    [[]]
  )).rejects.toThrow("ParameterLengthMismatch")
}

module.exports = { 
  updatesBeaconSetWithSignedData, updatedSetValueOutdated, lengthNotCorrect,
  lessThanTwoBeacons, notAllSignaturesValid, parameterLengthMismatch
};