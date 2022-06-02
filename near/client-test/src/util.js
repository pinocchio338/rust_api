const ethers = require("ethers");
const { ErrorContext } = require("near-api-js/lib/providers");

function deriveBeaconId(airnodeKey, templateId) {
    return keccak256Packed(["bytes", "bytes32"], [airnodeKey, templateId]);
}

function encodeData(decodedData) {
    const hex = ethers.utils.defaultAbiCoder.encode(['int256'], [decodedData]);
    return Buffer.from(hex.substr(2), "hex");
}

function prepareMessage(
    templateId,
    timestamp,
    data,
) {
    const bufferedTemplate = toBuffer(Buffer.from(templateId, 'hex'));
    const bufferedTimestamp = bufferU64BE(timestamp);
    const encodedData = encodeData(data);
    return keccak256Packed(
        ["bytes32", "uint256", "bytes"],
        [bufferedTemplate, bufferedTimestamp, encodedData]
    )
}

function keccak256Packed(types, data) {
    let hex = ethers.utils.solidityPack(types, data).substr(2); // remove starting "0x"
    const buf = Buffer.from(hex, "hex");
    hex = ethers.utils.keccak256(buf).substr(2); // remove starting "0x"
    return Buffer.from(hex, "hex");
}

function currentTimestamp() {
    return Math.floor(Date.now() / 1000);
}

async function encodeAndSignData(decodedData, requestHash, timestamp, signer) {
    const data = encodeData(decodedData);
    const signature = await signer.sign(prepareMessage(requestHash, timestamp, data));
    return [data, signature.signature];
}

function bufferU64BE(value) {
    const buffer = Buffer.alloc(32);
    buffer.writeBigUInt64BE(BigInt(value), 24);
    return buffer;
}

function toBuffer(ab) {
    const buf = Buffer.alloc(ab.byteLength);
    const view = new Uint8Array(ab);
    for (let i = 0; i < buf.length; ++i) {
        buf[i] = view[i];
    }
    return buf;
}

function generateRandomBytes32() {
    return ethers.utils.randomBytes(32);
}

function deriveDApiId(beaconIds) {
    const types = beaconIds.map(_ => "bytes32");
    return keccak256Packed(types, beaconIds);
}

function ensure(condition) {
    if (!condition) {
        throw new Error("failed test");
    }
}

function array_equals(a, b) {
    if (a.length !== b.length) { return false; }
    for (let i = 0; i < a.length; i++) {
        if (a[i] !== b[i]) { return false; }
    }
    return true;
}

module.exports = {
    keccak256Packed, currentTimestamp, encodeData, prepareMessage,
    generateRandomBytes32, toBuffer, bufferU64BE, encodeAndSignData,
    deriveBeaconId, deriveDApiId, ensure, array_equals
};