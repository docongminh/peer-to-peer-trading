import * as anchor from "@project-serum/anchor";
import { getAssociatedTokenAddress } from "@solana/spl-token";
import { TradeP2P } from "./p2p";

export async function setup() {
  const connection = new anchor.web3.Connection(
    anchor.web3.clusterApiUrl("devnet")
  );
  const tradeInstance = new TradeP2P(connection);

  const tradeCreator = anchor.web3.Keypair.fromSecretKey(
    new Uint8Array(require("./keys/userA.json"))
  ) as anchor.web3.Keypair;

  const peerUser = anchor.web3.Keypair.fromSecretKey(
    new Uint8Array(require("./keys/userB.json"))
  ) as anchor.web3.Keypair;

  const tokenA = new anchor.web3.PublicKey(
    "G6BRYMheLQtxD1Fo7Sk8RdnAoXcw9yAmG7R4nvrPu794"
  );

  const tokenB = new anchor.web3.PublicKey(
    "FNpSZFAyMpxegoRe4FJ3AkFDAgSNbyAfRn5u745zESZ2"
  );
  const creatorTokenATokenAccount = await getAssociatedTokenAddress(
    tokenA,
    tradeCreator.publicKey
  );

  const peerUserTokenATokenAccount = await getAssociatedTokenAddress(
    tokenA,
    peerUser.publicKey
  );

  const creatorTokenBTokenAccount = await getAssociatedTokenAddress(
    tokenB,
    tradeCreator.publicKey
  );

  const peerUserTokenBTokenAccount = await getAssociatedTokenAddress(
    tokenB,
    peerUser.publicKey
  );

  return {
    connection,
    tradeInstance,
    tokenA,
    tokenB,
    tradeCreator,
    peerUser,
    creatorTokenATokenAccount,
    creatorTokenBTokenAccount,
    peerUserTokenATokenAccount,
    peerUserTokenBTokenAccount,
  };
}
