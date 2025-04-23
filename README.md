[![Build Status](https://img.shields.io/endpoint.svg?url=https%3A%2F%2Factions-badge.atrox.dev%2Fhavefuntrading%2Fsje%2Fbadge%3Fref%3Dmain&style=flat&label=build&logo=none)](https://actions-badge.atrox.dev/havefuntrading/sje/goto?ref=main)
[![Crates.io](https://img.shields.io/crates/v/sje.svg)](https://crates.io/crates/sje)
[![Documentation](https://docs.rs/sje/badge.svg)](https://docs.rs/sje/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

# sje

Fast JSON deserialisation and serialisation schema based framework.

## Example

Make sure the `derive` feature is enabled.

```toml
sje = { version = "0.0.4", features = ["derive"]}
```

```rust
#[derive(Decoder)]
#[sje(object)]
pub struct Trade {
    #[sje(rename = "e", len = 5)]
    event_type: String,
    #[sje(rename = "E", len = 13)]
    event_time: u64,
    #[sje(rename = "s")]
    symbol: String,
    #[sje(rename = "t", len = 10)]
    trade_id: u64,
    #[sje(rename = "p")]
    price: String,
    #[sje(rename = "q")]
    quantity: String,
    #[sje(rename = "b", len = 11)]
    buyer_order_id: u64,
    #[sje(rename = "a", len = 11)]
    seller_order_id: u64,
    #[sje(rename = "T", len = 13)]
    transaction_time: u64,
    #[sje(rename = "m")]
    is_buyer_maker: bool,
}

let trade = TradeDecoder::decode(br#"{"e":"trade","E":1705085312569,"s":"BTCUSDT","t":3370034463,"p":"43520.00000000","q":"0.00022000","b":24269765071,"a":24269767699,"T":1705085312568,"m":true,"M":true}"#).unwrap();
assert_eq!("trade", trade.event_type_as_str());
assert_eq!(1705085312569, trade.event_time());
```


