import * as anchor from "@project-serum/anchor";
import { expect } from "chai";
import * as fs from "fs";

describe("beacon-server", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.Provider.env();
  anchor.setProvider(provider);

  const idl = JSON.parse(
    fs.readFileSync("./target/idl/beacon_server.json", "utf8")
  );
  const programId = new anchor.web3.PublicKey("FRoo7m8Sf6ZAirGgnn3KopQymDtujWx818kcnRxzi23b");
  const program = new anchor.Program(idl, programId);

  const beaconID = Buffer.from("0384392".padEnd(64, "0"), "hex");

  it("updateBeaconWithSignedData", async () => {
    const templateID = Buffer.allocUnsafe(32);
    const timestamp = Buffer.allocUnsafe(32);
    const data = Buffer.from(anchor.utils.bytes.utf8.encode("random-test-data"));
    const signature = Buffer.from(anchor.utils.bytes.utf8.encode("random-test-signature"));

    const [beaconIdPDA] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(anchor.utils.bytes.utf8.encode("datapoint")),
        beaconID
      ],
      program.programId
    )

    // TODO: use methods here
    await program.rpc.updateBeaconWithSignedData(
      beaconID,
      templateID,
      timestamp,
      data,
      signature,
      {
        accounts: {
          datapoint: beaconIdPDA,
          user: anchor.getProvider().wallet.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        }
      }
    );

    const wrappedDataPoint = await program.account.wrappedDataPoint.fetch(beaconIdPDA);
    expect(wrappedDataPoint.rawDatapoint).to.deep.eq(data);
  });

  it("updateDapiWithBeacons", async () => {
    const [beaconIdPDA] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(anchor.utils.bytes.utf8.encode("datapoint")),
        beaconID
      ],
      program.programId
    )

    const tempDAPIId = Buffer.from("1".padEnd(64, "0"), "hex");
    const [dapiPDA] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(anchor.utils.bytes.utf8.encode("datapoint")),
        tempDAPIId
      ],
      program.programId
    )

    const tx = await program.rpc.updateDapiWithBeacons(
      tempDAPIId,
      [beaconID],
      {
        accounts: {
          dapi: dapiPDA,
          user: anchor.getProvider().wallet.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        },
        remainingAccounts: [
          { isSigner: false, isWritable: false, pubkey: beaconIdPDA }
        ],
      }
    );

    const wrappedDataPoint = await program.account.wrappedDataPoint.fetch(dapiPDA);
    console.log(JSON.stringify(wrappedDataPoint));
    // expect(wrappedDataPoint.rawDatapoint).to.deep.eq(data);
  });
});
