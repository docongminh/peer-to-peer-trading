# Trading-p2p
 
A solana program support user can trade peer to peer solana token between 2 users

Notes: Because program designed and developed in May, 2022, so, have some piece of code outdated with latest version.

# Overview

  - [Context](#context)
  - [Logic](#logic)
  - [How to use ?](#how-to-use-)
  - [Run example](#clients-example-in-here-has-wallet-and-config-already-to-example-client)
  

# Context
  - In some bussiness cases, out token has not had ido, or listed in any DEX. Pool liquidity has not provide. So, user can not trade or swap token.
  In this case, trading p2p program support user can create a deal to trade token between specify partner or any user has demand trade with this token.
  - Example use case:
    As a discord user in any community want to trade token with another discord user.

# Logic
  - In this program. Currently supporting 3 types of trade [here](https://github.com/docongminh/trading-p2p/blob/master/programs/trade-p2p/src/state.rs#L6-L32):
    - **Token - Token**: User A has ***Token A*** and want to trade ***x amount*** with someone to receive ***y Token B amount***
    - **Token - SOL**: User A has ***Token A*** and want to trade ***x Token A amount*** with someone to receive  ***y SOL amount***
    - **SOL - Token**: User A has ***SOL*** and want to trade ***x SOL amount*** with someone to receive ***y Token B amount***
  - Diagram for each trade type
  
   ***Token - Token***
     <p align="center">
       <img width="258" height="182" src="https://github.com/docongminh/trading-p2p/blob/master/resources/splspl.png">
     </p>
     
   ***Token - SOL***
     <p align="center">
       <img width="258" height="182" src="https://github.com/docongminh/trading-p2p/blob/master/resources/splsol.png">
     </p>
     
   ***SOL - Token***
     <p align="center">
       <img width="258" height="182" src="https://github.com/docongminh/trading-p2p/blob/master/resources/solspl.png">
     </p>


# How to use ?
 - Basically, To create a deal, i designed a class [`TradeP2P`](https://github.com/docongminh/trading-p2p/blob/master/clients/p2p/TradeP2P.ts) that supported all methods related to `create`, `exchange` and `cancel` a deal.
    - Create a trade instance example:
      ```ts
        const rpc =  anchor.web3.clusterApiUrl("devnet")
        const connection = new anchor.web3.Connection(rpc);
        const tradeInstance = new TradeP2P(connection);
      ```
    - Follow up [TradeType](https://github.com/docongminh/trading-p2p/blob/master/clients/p2p/types.ts#L32-L36)
      ```ts
        export enum TradeType {
            SPLSPL,
            SPLSOL,
            SOLSPL,
        }
      ```
  - Create trade:
  
    Follow up [`TradeOrderRequest`](https://github.com/docongminh/trading-p2p/blob/master/clients/p2p/types.ts#L38-L50)
    
       ```ts
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
       ```
        
     - SPL - SPL:
        ```ts
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

           const transcationBuffer = await tradeInstance.createTrade(tradeOrder);
        ```
     - SPL - SOL:
        ```ts
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

          const transactionBuffer = await tradeInstance.createTrade(tradeOrder);
        ```
   
     - SOL - SPL:
        ```ts
            const tradeOrder: TradeOrderRequest = {
             creator: tradeCreator.publicKey,
             orderId: orderId,
             tradeValue: tradeValue,
             receiveValue: receivevalue,
             creatorSendAccount: tradeCreator.publicKey,
             creatorReceiveAccount: creatorReceiveTokenAccount,
             receiveMint: receiveMintAddress,
             timestamp: Date.now().toString(),
             tradeType: TradeType.SOLSPL,
          };

          const transactionBuffer = await tradeInstance.createTrade(tradeOrder);
        ```
     
  - Exchange:
     - P2P SPL - SPL:
        ```ts
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

          const transactionBuffer = await tradeInstance.exchange(tradeInfo, partnerInfo);
        ```
        
     - P2P SPL - SOL:
        ```ts
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

           const transactionBuffer = await tradeInstance.exchange(tradeInfo, partnerInfo);
        
        ```
        
     - P2P SPL - SPL:
        ```ts
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

          const transactionBuffer = await tradeInstance.exchange(tradeInfo, partnerInfo);
        
        ```
     
  - Cancel:
      Follow up [`CancelParams`](https://github.com/docongminh/trading-p2p/blob/master/clients/p2p/types.ts#L66-L71)
    
       ```ts
          export type CancelParams = {
             creator: PublicKey;
             orderId: number;
             creatorSendAccount: PublicKey;
             tradeMint: PublicKey;
             tradeType: TradeType;
          }
       ```
        
     - SPL - SPL:
        ```ts
           const cancelParams: CancelParams = {
             orderId: orderId,
             creator: tradeCreator.publicKey,
             creatorSendAccount: creatorSendTokenAccount,
             tradeType: TradeType.SPLSPL,
             tradeMint: tradeMintAddress,
           };
           const transactionBuffer = await tradeInstance.cancel(cancelParams);
        ```
     - SPL - SOL:
        ```ts
           const cancelParams: CancelParams = {
              orderId: orderId,
              creator: tradeCreator.publicKey,
              creatorSendAccount: creatorSendTokenAccount,
              tradeType: TradeType.SPLSOL,
              tradeMint: tradeMintAddress,
           };
           const transactionBuffer = await tradeInstance.cancel(cancelParams);
        ```
   
     - SOL - SPL:
        ```ts
            const tradeOrder: TradeOrderRequest = {
               creator: tradeCreator.publicKey,
               orderId: orderId,
               tradeValue: tradeValue,
               receiveValue: receivevalue,
               creatorSendAccount: tradeCreator.publicKey,
               creatorReceiveAccount: creatorReceiveTokenAccount,
               receiveMint: receiveMintAddress,
               timestamp: Date.now().toString(),
               tradeType: TradeType.SOLSPL,
            };

            const transactionBuffer = await tradeInstance.createTrade(tradeOrder);
        ``` 
   ## clients example in [here](https://github.com/docongminh/trading-p2p/tree/master/clients) has wallet and config already to example client
   - run create and exchange SPL-SPL:
     
       ```bash
          npx ts-node clients/splspl.ts
       ```
