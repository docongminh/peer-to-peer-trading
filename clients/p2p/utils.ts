import * as anchor from "@project-serum/anchor";
import {
  getAccount,
  TokenAccountNotFoundError,
  TokenInvalidAccountOwnerError,
} from "@solana/spl-token";
import BN from "bn.js";

export async function findPdaAccount(
  programId: anchor.web3.PublicKey,
  seed: string,
  creator: anchor.web3.PublicKey,
  orderId: number
): Promise<{ address: anchor.web3.PublicKey; bump: number }> {
  const uidBuffer = new BN(orderId).toBuffer("le", 8);
  const [address, bump] = await anchor.web3.PublicKey.findProgramAddress(
    [Buffer.from(seed), creator.toBuffer(), uidBuffer],
    programId
  );
  return {
    address: address,
    bump: bump,
  };
}

export async function encodeTransaction(
  connection: anchor.web3.Connection,
  transaction: anchor.web3.Transaction
): Promise<Buffer> {
  transaction.recentBlockhash = (
    await connection.getLatestBlockhash()
  ).blockhash;
  transaction.feePayer = transaction.instructions[0].keys
    .filter((item) => item.isSigner)
    .map((item) => {
      return item.pubkey;
    })[0];
  return transaction.serialize({ requireAllSignatures: false });
}

export async function isValidTokenAccount(
  connection: anchor.web3.Connection,
  tokenAccount: anchor.web3.PublicKey,
  owner: anchor.web3.PublicKey,
  mintAddress?: anchor.web3.PublicKey
): Promise<boolean> {
  if (!mintAddress && owner.toString() == tokenAccount.toString()) {
    return false;
  }
  try {
    const accountInfo = await getAccount(connection, tokenAccount);
    return (
      accountInfo.mint.toString() == mintAddress.toString() &&
      accountInfo.owner.toString() == owner.toString()
    );
  } catch (error: unknown) {
    if (
      error instanceof TokenAccountNotFoundError ||
      error instanceof TokenInvalidAccountOwnerError
    ) {
      return false;
    } else {
      throw error;
    }
  }
}
