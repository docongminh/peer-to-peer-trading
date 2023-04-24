import * as anchor from "@project-serum/anchor";
import { delay, PartnerInfo, TradeP2P } from "./p2p";
import { PublicKey, Transaction } from "@solana/web3.js";
import { TradeInfo, TradeOrderRequest, TradeType, CancelParams } from "./p2p";
import { setup } from "./setup";

async function createTradeSplSpl(
  connection: anchor.web3.Connection,
  orderId: number,
  tradeValue: number,
  receivevalue: number,
  tradeInstance: TradeP2P,
  tradeMintAddress: anchor.web3.PublicKey,
  receiveMintAddress: anchor.web3.PublicKey,
  tradeCreator: anchor.web3.Keypair,
  creatorSendTokenAccount: anchor.web3.PublicKey,
  creatorReceiveTokenAccount: anchor.web3.PublicKey
): Promise<string> {
  const tradeOrder: TradeOrderRequest = {
    creator: tradeCreator.publicKey,
    orderId: orderId,
    tradeValue: tradeValue,
    receiveValue: receivevalue,
    creatorSendAccount: creatorSendTokenAccount,
    creatorReceiveAccount: creatorReceiveTokenAccount,
    tradeMint: tradeMintAddress,
    receiveMint: receiveMintAddress,
    timestamp: Date.now().toString(),
    tradeType: TradeType.SPLSPL,
  };

  const rawMessage = await tradeInstance.createTrade(tradeOrder);
  const transaction = Transaction.from(Buffer.from(rawMessage));
  const signature = await connection.sendTransaction(transaction, [
    tradeCreator,
  ]);
  return signature;
}

async function exchange(
  connection: anchor.web3.Connection,
  orderId: number,
  tradeInstance: TradeP2P,
  tradeMintAddress: anchor.web3.PublicKey,
  receiveMintAddress: anchor.web3.PublicKey,
  tradeCreator: anchor.web3.PublicKey,
  partner: anchor.web3.Keypair,
  creatorSendTokenAccount: anchor.web3.PublicKey,
  creatorReceiveTokenAccount: anchor.web3.PublicKey,
  partnerSendTokenAccount: anchor.web3.PublicKey,
  partnerReceiveTokenAccount: anchor.web3.PublicKey
): Promise<string> {
  const tradeInfo: TradeInfo = {
    orderId: orderId,
    creator: tradeCreator,
    creatorSendAccount: creatorSendTokenAccount,
    creatorReceiveAccount: creatorReceiveTokenAccount,
    tradeMint: tradeMintAddress,
    receiveMint: receiveMintAddress,
    tradeType: TradeType.SPLSPL,
  };

  const partnerInfo: PartnerInfo = {
    partner: partner.publicKey,
    partnerSendAccount: partnerSendTokenAccount,
    partnerReceiveAccount: partnerReceiveTokenAccount,
  };

  const rawMessage = await tradeInstance.exchange(tradeInfo, partnerInfo);
  const transaction = Transaction.from(Buffer.from(rawMessage));
  const signature = await connection.sendTransaction(transaction, [partner]);
  return signature;
}

async function cancelTradeSplSpl(
  connection: anchor.web3.Connection,
  orderId: number,
  tradeInstance: TradeP2P,
  tradeMintAddress: anchor.web3.PublicKey,
  tradeCreator: anchor.web3.Keypair,
  creatorSendTokenAccount: anchor.web3.PublicKey
): Promise<string> {
  const cancelParams: CancelParams = {
    orderId: orderId,
    creator: tradeCreator.publicKey,
    creatorSendAccount: creatorSendTokenAccount,
    tradeType: TradeType.SPLSPL,
    tradeMint: tradeMintAddress,
  };
  const cancelRawTransaction = await tradeInstance.cancel(cancelParams);
  const cancelTransaction = Transaction.from(Buffer.from(cancelRawTransaction));
  const cancelSignature = await connection.sendTransaction(cancelTransaction, [
    tradeCreator,
  ]);

  return cancelSignature;
}

(async () => {
  const {
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
  } = await setup();
  const orderId = Math.floor(Math.random() * 10000);
  const tradeValue = 10;
  const receivevalue = 1;

  const signature = await createTradeSplSpl(
    connection,
    orderId,
    tradeValue,
    receivevalue,
    tradeInstance,
    tokenA,
    tokenB,
    tradeCreator,
    creatorTokenATokenAccount,
    creatorTokenBTokenAccount
  );

  console.log("signature create trade SPL - SPL: ", signature);

  await delay(20000);
  /// Exchange

  const exchangeSig = await exchange(
    connection,
    orderId,
    tradeInstance,
    tokenA,
    tokenB,
    tradeCreator.publicKey,
    peerUser,
    creatorTokenATokenAccount,
    creatorTokenBTokenAccount,
    peerUserTokenBTokenAccount,
    peerUserTokenATokenAccount
  );
  console.log("signature exchange trade SPL - SPL: ", exchangeSig);
})();
