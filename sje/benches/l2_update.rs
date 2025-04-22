use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use serde::{Deserialize, Deserializer};
use sje_derive::Decoder;
use std::str::FromStr;

const JSON: &[u8] = br#"{"e":"depthUpdate","E":1739836781765,"T":1739836781757,"s":"XRPUSDT","U":6780157664288,"u":6780157666166,"pu":6780157664112,"b":[["2.6461","6404.9"],["2.6468","22540.8"]],"a":[["2.6582","12708.6"],["2.6588","10898.1"],["2.6611","16595.4"]]}"#;

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(transparent)]
pub struct Price(f64);

impl<'de> Deserialize<'de> for Price {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: &str = Deserialize::deserialize(deserializer)?;
        Ok(Price(value.parse().map_err(serde::de::Error::custom)?))
    }
}

impl FromStr for Price {
    type Err = <f64 as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(transparent)]
pub struct Quantity(f64);

impl<'de> Deserialize<'de> for Quantity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: &str = Deserialize::deserialize(deserializer)?;
        Ok(Quantity(value.parse().map_err(serde::de::Error::custom)?))
    }
}

impl FromStr for Quantity {
    type Err = <f64 as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

#[derive(Decoder, Deserialize)]
#[sje(object)]
#[allow(dead_code)]
pub struct L2Update {
    #[sje(rename = "e", len = 11)]
    #[serde(rename = "e")]
    event_type: String,
    #[sje(rename = "E", len = 13)]
    #[serde(rename = "E")]
    event_time: u64,
    #[sje(rename = "T", len = 13)]
    #[serde(rename = "T")]
    transaction_time: u64,
    #[sje(rename = "s")]
    #[serde(rename = "s")]
    symbol: String,
    #[sje(rename = "U", len = 13)]
    #[serde(rename = "U")]
    first_update_id: u64,
    #[sje(rename = "u", len = 13)]
    #[serde(rename = "u")]
    final_update_id: u64,
    #[sje(rename = "pu", len = 13)]
    #[serde(rename = "pu")]
    previous_final_update_id: u64,
    #[sje(rename = "b")]
    #[serde(rename = "b")]
    bids: Vec<(Price, Quantity)>,
    #[sje(rename = "a")]
    #[serde(rename = "a")]
    asks: Vec<(Price, Quantity)>,
}

fn sje_l2_update_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("sje");
    group.throughput(Throughput::Elements(1));
    group.throughput(Throughput::Bytes(JSON.len() as u64));

    group.bench_function("sje_l2_update", |b| {
        b.iter(|| {
            let l2_update = L2UpdateDecoder::decode(JSON).unwrap();

            assert_eq!(b"depthUpdate", l2_update.event_type_as_slice());
            assert_eq!(b"1739836781765", l2_update.event_time_as_slice());
            assert_eq!(b"1739836781757", l2_update.transaction_time_as_slice());
            assert_eq!(b"XRPUSDT", l2_update.symbol_as_slice());
            assert_eq!(b"6780157664288", l2_update.first_update_id_as_slice());
            assert_eq!(b"6780157666166", l2_update.final_update_id_as_slice());
            assert_eq!(b"6780157664112", l2_update.previous_final_update_id_as_slice());

            let mut bids = l2_update.bids().into_iter();
            assert_eq!((Price(2.6461), Quantity(6404.9)), bids.next().unwrap());
            assert_eq!((Price(2.6468), Quantity(22540.8)), bids.next().unwrap());

            let mut asks = l2_update.asks().into_iter();
            assert_eq!((Price(2.6582), Quantity(12708.6)), asks.next().unwrap());
            assert_eq!((Price(2.6588), Quantity(10898.1)), asks.next().unwrap());
            assert_eq!((Price(2.6611), Quantity(16595.4)), asks.next().unwrap());
        })
    });
}

fn serde_l2_update_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("serde");
    group.throughput(Throughput::Elements(1));
    group.throughput(Throughput::Bytes(JSON.len() as u64));

    group.bench_function("serde_l2_update", |b| {
        b.iter(|| {
            let l2_update: L2Update = serde_json::from_slice(JSON).unwrap();

            assert_eq!("depthUpdate", l2_update.event_type);
            assert_eq!(1739836781765, l2_update.event_time);
            assert_eq!(1739836781757, l2_update.transaction_time);
            assert_eq!("XRPUSDT", l2_update.symbol);
            assert_eq!(6780157664288, l2_update.first_update_id);
            assert_eq!(6780157666166, l2_update.final_update_id);
            assert_eq!(6780157664112, l2_update.previous_final_update_id);

            let mut bids = l2_update.bids.into_iter();
            assert_eq!((Price(2.6461), Quantity(6404.9)), bids.next().unwrap());
            assert_eq!((Price(2.6468), Quantity(22540.8)), bids.next().unwrap());

            let mut asks = l2_update.asks.into_iter();
            assert_eq!((Price(2.6582), Quantity(12708.6)), asks.next().unwrap());
            assert_eq!((Price(2.6588), Quantity(10898.1)), asks.next().unwrap());
            assert_eq!((Price(2.6611), Quantity(16595.4)), asks.next().unwrap());
        })
    });
}

criterion_group!(benches, sje_l2_update_benchmark, serde_l2_update_benchmark);
criterion_main!(benches);
