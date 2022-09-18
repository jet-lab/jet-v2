import { Address } from "@project-serum/anchor";
import { Connection, PublicKey } from "@solana/web3.js";

export class DerivedAccount {
  public address: PublicKey;
  public bumpSeed: number;

  constructor(address: PublicKey, bumpSeed: number) {
    this.address = address;
    this.bumpSeed = bumpSeed;
  }
}
interface ToBytes {
  toBytes(): Uint8Array;
}

interface HasPublicKey {
  publicKey: PublicKey;
}

type DerivedAccountSeed = HasPublicKey | ToBytes | Uint8Array | string;

export async function findDerivedAccount(
  seeds: DerivedAccountSeed[],
  programId: PublicKey
): Promise<PublicKey> {
  const seedBytes = seeds.map((s) => {
    if (typeof s == "string") {
      return Buffer.from(s);
    } else if ("publicKey" in s) {
      return s.publicKey.toBytes();
    } else if ("toBytes" in s) {
      return s.toBytes();
    } else {
      return s;
    }
  });
  const [address, bumpSeed] = await PublicKey.findProgramAddress(
    seedBytes,
    programId
  );
  return new DerivedAccount(address, bumpSeed).address;
}

export const fetchData = async (
  connection: Connection,
  address: Address
): Promise<Buffer> => {
  let data = (await connection.getAccountInfo(new PublicKey(address)))?.data;
  if (!data) {
    throw "could not fetch account";
  }

  return data;
};

export const logAccounts = ({ ...accounts }) => {
  for (let name in accounts) {
    console.log(name + ": " + accounts[name]);
  }
};
