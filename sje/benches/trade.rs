use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use serde::Deserialize;
use sje_derive::Decoder;

const JSON: &[u8] = br#"{"e":"trade","E":1705085312569,"s":"BTCUSDT","t":3370034463,"p":"43520.00000000","q":"0.00022000","b":24269765071,"a":24269767699,"T":1705085312568,"m":true,"M":true}"#;

#[derive(Decoder, Deserialize)]
#[sje(object)]
#[allow(dead_code)]
pub struct Trade {
    #[sje(rename = "e", len = 5)]
    #[serde(rename = "e")]
    event_type: String,
    #[sje(rename = "E", len = 13)]
    #[serde(rename = "E")]
    event_time: u64,
    #[sje(rename = "s")]
    #[serde(rename = "s")]
    symbol: String,
    #[sje(rename = "t", len = 10)]
    #[serde(rename = "t")]
    trade_id: u64,
    #[sje(rename = "p")]
    #[serde(rename = "p")]
    price: String,
    #[sje(rename = "q")]
    #[serde(rename = "q")]
    quantity: String,
    #[sje(rename = "b", len = 11)]
    #[serde(rename = "b")]
    buyer_order_id: u64,
    #[sje(rename = "a", len = 11)]
    #[serde(rename = "a")]
    seller_order_id: u64,
    #[sje(rename = "T", len = 13)]
    #[serde(rename = "T")]
    transaction_time: u64,
    #[sje(rename = "m")]
    #[serde(rename = "m")]
    is_buyer_maker: bool,
}

fn sje_trade_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("sje");
    group.throughput(Throughput::Elements(1));
    group.throughput(Throughput::Bytes(JSON.len() as u64));

    group.bench_function("sje_trade", |b| {
        b.iter(|| {
            let trade = TradeDecoder::decode(JSON).unwrap();
            assert_eq!(b"trade", trade.event_type_as_slice());
            assert_eq!(1705085312569, trade.event_time());
            assert_eq!(b"BTCUSDT", trade.symbol_as_slice());
            assert_eq!(3370034463, trade.trade_id());
            assert_eq!(b"43520.00000000", trade.price_as_slice());
            assert_eq!(b"0.00022000", trade.quantity_as_slice());
            assert_eq!(24269765071, trade.buyer_order_id());
            assert_eq!(24269767699, trade.seller_order_id());
            assert_eq!(1705085312568, trade.transaction_time());
            assert_eq!(b"true", trade.is_buyer_maker_as_slice());
        })
    });
}

fn serde_trade_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("serde");
    group.throughput(Throughput::Elements(1));
    group.throughput(Throughput::Bytes(JSON.len() as u64));

    group.bench_function("serde_trade", |b| {
        b.iter(|| {
            let trade: Trade = serde_json::from_slice(JSON).unwrap();
            assert_eq!("trade", trade.event_type);
            assert_eq!(1705085312569, trade.event_time);
            assert_eq!("BTCUSDT", trade.symbol);
            assert_eq!(3370034463, trade.trade_id);
            assert_eq!("43520.00000000", trade.price);
            assert_eq!("0.00022000", trade.quantity);
            assert_eq!(24269765071, trade.buyer_order_id);
            assert_eq!(24269767699, trade.seller_order_id);
            assert_eq!(1705085312568, trade.transaction_time);
            assert!(trade.is_buyer_maker);
        })
    });
}

criterion_group!(benches, sje_trade_benchmark, serde_trade_benchmark);
criterion_main!(benches);
