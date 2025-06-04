use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use serde::Deserialize;
use sje_derive::Decoder;

const JSON: &[u8] = br#"{"e":"bookTicker","u":6780157666962,"s":"BTCUSDT","b":"95732.60","B":"2.073","a":"95732.70","A":"23.383","T":1739836781773,"E":1739836781774}"#;

#[derive(Decoder, Deserialize)]
#[sje(object)]
#[allow(dead_code)]
pub struct Ticker {
    #[sje(rename = "e", len = 10)]
    #[serde(rename = "e")]
    event_type: String,
    #[sje(rename = "u", len = 13)]
    #[serde(rename = "u")]
    update_id: u64,
    #[sje(rename = "s")]
    #[serde(rename = "s")]
    symbol: String,
    #[sje(rename = "b")]
    #[serde(rename = "b")]
    bid_price: String,
    #[sje(rename = "B")]
    #[serde(rename = "B")]
    bid_qty: String,
    #[sje(rename = "a")]
    #[serde(rename = "a")]
    ask_price: String,
    #[sje(rename = "A")]
    #[serde(rename = "A")]
    ask_qty: String,
    #[sje(rename = "T", len = 13)]
    #[serde(rename = "T")]
    transaction_time: u64,
    #[sje(rename = "E", len = 13)]
    #[serde(rename = "E")]
    event_time: u64,
}

fn sje_ticker_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("sje");
    group.throughput(Throughput::Elements(1));
    group.throughput(Throughput::Bytes(JSON.len() as u64));

    group.bench_function("sje_ticker", |b| {
        b.iter(|| {
            let ticker = TickerDecoder::decode(JSON).unwrap();
            assert_eq!(b"bookTicker", ticker.event_type_as_slice());
            assert_eq!(6780157666962, ticker.update_id());
            assert_eq!(b"BTCUSDT", ticker.symbol_as_slice());
            assert_eq!(b"95732.60", ticker.bid_price_as_slice());
            assert_eq!(b"2.073", ticker.bid_qty_as_slice());
            assert_eq!(b"95732.70", ticker.ask_price_as_slice());
            assert_eq!(b"23.383", ticker.ask_qty_as_slice());
            assert_eq!(1739836781773, ticker.transaction_time());
            assert_eq!(1739836781774, ticker.event_time());
        })
    });
}

fn serde_ticker_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("serde");
    group.throughput(Throughput::Elements(1));
    group.throughput(Throughput::Bytes(JSON.len() as u64));

    group.bench_function("serde_ticker", |b| {
        b.iter(|| {
            let ticker: Ticker = serde_json::from_slice(JSON).unwrap();
            assert_eq!("bookTicker", ticker.event_type);
            assert_eq!(6780157666962, ticker.update_id);
            assert_eq!("BTCUSDT", ticker.symbol);
            assert_eq!("95732.60", ticker.bid_price);
            assert_eq!("2.073", ticker.bid_qty);
            assert_eq!("95732.70", ticker.ask_price);
            assert_eq!("23.383", ticker.ask_qty);
            assert_eq!(1739836781773, ticker.transaction_time);
            assert_eq!(1739836781774, ticker.event_time);
        })
    });
}

criterion_group!(benches, sje_ticker_benchmark, serde_ticker_benchmark);
criterion_main!(benches);
