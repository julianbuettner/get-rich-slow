# 🐌 Get Rich Slow

## 🦋 Purpose
This is a stateless Rust HTTP server to track your assets,
portfolios and crypto savings
across multiple trading platforms, blockchains and wallets.

## 🔧 Supported Accounts

- Kraken | _Crypto Exchange_
- Nordigen | _European Banks_
- TODO: Binance | _Crypto Exchange_
- TODO: Etoro | _Stocks, ETFs, Crypto_
- Binance Smart Chain | _Crypto, DeFi_
    - BEP20 Tokens
    - Venus
    - Alpaca Finance
- Avalanche C-Chain | _Crypto, Defi_
    - ERC20 Token
    - BenQi
- TODO: Solana | _Cypto, DeFi_
    - TODO: Solend
- TODO: Polygon | _Cypto, DeFi_


## 🖥️ Example Output
`http://127.0.0.1:8000/`
```json
[
  {
    "name": "DeFi Wallet One",
    "icon": "sun",
    "balance": 242.93645,
    "nominal_yearly_growth": 0.059315205,
    "real_yearly_growth": 0.059315205,
    "assets": [
      {
        "name": "BNB",
        "description": "BSC",
        "nominal_growth": 0,
        "real_growth": 0,
        "units": 0.01087607,
        "unit_price": 234.9
      },
      {
        "name": "BUSD",
        "description": "ibBUSD",
        "nominal_growth": 0.060830735,
        "real_growth": 0.060830735,
        "units": 222.88519,
        "unit_price": 1
      }
    ]
  }
]
```

Open Metrics (Prometheus):  
`http://127.0.0.1:8000/metrics`
```
get_rich_slow_asset {fund="DeFi Wallet One", name="BNB", description="ibBUSD"} 200.554717328
get_rich_slow_growth {fund="DeFi Wallet One", name="BNB", description="ibBUSD"} 22.88519
```
`get_rich_slow_asset` - Dollars.  
`get_rich_slow_growth` - How much more dollars will you
have in one year from now.


`http://127.0.0.1:8000/block`
```json
{
  "Avalanche-C": 879265,
  "BSC": 1387991,
  "Ethereum": 1388869,
  "Moonriver": 117575
}
```

## ⚙️ Config

```yaml
---
# ~/.get-rich-slow.yaml

# ==
# Here you specify all wallets/accounts you have

accounts:
    etoro-1:
      kind: etoro
      api-key: jlmnop6789
    
    kraken-1:
      kind: kraken
      api-key: abcdef
      api-secret: abcd12345

    avalanche-ledger-wallet-1:
      kind: avalance-c
      address: 0xa1b2c3d4

    # Check out nordigen-helper/
    bank-account-1:
      kind: nordigen
      refresh-token: eyJWT
      account-id: uuid

# ==
# Here you bundle wallets/accounts into one fund

funds:
    - name: My long term stock and crypto savings
      icon: money_bag
      accounts:
        - kraken-1

    - name: My DeFi savings
      icon: car
      accounts:
        - avalanche-ledger-wallet-1

# ==
# Here you specify a list of ethereum compatible smart contacts.
# Names are fetched automatically.
# TODO: smart-contracty.yaml

smart-contracts:
    bsc:
        bep20:
            - 0xe9e7CEA3DedcA5984780Bafc599bD69ADd087D56  # BUSD
            - 0x7130d2A12B9BCbFAe4f2634d864A1Ee1Ce3Ead9c  # BTCB
        alpaca:
            - 0x7C9e73d4C71dae564d41F78d56439bB4ba87592f  # ibBUSD
            - 0xd7D069493685A581d27824Fc46EdA46B7EfC0063  # ibBNB
        venus:
            - 0xA07c5b74C9B40447a954e1466938b865b6BBea36  # vBNB
    avalanche-c:
        benqi:
            - 0x5C0401e81Bc07Ca70fAD469b451682c0d747Ef1c  # qiAVAX


# ==
# Here you specify one HTTP node per blockchain

nodes:
    bsc: https://bsc-dataseed1.ninicoin.io
    avalanche-c: https://api.avax.network/ext/bc/C/rpc

# ==
# Here you specify a list of whitelisted clients
# The tokens have to be submitted as header named Auth
#
# tr -dc A-Za-z0-9 </dev/urandom | head -c 20 ; echo ''
# curl -H "Auth: here-is-my-very-long-token" http://127.0.0.1:8080

clients:
    - here-is-my-very-long-token
    - CTeKAMNbZYVBeE4Zlf5z  # ESP32 with tracking display


# ==
# Server port and IP address listening to

port: 8080
listen-address: 127.0.0.1  # To make public use 0.0.0.0

```

## 🐛 Contributing

### 🌳 Pull requests
Feel free to open a pull request for bug fixes or new features.  
Please run `cargo fmt` before requesting.

### ✅ Todos
_Specific todos_
- Kraken support
- Binance support
- Etoro support
- Support multiple nodes per blockchain
- Caching
- Improve growth information
    - Fetch crypto PoS inflation data
    - Fetch USD inflation data
    - Fetch stocks growth average to calculate apr

_Infinite todos_
- [Insert crypto exchange] support
- [Insert blockchain] support
- [Insert defi platform] support

