import {Buffer} from 'buffer';
import * as anchor from '@project-serum/anchor';

type TransactionInstruction = anchor.web3.TransactionInstruction;

const PUBLIC_KEY_BYTES = 32;
const SIGNATURE_BYTES = 64;
const SIGNATURE_OFFSETS_SERIALIZED_SIZE = 14;
// bytemuck requires structures to be aligned
const SIGNATURE_OFFSETS_START = 2;

/**
 * Params for creating an ed25519 instruction using a public key
 */
export type SignatureParam = {
  publicKey: Uint8Array;
  message: Uint8Array;
  signature: Uint8Array;
};

/**
 * Params for creating an ed25519 instruction using a private key
 */
export type CreateEd25519InstructionWithPrivateKeyParams = {
  privateKey: Uint8Array;
  message: Uint8Array;
  instructionIndex?: number;
};

function encodeSignature(
    instructionData: Buffer,
    param: SignatureParam,
    nextPubKeyIndex: number,
    nextSignatureOffsetIndex: number,
    index: number
): [number, number] {
    // calculate the offsets 
    const nextSignatureIndex = nextPubKeyIndex + PUBLIC_KEY_BYTES;
    const nextMessageIndex = nextSignatureIndex + SIGNATURE_BYTES;
    
    // write the offset
    instructionData.writeUint16LE(nextSignatureIndex, nextSignatureOffsetIndex);        // signatureOffset
    instructionData.writeUint16LE(index, nextSignatureOffsetIndex + 2);                 // signatureInstructionIndex
    instructionData.writeUint16LE(nextPubKeyIndex, nextSignatureOffsetIndex + 4);       // nextPubKeyIndex
    instructionData.writeUint16LE(index, nextSignatureOffsetIndex + 6);                 // publicKeyInstructionIndex
    instructionData.writeUint16LE(nextMessageIndex, nextSignatureOffsetIndex + 8);      // messageDataOffset
    instructionData.writeUint16LE(param.message.length, nextSignatureOffsetIndex + 10); // messageDataSize
    instructionData.writeUint16LE(index, nextSignatureOffsetIndex + 12);                // messageInstructionIndex

    instructionData.fill(param.publicKey, nextPubKeyIndex);
    instructionData.fill(param.signature, nextSignatureIndex);
    instructionData.fill(param.message, nextMessageIndex);

    return [
        nextMessageIndex + param.message.length,
        nextSignatureOffsetIndex + SIGNATURE_OFFSETS_SERIALIZED_SIZE
    ];
}

export function createInstructionWithPublicKey(
    params: SignatureParam[],
    instructionIndex?: number,
): TransactionInstruction {
    const dataOffset = SIGNATURE_OFFSETS_START + SIGNATURE_OFFSETS_SERIALIZED_SIZE * params.length;
    let totalSize = dataOffset + (PUBLIC_KEY_BYTES + SIGNATURE_BYTES) * params.length;
    params.forEach(p => totalSize += p.message.length);

    // assert(
    //   publicKey.length === PUBLIC_KEY_BYTES,
    //   `Public Key must be ${PUBLIC_KEY_BYTES} bytes but received ${publicKey.length} bytes`,
    // );

    // assert(
    //   signature.length === SIGNATURE_BYTES,
    //   `Signature must be ${SIGNATURE_BYTES} bytes but received ${signature.length} bytes`,
    // );

    const instructionData = Buffer.alloc(totalSize);
    const numSignatures = params.length;
    const index =
      instructionIndex == null
        ? 0xffff // An index of `u16::MAX` makes it default to the current instruction.
        : instructionIndex;
    
    // write the number of signatures
    instructionData.writeUInt8(numSignatures);
    instructionData.writeUint8(0, 1); // padding byte

    // process all the signatures
    let nextPubKey = dataOffset;
    let nextSignatureOffset = SIGNATURE_OFFSETS_START;
    for (const p of params) {
        [nextPubKey, nextSignatureOffset] = encodeSignature(
            instructionData,
            p,
            nextPubKey,
            nextSignatureOffset,
            index
        );
    }

    return new anchor.web3.TransactionInstruction({
      keys: [],
      programId,
      data: instructionData,
    });
}

const programId: anchor.web3.PublicKey = new anchor.web3.PublicKey(
    'Ed25519SigVerify111111111111111111111111111',
);