import { delay, TradeP2P } from "./p2p";

import * as anchor from "@project-serum/anchor";
import {
  PartnerInfo,
  TradeInfo,
  TradeOrderRequest,
  TradeType,
  CancelParams,
} from "./p2p";
import { setup } from "./setup";

async function createTradeSplSol(
  connection: anchor.web3.Connection,
  orderId: number,
  tradeValue: number,
  receivevalue: number,
  tradeInstance: TradeP2P,
  tradeMintAddress: anchor.web3.PublicKey,
  tradeCreator: anchor.web3.Keypair,
  creatorSendTokenAccount: anchor.web3.PublicKey
): Promise<string> {
  const tradeOrder: TradeOrderRequest = {
    creator: tradeCreator.publicKey,
    orderId: orderId,
    tradeValue: tradeValue,
    receiveValue: receivevalue,
    creatorReceiveAccount: tradeCreator.publicKey,
    creatorSendAccount: creatorSendTokenAccount,
    tradeMint: tradeMintAddress,
    timestamp: Date.now().toString(),
    tradeType: TradeType.SPLSOL,
  };

  const buffer = await tradeInstance.createTrade(tradeOrder);
  const transaction = anchor.web3.Transaction.from(Buffer.from(buffer));
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
  tradeCreator: anchor.web3.PublicKey,
  partner: anchor.web3.Keypair,
  creatorSendTokenAccount: anchor.web3.PublicKey,
  partnerReceiveTokenAccount: anchor.web3.PublicKey
): Promise<string> {
  const tradeInfo: TradeInfo = {
    orderId: orderId,
    creator: tradeCreator,
    creatorReceiveAccount: tradeCreator,
    creatorSendAccount: creatorSendTokenAccount,
    tradeType: TradeType.SPLSOL,
    tradeMint: tradeMintAddress,
  };

  const partnerInfo: PartnerInfo = {
    partner: partner.publicKey,
    partnerSendAccount: partner.publicKey,
    partnerReceiveAccount: partnerReceiveTokenAccount,
  };

  const rawTransaction = await tradeInstance.exchange(tradeInfo, partnerInfo);

  const transaction = anchor.web3.Transaction.from(Buffer.from(rawTransaction));
  const signature = await connection.sendTransaction(transaction, [partner]);
  return signature;
}

async function cancelTradeSplSol(
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
    tradeType: TradeType.SPLSOL,
    tradeMint: tradeMintAddress,
  };
  const cancelRawTransaction = await tradeInstance.cancel(cancelParams);
  const cancelTransaction = anchor.web3.Transaction.from(
    Buffer.from(cancelRawTransaction)
  );
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
    tradeCreator,
    creatorTokenATokenAccount,
    peerUser,
    peerUserTokenATokenAccount,
  } = await setup();
  const orderId = Math.floor(Math.random() * 10000);
  const tradeValue = 10;
  const receivevalue = 0.1 * anchor.web3.LAMPORTS_PER_SOL;
  const signature = await createTradeSplSol(
    connection,
    orderId,
    tradeValue,
    receivevalue,
    tradeInstance,
    tokenA,
    tradeCreator,
    creatorTokenATokenAccount
  );

  console.log("signature create trade SPL-SOL: ", signature);

  await delay(20000);
  /// Exchange

  const exchangeSig = await exchange(
    connection,
    orderId,
    tradeInstance,
    tokenA,
    tradeCreator.publicKey,
    peerUser,
    creatorTokenATokenAccount,
    peerUserTokenATokenAccount
  );
  console.log("signature exchange trade SPL - SOL: ", exchangeSig);
})();
