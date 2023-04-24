import {
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  PublicKey,
  Keypair,
  Connection,
} from "@solana/web3.js";
import {
  Program,
  BN,
  Idl,
  AnchorProvider,
  Wallet,
} from "@project-serum/anchor";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import {
  TradeType,
  TradeOrderRequest,
  TradeInfo,
  CreateInstructionParams,
  STATE_SEED,
  VAULT_SEED,
  RemainAccount,
  InstructionCreateAccounts,
  PartnerInfo,
  CancelParams,
} from "./types";
import {
  findPdaAccount,
  encodeTransaction,
  isValidTokenAccount,
} from "./utils";
import idl from "../../target/idl/trade_p2p.json";

const FEE_ACCOUNT = new PublicKey(
  "DisXwVm1T6jdajyKX6FoMmSJ98CzCPcWWqUAJ3xUASc9"
);

const PROGRAM_ID = "EJV62xsWEZ5Kbzy7QNR8ogvDDQYqMkdN31UyCqkeaHDe";
export class TradeP2P {
  private _programId: PublicKey;
  private _program: Program;
  private _connection: Connection;

  constructor(connection: Connection) {
    this._programId = new PublicKey(PROGRAM_ID);
    const provider = new AnchorProvider(
      connection,
      new Wallet(Keypair.generate()),
      { preflightCommitment: "confirmed" }
    );
    this._program = new Program(idl as Idl, this.programId, provider);
    this._connection = connection;
  }

  get programId(): PublicKey {
    return this._programId;
  }

  get program(): Program {
    return this._program;
  }

  async createTrade(tradeOrderRequest: TradeOrderRequest): Promise<Buffer> {
    const { address: stateAccount, bump: _ } = await findPdaAccount(
      this.programId,
      STATE_SEED,
      tradeOrderRequest.creator,
      tradeOrderRequest.orderId
    );
    const { address: vaultAccount, bump: vaultBump } = await findPdaAccount(
      this.programId,
      VAULT_SEED,
      tradeOrderRequest.creator,
      tradeOrderRequest.orderId
    );
    //
    const isCreatorSendTokenAccount = await isValidTokenAccount(
      this._connection,
      tradeOrderRequest.creatorSendAccount,
      tradeOrderRequest.creator,
      tradeOrderRequest.tradeMint
    );
    const isCreatorReceiveTokenAccount = await isValidTokenAccount(
      this._connection,
      tradeOrderRequest.creatorReceiveAccount,
      tradeOrderRequest.creator,
      tradeOrderRequest.receiveMint
    );
    switch (tradeOrderRequest.tradeType) {
      case TradeType.SPLSPL:
        if (!tradeOrderRequest.tradeMint || !tradeOrderRequest.receiveMint) {
          throw new Error(
            "Missing trade mint or receive mint for SPL SPL trade"
          );
        }

        if (!isCreatorSendTokenAccount || !isCreatorReceiveTokenAccount) {
          throw new Error(
            "invalid creatorSendAccount or creatorReceiveAccount for SPLSPL"
          );
        }
        break;

      case TradeType.SPLSOL:
        if (!tradeOrderRequest.tradeMint) {
          throw new Error("Missing trade mint for SPL SOL trade");
        }
        if (tradeOrderRequest.receiveMint) {
          throw new Error("SPLSOL trade do not accept receiveMint");
        }
        if (!isCreatorSendTokenAccount || isCreatorReceiveTokenAccount) {
          throw new Error(
            "invalid creatorSendAccount or creatorReceiveAccount for SPLSOL"
          );
        }
        break;

      case TradeType.SOLSPL:
        if (!tradeOrderRequest.receiveMint) {
          throw new Error("Missing receive mint fro SOL SPL trade");
        }

        if (tradeOrderRequest.tradeMint) {
          throw new Error("SOLSPL trade do not accept tradeMint");
        }

        if (isCreatorSendTokenAccount || !isCreatorReceiveTokenAccount) {
          throw new Error(
            "invalid creatorSendAccount or creatorReceiveAccount for SOLSPL"
          );
        }
        break;

      default:
        throw new Error("Missing trade type");
    }

    const remainingAccounts: RemainAccount[] = [];

    if (tradeOrderRequest.tradeMint) {
      remainingAccounts.push({
        pubkey: tradeOrderRequest.tradeMint,
        isWritable: true,
        isSigner: false,
      });
    }
    if (tradeOrderRequest.receiveMint) {
      remainingAccounts.push({
        pubkey: tradeOrderRequest.receiveMint,
        isWritable: true,
        isSigner: false,
      });
    }

    // setup params instructions
    const params: CreateInstructionParams = {
      orderId: new BN(tradeOrderRequest.orderId),
      specifyPartner: tradeOrderRequest.specifyPartner
        ? tradeOrderRequest.specifyPartner
        : null,
      tradeValue: new BN(tradeOrderRequest.tradeValue),
      receiveValue: new BN(tradeOrderRequest.receiveValue),
      timestamp: new BN(tradeOrderRequest.timestamp),
      vaultBump: new BN(vaultBump),
    };

    // setup accounts for instructions
    const accounts: InstructionCreateAccounts = {
      escrowState: stateAccount,
      escrowVault: vaultAccount,
      creator: tradeOrderRequest.creator,
      creatorSendAccount: tradeOrderRequest.creatorSendAccount,
      creatorReceiveAccount: tradeOrderRequest.creatorReceiveAccount,
      feeAccount: FEE_ACCOUNT,
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      rent: SYSVAR_RENT_PUBKEY,
    };
    const transaction = await this._program.methods
      .createTrade(params)
      .accounts(accounts)
      .remainingAccounts(remainingAccounts)
      .transaction();

    return await encodeTransaction(this._connection, transaction);
  }

