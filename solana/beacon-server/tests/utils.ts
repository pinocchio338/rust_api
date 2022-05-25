import { ethers } from "ethers";
import * as anchor from "@project-serum/anchor";
import nacl from "tweetnacl";

export async function relayTxn(
    rawTxn: Buffer,
    storageSignature: Uint8Array,
    storageFunderKey: anchor.web3.PublicKey,
    relayer: anchor.web3.Keypair,
  ): Promise<Buffer> {
    const relayerSignature = nacl.sign.detached(rawTxn, relayer.secretKey);
    let recoverTx = anchor.web3.Transaction.populate(anchor.web3.Message.from(rawTxn));
    recoverTx.addSignature(relayer.publicKey, Buffer.from(relayerSignature));
    recoverTx.addSignature(storageFunderKey, Buffer.from(storageSignature));
  
    return recoverTx.serialize();
}

export class Datapoint {
    public value: number;
    public timestamp: number;

    constructor(value: number, timestamp: number) {
        this.timestamp = timestamp;
        this.value = value;
    }

    public static deserialize(bytes: Buffer): Datapoint {
        return new Datapoint(
            Number(BigInt(`0x${bytes.slice(0, 32).toString("hex")}`)),
            Number(bytes.readUInt32BE(32))
        )
    }
}

export function createRawDatapointBuffer(data: number, timestamp: number): Buffer {
    const expected = Buffer.allocUnsafe(36);
    expected.writeBigInt64BE(BigInt(0), 0);
    expected.writeBigInt64BE(BigInt(0), 8);
    expected.writeBigInt64BE(BigInt(0), 16);
    expected.writeBigInt64BE(BigInt(data), 24);
    expected.writeUInt32BE(timestamp, 32);
    return expected;
}

export function prepareMessage(
    templateId: number,
    timestamp: number,
    data: number,
): Buffer {
    const bufferedTemplate = bufferU64BE(templateId);
    const bufferedTimestamp = bufferU64BE(timestamp);
    const encodedData = encodeData(data);
    return keccak256Packed(
        ["bytes32", "uint256", "bytes"],
        [bufferedTemplate, bufferedTimestamp, encodedData]
    )
}

export function keccak256Packed(types: string[], data: any[]): Buffer {
    let hex = ethers.utils.solidityPack(types, data).substr(2); // remove starting "0x"
    const buf = Buffer.from(hex, "hex");
    hex = ethers.utils.keccak256(buf).substr(2); // remove starting "0x"
    return Buffer.from(hex, "hex");
}

export function deriveBeaconId(airnodeKey: Uint8Array, templateId: number): Buffer {
    const bufferedTemplate = bufferU64BE(templateId);
    return keccak256Packed(["bytes", "bytes32"], [airnodeKey, bufferedTemplate]);
    // let hex = ethers.utils.solidityPack(["bytes", "bytes32"], [airnodeKey, templateId]).substr(2); // remove starting "0x"
    // const buf = Buffer.from(hex, "hex");
    // hex = ethers.utils.keccak256(buf).substr(2); // remove starting "0x"
    // return Buffer.from(hex, "hex");
}

export function deriveDApiId(beaconIds: Buffer[]): Buffer {
    const types = beaconIds.map(_ => "bytes32");
    return keccak256Packed(types, beaconIds);
}

export function encodeData(decodedData: number): Buffer {
    const hex = ethers.utils.defaultAbiCoder.encode(['int256'], [decodedData]);
    return Buffer.from(hex.substr(2), "hex");
}

export function bufferU64BE(value: number): Buffer {
    const buffer = Buffer.alloc(32);
    buffer.writeBigUInt64BE(BigInt(value), 24);
    return buffer;
}

export async function deriveDatapointPDA(dataPointId: Buffer, programId: anchor.web3.PublicKey): Promise<anchor.web3.PublicKey> {
    const [pda] = await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from(anchor.utils.bytes.utf8.encode("datapoint")),
          dataPointId
        ],
        programId
    );
    return pda;
}

export async function deriveNameHashPDA(nameHash: Buffer, programId: anchor.web3.PublicKey): Promise<anchor.web3.PublicKey> {
    const [pda] = await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from(anchor.utils.bytes.utf8.encode("hashed-name")),
          nameHash
        ],
        programId
    );
    return pda;
}