import { ethers } from "ethers";

export function deriveBeaconId(airnodeKey: Uint8Array, templateId: Buffer): Buffer {
    let hex = ethers.utils.solidityPack(["bytes", "bytes32"], [airnodeKey, templateId]).substr(2); // remove starting "0x"
    const buf = Buffer.from(hex, "hex");
    hex = ethers.utils.keccak256(buf).substr(2); // remove starting "0x"
    return Buffer.from(hex, "hex");
}

export function encodeData(decodedData: number): Buffer {
    const hex = ethers.utils.defaultAbiCoder.encode(['int256'], [decodedData]);
    console.log(hex);
    return Buffer.from(hex.substr(2), "hex");
}