  async exchange(
    tradeInfo: TradeInfo,
    partnerInfo: PartnerInfo
  ): Promise<Buffer> {
    if (tradeInfo.creator === partnerInfo.partner) {
      throw new Error("Duplicated creator with partner");
    }

    if (
      tradeInfo.specifyPartner &&
      tradeInfo.specifyPartner !== partnerInfo.partner
    ) {
      throw new Error("Invalid specify partner");
    }
    // get pda accounts
    const { address: stateAccount, bump: stateBump } = await findPdaAccount(
      this.programId,
      STATE_SEED,
      tradeInfo.creator,
      tradeInfo.orderId
    );
    const { address: vaultAccount, bump: vaultBump } = await findPdaAccount(
      this.programId,
      VAULT_SEED,
      tradeInfo.creator,
      tradeInfo.orderId
    );
    //
    const isPartnerSendTokenAccount = await isValidTokenAccount(
      this._connection,
      partnerInfo.partnerSendAccount,
      partnerInfo.partner,
      tradeInfo.receiveMint
    );
    const isPartnerReceiveTokenAccount = await isValidTokenAccount(
      this._connection,
      partnerInfo.partnerReceiveAccount,
      partnerInfo.partner,
      tradeInfo.tradeMint
    );
    switch (tradeInfo.tradeType) {
      case TradeType.SPLSPL:
        if (!tradeInfo.tradeMint || !tradeInfo.receiveMint) {
          throw new Error(
            "Missing trade mint or receive mint for SPL SPL trade"
          );
        }
        if (!isPartnerSendTokenAccount || !isPartnerReceiveTokenAccount) {
          throw new Error(
            "invalid partnerSendAccount or partnerReceiveAccount for SPLSPL"
          );
        }

        break;
      case TradeType.SPLSOL:
        if (!tradeInfo.tradeMint) {
          throw new Error("Missing trade mint for SPL SOL trade");
        }
        if (isPartnerSendTokenAccount || !isPartnerReceiveTokenAccount) {
          throw new Error(
            "invalid partnerSendAccount or partnerReceiveAccount for SPLSOL"
          );
        }
        //
        break;
      case TradeType.SOLSPL:
        if (!tradeInfo.receiveMint) {
          throw new Error("Missing receive mint for SOL SPL trade");
        }
        if (!isPartnerSendTokenAccount || isPartnerReceiveTokenAccount) {
          throw new Error(
            "invalid partnerSendAccount or partnerReceiveAccount for SOLSPL"
          );
        }
        break;
      default:
        throw new Error("Missing trade type");
    }
    const accounts = {
      escrowState: stateAccount,
      escrowVault: vaultAccount,
      creator: tradeInfo.creator,
      partnerSendAccount: partnerInfo.partnerSendAccount,
      partnerReceiveAccount: partnerInfo.partnerReceiveAccount,
      creatorReceiveAccount: tradeInfo.creatorReceiveAccount,
      partner: partnerInfo.partner,
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
    };

    const transaction = await this._program.methods
      .exchange(new BN(tradeInfo.orderId), new BN(stateBump), new BN(vaultBump))
      .accounts(accounts)
      .transaction();
    return await encodeTransaction(this._connection, transaction);
  }

  async cancel(cancelParams: CancelParams): Promise<Buffer> {
    const { address: stateAccount, bump: stateBump } = await findPdaAccount(
      this.programId,
      STATE_SEED,
      cancelParams.creator,
      cancelParams.orderId
    );
    const { address: vaultAccount, bump: vaultBump } = await findPdaAccount(
      this.programId,
      VAULT_SEED,
      cancelParams.creator,
      cancelParams.orderId
    );
    const isCreatorSendTokenAccount = await isValidTokenAccount(
      this._connection,
      cancelParams.creatorSendAccount,
      cancelParams.creator,
      cancelParams.tradeMint
    );
    switch (cancelParams.tradeType) {
      case TradeType.SPLSPL:
      case TradeType.SPLSOL:
        if (!isCreatorSendTokenAccount) {
          throw new Error(
            "Invalid creator send token account with SPLSPL or SPLSOL"
          );
        }
        break;
      case TradeType.SOLSPL:
        if (isCreatorSendTokenAccount) {
          throw new Error("Invalid creator send token account with SOLSPL");
        }
        break;

      default:
        throw new Error("Missing trade type");
    }
    const accounts = {
      escrowState: stateAccount,
      escrowVault: vaultAccount,
      creatorSendAccount: cancelParams.creatorSendAccount,
      creator: cancelParams.creator,
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
    };
    const transaction = await this._program.methods
      .cancel(
        new BN(cancelParams.orderId),
        new BN(stateBump),
        new BN(vaultBump)
      )
      .accounts(accounts)
      .transaction();
    return await encodeTransaction(this._connection, transaction);
  }
}
