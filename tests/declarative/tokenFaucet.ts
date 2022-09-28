import { BN } from '@project-serum/anchor';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import {
  Connection,
  Keypair,
  PublicKey,
  sendAndConfirmTransaction,
  Transaction,
  TransactionInstruction
} from '@solana/web3.js';

export const airdropTokens = async (
  connection: Connection,
  faucetProgramId: PublicKey,
  feePayerAccount: Keypair,
  faucetAddress: PublicKey,
  tokenDestinationAddress: PublicKey,
  amount: BN
) => {
  const pubkeyNonce = await PublicKey.findProgramAddress([Buffer.from('faucet')], faucetProgramId);

  const keys = [
    { pubkey: pubkeyNonce[0], isSigner: false, isWritable: false },
    {
      pubkey: await getMintPubkeyFromTokenAccountPubkey(connection, tokenDestinationAddress),
      isSigner: false,
      isWritable: true
    },
    { pubkey: tokenDestinationAddress, isSigner: false, isWritable: true },
    { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
    { pubkey: faucetAddress, isSigner: false, isWritable: false }
  ];

  const tx = new Transaction().add(
    new TransactionInstruction({
      programId: faucetProgramId,
      data: Buffer.from([1, ...amount.toArray('le', 8)]),
      keys
    })
  );
  const txid = await sendAndConfirmTransaction(connection, tx, [feePayerAccount], {
    skipPreflight: false,
    commitment: 'singleGossip'
  });
  console.log(txid);
};

const getMintPubkeyFromTokenAccountPubkey = async (connection: Connection, tokenAccountPubkey: PublicKey) => {
  try {
    const tokenMintData = (await connection.getParsedAccountInfo(tokenAccountPubkey, 'singleGossip')).value!.data;
    //@ts-expect-error (doing the data parsing into steps so this ignore line is not moved around by formatting)
    const tokenMintAddress = tokenMintData.parsed.info.mint;

    return new PublicKey(tokenMintAddress);
  } catch (err) {
    console.log(err);
    throw new Error(
      'Error calculating mint address from token account. Are you sure you inserted a valid token account address'
    );
  }
};
