"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.logAccounts = exports.fetchData = exports.findDerivedAccount = exports.DerivedAccount = void 0;
const web3_js_1 = require("@solana/web3.js");
class DerivedAccount {
    constructor(address, bumpSeed) {
        this.address = address;
        this.bumpSeed = bumpSeed;
    }
}
exports.DerivedAccount = DerivedAccount;
async function findDerivedAccount(seeds, programId) {
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
    const [address, bumpSeed] = await web3_js_1.PublicKey.findProgramAddress(seedBytes, programId);
    return new DerivedAccount(address, bumpSeed).address;
}
exports.findDerivedAccount = findDerivedAccount;
const fetchData = async (connection, address) => {
    let data = (await connection.getAccountInfo(new web3_js_1.PublicKey(address)))?.data;
    if (!data) {
        throw "could not fetch account";
    }
    return data;
};
exports.fetchData = fetchData;
const logAccounts = ({ ...accounts }) => {
    for (let name in accounts) {
        console.log(name + ": " + accounts[name]);
    }
};
exports.logAccounts = logAccounts;
//# sourceMappingURL=utils.js.map