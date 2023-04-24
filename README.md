# Trading-p2p
 
A solana program support user can trade peer to peer solana token between 2 users

Notes: Because program designed and developed in May, 2022, so, have some piece of code outdated with latest version.

# Overview

  - [Context](#context)
  - [Logic](#logic)
  - [How to use ?](#how-to-use-)
  - [Tech notes](#tech-notes)
  

# Context
  - In some bussiness cases, out token has not ido, or listing in any DEX. Pool liquidity has not provide. So, user can not trade or swap token.
  In this case, trading p2p program support user can create a deal to trade token between specify partner or any user has demand trade with this token.
  - Example use case:
    As a discord user in any community want to trade token with another discord user.

# Logic
  - In this program. Currently supporting 3 types of trade [here](https://github.com/docongminh/trading-p2p/blob/master/programs/trade-p2p/src/state.rs#L6-L32):
    - **Token - Token**: User A has ***Token A*** and want to trade with someone to receive ***Token B***
    - **Token - SOL**: User A has ***Token A*** and want to trade with someone to receive ***SOL***
    - **SOL - Token**: User A has ***SOL*** and want to trade with someone to receive ***Token B***


# How to use ?


# Tech notes
