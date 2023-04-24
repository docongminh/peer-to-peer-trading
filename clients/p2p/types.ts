import { PublicKey } from "@solana/web3.js";
import BN from "bn.js";

export const STATE_SEED = "state";
export const VAULT_SEED = "vault";

export type RemainAccount = {
  pubkey: PublicKey;
  isWritable: boolean;
  isSigner: boolean;
};
export type InstructionCreateAccounts = {
  escrowState: PublicKey;
  escrowVault: PublicKey;
  creator: PublicKey;
  creatorSendAccount: PublicKey;
  creatorReceiveAccount: PublicKey;
  feeAccount: PublicKey;
  systemProgram: PublicKey;
  tokenProgram: PublicKey;
  rent: PublicKey;
};
export type CreateInstructionParams = {
  orderId: BN;
  specifyPartner?: PublicKey;
  tradeValue: BN;
  receiveValue: BN;
  timestamp: BN;
  vaultBump: BN;
};

export enum TradeType {
  SPLSPL,
  SPLSOL,
  SOLSPL,
}

export type TradeOrderRequest = {
  creator: PublicKey;
  orderId: number;
  specifyPartner?: PublicKey;
  tradeValue: number;
  receiveValue: number;
  creatorSendAccount: PublicKey;
  creatorReceiveAccount: PublicKey;
  tradeMint?: PublicKey;
  receiveMint?: PublicKey;
  timestamp: string;
  tradeType: TradeType;
};


export type TradeInfo = {
  creator: PublicKey;
  creatorSendAccount: PublicKey;
  creatorReceiveAccount: PublicKey;
  orderId: number;
  tradeType: TradeType;
  valueTrade?: number;
  valueReceive?: number;
  specifyPartner?: PublicKey;
  tradeMint?: PublicKey;
  receiveMint?: PublicKey;
};

export type CancelParams = {
  creator: PublicKey;
  orderId: number;
  creatorSendAccount: PublicKey;
  tradeMint: PublicKey;
  tradeType: TradeType;
  

}


export type PartnerInfo = {
  partner: PublicKey;
  partnerSendAccount: PublicKey;
  partnerReceiveAccount: PublicKey;
}