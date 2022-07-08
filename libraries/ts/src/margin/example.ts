import { MarginClient } from "./marginClient";
import { MarginAccount } from "./marginAccount";
import { PoolManager } from "./pool";
import { Connection} from '@solana/web3.js';
import { AnchorProvider, Wallet } from '@project-serum/anchor';

//An example of loading margin accounts and getting a margin account's risk indicator

const walletPublicKey = '7cBzzWR78p41zX6VfZC7xJQmtJQKs5hcJDoeiCEpzycd'
const config = MarginClient.getConfig('devnet')
const connection = new Connection('https://api.devnet.solana.com', 'recent')
const options = AnchorProvider.defaultOptions()
const wallet = undefined as any as Wallet
const provider = new AnchorProvider(connection, wallet, options)

const programs = MarginClient.getPrograms(provider, config)
const poolManager = new PoolManager(programs, provider)

//Load margin pools
const pools = await poolManager.loadAll()

//Load wallet tokens
const walletTokens = await MarginAccount.loadTokens(poolManager.programs, walletPublicKey)

//Load all margin accounts - users can have multiple margin accounts eventually
const marginAccounts = await MarginAccount.loadAllByOwner({
    programs: poolManager.programs,
    provider: poolManager.provider,
    pools,
    walletTokens,
    owner: walletPublicKey,
});

//Print risk levels of a margin account
if (marginAccounts) {
  console.log(`Public key 7cBzzWR78p41zX6VfZC7xJQmtJQKs5hcJDoeiCEpzycd risk indicator is ${marginAccounts[0].riskIndicator}`)
} else {
  console.log('We have trouble getting margin accounts')
}

