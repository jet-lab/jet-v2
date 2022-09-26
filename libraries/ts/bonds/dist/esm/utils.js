import { PublicKey } from "@solana/web3.js";
export class DerivedAccount {
    constructor(address, bumpSeed) {
        this.address = address;
        this.bumpSeed = bumpSeed;
    }
}
export async function findDerivedAccount(seeds, programId) {
    const seedBytes = seeds.map((s) => {
        if (typeof s == "string") {
            return Buffer.from(s);
        }
        else if ("publicKey" in s) {
            return s.publicKey.toBytes();
        }
        else if ("toBytes" in s) {
            return s.toBytes();
        }
        else {
            return s;
        }
    });
    const [address, bumpSeed] = await PublicKey.findProgramAddress(seedBytes, programId);
    return new DerivedAccount(address, bumpSeed).address;
}
export const fetchData = async (connection, address) => {
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
//# sourceMappingURL=utils.js.map