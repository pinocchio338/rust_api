import * as anchor from "@project-serum/anchor";
import { 
    bufferU64BE, Datapoint, deriveBeaconId, deriveDApiId,
    deriveDatapointPDA, deriveNameHashPDA, encodeData, keccak256Packed
} from "./utils";
import nacl from 'tweetnacl';
import { createInstructionWithPublicKey, SignatureParam } from "./sig";

export class DapiClient {
    private program: anchor.Program;
    private provider: anchor.Provider;

    constructor(program: anchor.Program, provider: anchor.Provider) {
        this.program = program;
        this.provider = provider;
    }

    /**
     * Reads the data point with ID
     * @param datapointId The datapoint id to read
     * @returns The Datapoint class
     */
    public async readWithDataPointId(datapointId: Buffer): Promise<Datapoint> {
        const pda = await deriveDatapointPDA(datapointId, this.program.programId);
        const wrappedDataPoint = await this.program.account.wrappedDataPoint.fetch(pda);
        return Datapoint.deserialize(wrappedDataPoint.rawDatapoint);
    }

    /**
     * Create a new offline UpdateBeaconWithSignedData transasction
     * 
     * @param templateID 
     * @param timestamp 
     * @param data
     * @param storageFunderKey 
     * @param txnRelayerKey 
     * @returns The serialized offline transaction buffer
     */
    public async newUpdateBeaconWithSignedDataTxn(
        templateID: number,
        timestamp: number,
        data: number,
        storageFunder: anchor.web3.Keypair,
        txnRelayerKey: anchor.web3.PublicKey,
    ): Promise<[Uint8Array, Buffer]> {
        const beaconId = deriveBeaconId(storageFunder.publicKey.toBytes(), templateID);
        const beaconIdPDA = await deriveDatapointPDA(beaconId, this.program.programId);
    
        const bufferedTimestamp = bufferU64BE(timestamp);
        const encodedData = encodeData(data);
      
        const method = this.program.instruction.updateBeaconWithSignedData(
          beaconId,
          bufferU64BE(templateID),
          bufferedTimestamp,
          encodedData,
          {
            accounts: {
              datapoint: beaconIdPDA,
              user: storageFunder.publicKey,
              systemProgram: anchor.web3.SystemProgram.programId,
            }
          }
        );
        
        const tx = new anchor.web3.Transaction().add(method);
        tx.recentBlockhash = (await this.program.provider.connection.getLatestBlockhash()).blockhash;
        tx.feePayer = txnRelayerKey;
      
        const rawTxn = tx.serializeMessage();
        const signature = nacl.sign.detached(rawTxn, storageFunder.secretKey);
        return [signature, rawTxn];
    }

    public async updateDapiWithBeacons(beaconIds: Buffer[], sender: anchor.web3.Keypair) {
      const dataPointId = deriveDApiId(beaconIds);
      const dapiPDA = await deriveDatapointPDA(dataPointId, this.program.programId);

      const remainingAccounts = [];
      for (const b of beaconIds) {
        remainingAccounts.push(
          { isSigner: false, isWritable: false, pubkey: await deriveDatapointPDA(b, this.program.programId) }
        );
      }
      const updateInstruction = await this.program.instruction.updateDapiWithBeacons(
        dataPointId,
        beaconIds,
        {
          accounts: {
            datapoint: dapiPDA,
            user: sender.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          },
          remainingAccounts
        }
      );
  
      const tx = new anchor.web3.Transaction();
      tx.add(updateInstruction);
      await anchor.web3.sendAndConfirmTransaction(
        this.provider.connection,
        tx,
        [sender],
      );
    }

    public async updateDapiWithSignedData(
      airnodes: anchor.web3.PublicKey[],
      templateIds: number[],
      timstamps: number[],
      datas: number[],
      sigWithMessages: SignatureParam[],
      sender: anchor.web3.Keypair
    ) {
      const sigVerify = createInstructionWithPublicKey(sigWithMessages, 0);

      const beaconIds = [];
      for (let i = 0; i < airnodes.length; i++) {
        beaconIds.push(deriveBeaconId(airnodes[i].toBytes(), templateIds[i]));
      }

      const dataPointId = deriveDApiId(beaconIds);
      const dapiPDA = await deriveDatapointPDA(dataPointId, this.program.programId);

      const remainingAccounts = [{ isSigner: false, isWritable: false, pubkey: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY }];
      for (let i = sigWithMessages.length; i < beaconIds.length; i++) {
        const id = beaconIds[i];
        const pda = await deriveDatapointPDA(id, this.program.programId);
        remainingAccounts.push({ isSigner: false, isWritable: false, pubkey: pda });
      }
  
      const updateInstruction = this.program.instruction.updateDapiWithSignedData(
        dataPointId,
        airnodes.map(t => t.toBytes()),
        beaconIds,
        templateIds.map(t => bufferU64BE(t)),
        timstamps.map(t => bufferU64BE(t)),
        datas.map(t => encodeData(t)),
        {
          accounts: {
            datapoint: dapiPDA,
            user: sender.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          },
          remainingAccounts,
        }
      );
  
      const tx = new anchor.web3.Transaction();
      tx.add(sigVerify);
      tx.add(updateInstruction);
  
      await anchor.web3.sendAndConfirmTransaction(
        this.provider.connection,
        tx,
        [sender],
      );
    }

    public deriveBeaconId(airnodeKey: Uint8Array, templateId: number): Buffer {
      return deriveBeaconId(airnodeKey, templateId);
    }

    public deriveDapiId(beaconIds: Buffer[]): Buffer {
      return deriveDApiId(beaconIds);
    }

    public async setName(name: Buffer, datapointId: Buffer, sender: anchor.web3.PublicKey) {
      const nameHash = keccak256Packed(["bytes32"], [name]);
      const nameHashPDA = await deriveNameHashPDA(nameHash, this.program.programId);
      return await this.program.rpc.setName(
        nameHash,
        name,
        datapointId,
        {
          accounts: {
            hash: nameHashPDA,
            user: sender,
            systemProgram: anchor.web3.SystemProgram.programId,
          },
        }
      );
    }

    public async nameToDataPointId(name: Buffer): Promise<Buffer> {
      const nameHash = keccak256Packed(["bytes32"], [name]);
      const nameHashPDA = await deriveNameHashPDA(nameHash, this.program.programId);
      const wrappedDataPointId = await this.program.account.wrappedDataPointId.fetch(nameHashPDA);
      return wrappedDataPointId.datapointId as Buffer;
    }

    public async readWithName(name: Buffer): Promise<Datapoint> {
      const nameHash = keccak256Packed(["bytes32"], [name]);
      const nameHashPDA = await deriveNameHashPDA(nameHash, this.program.programId);
      const wrappedDataPointId = await this.program.account.wrappedDataPointId.fetch(nameHashPDA);
      return this.readWithDataPointId(wrappedDataPointId.datapointId);
    }
}
