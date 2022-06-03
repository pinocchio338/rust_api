const { deriveBeaconId, deriveDApiId, ensure, currentTimestamp, encodeAndSignData, delay, encodeData, prepareMessage } = require("../util");

async function updatesBeaconSetWithSignedData(client, signer, airnodeAddress, beaconSetTemplateIds) {
    let timestamp = currentTimestamp();
    timestamp++;
    
    const [data, signature] = await encodeAndSignData(100, beaconSetTemplateIds[0], timestamp, signer);
    await client.updateBeaconWithSignedData(airnodeAddress, beaconSetTemplateIds[0], timestamp, data, signature);
    await delay(1000);

    // Sign data for the next two beacons
    const [data1, signature1] = await encodeAndSignData(105, beaconSetTemplateIds[1], timestamp, signer);
    const [data2, signature2] = await encodeAndSignData(110, beaconSetTemplateIds[2], timestamp, signer);

    await client.updateBeaconSetWithSignedData(
      [airnodeAddress, airnodeAddress, airnodeAddress],
      beaconSetTemplateIds,
      [0, timestamp, timestamp],
      [[], [...data1], [...data2]],
      [[], [...signature1], [...signature2]]
    )

    // const beacon = await client.readDataFeedWithId(expectedId);
    // expect([...beacon.value]).to.equal(expectedValue);
    // expect(beacon.timestamp).to.equal(expectedTimestamp);
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

    try {
      await client.updateBeaconSetWithSignedData(
        [airnodeAddress, airnodeAddress, airnodeAddress],
        beaconSetTemplateIds,
        [0, timestamp-5, timestamp-5],
        [[], [...data1], [...data2]],
        [[], [...signature1], [...signature2]]
      );
    } catch (e) {
      ensure(e.toString().includes("UpdatedValueOutdated"));
    }
}

async function lengthNotCorrect(client, signer, airnodeAddress, beaconSetTemplateIds) {
  const timestamp = currentTimestamp();

  const data = Buffer.alloc(21, 0);
  const signature = await signer.sign(prepareMessage(beaconSetTemplateIds[1], timestamp, data));

  await client.updateBeaconSetWithSignedData(
    [airnodeAddress, airnodeAddress],
    [beaconSetTemplateIds[0], beaconSetTemplateIds[1]],
    [0, timestamp],
    [[], [...data]],
    [[], [...signature.signature]]
  );
}

async function notAllSignaturesValid(client, signer, airnodeAddress, beaconSetTemplateIds) {
  const timestamp = currentTimestamp();

  const data = Buffer.alloc(21, 0);
  
  try {
    await client.updateBeaconSetWithSignedData(
      [airnodeAddress, airnodeAddress],
      [beaconSetTemplateIds[0], beaconSetTemplateIds[1]],
      [0, timestamp],
      [[], [...data]],
      [[], [...Buffer.alloc(64)]]
    );
  } catch (e) {
    ensure(e.toString().includes("InvalidSignature"));
  } 
}

async function notAllTimestampValid(client, signer, airnodeAddress, beaconSetTemplateIds) {
}

async function lessThanTwoBeacons(client) {
    try {
      await client.updateBeaconSetWithSignedData(
        [airnodeAddress],
        [beaconSetTemplateIds[0]],
        [0],
        [[]],
        [[]]
      );
    } catch (e) {
      ensure(e.toString().includes("LessThanTwoBeacons"));
    }
}

async function parameterLengthMismatch(client, airnodeAddress, beaconSetTemplateIds) {
  try {
    await client.updateBeaconSetWithSignedData(
      [airnodeAddress],
      [beaconSetTemplateIds[0]],
      [0, 123],
      [[]],
      [[]]
    );
  } catch (e) {
    ensure(e.toString().includes("ParameterLengthMismatch"));
  }
}

module.exports = { 
  updatesBeaconSetWithSignedData, updatedSetValueOutdated, lengthNotCorrect,
  lessThanTwoBeacons, notAllSignaturesValid, notAllTimestampValid, parameterLengthMismatch
};