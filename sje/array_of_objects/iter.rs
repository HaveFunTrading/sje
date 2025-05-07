use sje_derive::Decoder;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(transparent)]
pub struct Price(f64);

impl FromStr for Price {
    type Err = <f64 as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(transparent)]
pub struct Quantity(f64);

impl FromStr for Quantity {
    type Err = <f64 as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

#[derive(Decoder, Debug)]
#[sje(object)]
#[allow(dead_code)]
pub struct L2Update {
    #[sje(rename = "e", len = 11)]
    event_type: String,
    #[sje(rename = "b")]
    bids: Vec<(Price, Quantity)>,
    #[sje(rename = "a")]
    asks: Vec<(Price, Quantity)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_iterate_over_bids_and_asks() {
        let update = L2UpdateDecoder::decode(
            br#"{"e":"depthUpdate","b":[["2.6461","6404.9"],["2.6468","22540.8"]],"a":[["2.6461","6404.9"],["2.6468","22540.8"]]}"#,
        ).unwrap();
        let mut bids = update.bids().into_iter();
        assert_eq!(2, update.bids_count());
        assert_eq!(2, bids.len());
        assert_eq!(Some((Price(2.6461), Quantity(6404.9))), bids.next());
        assert_eq!(1, bids.len());
        assert_eq!(Some((Price(2.6468), Quantity(22540.8))), bids.next());
        assert_eq!(0, bids.len());
        assert_eq!(None, bids.next());
        assert_eq!(0, bids.len());
        let mut asks = update.asks().into_iter();
        assert_eq!(2, update.asks_count());
        assert_eq!(2, asks.len());
        assert_eq!(Some((Price(2.6461), Quantity(6404.9))), asks.next());
        assert_eq!(1, asks.len());
        assert_eq!(Some((Price(2.6468), Quantity(22540.8))), asks.next());
        assert_eq!(0, asks.len());
        assert_eq!(None, asks.next());
        assert_eq!(0, asks.len());

        let update =
            L2UpdateDecoder::decode(br#"{"e":"depthUpdate","b":[["2.6461","6404.9"]],"a":[["2.6461","6404.9"]]}"#)
                .unwrap();
        let mut bids = update.bids().into_iter();
        assert_eq!(1, update.bids_count());
        assert_eq!(1, bids.len());
        assert_eq!(Some((Price(2.6461), Quantity(6404.9))), bids.next());
        assert_eq!(0, bids.len());
        assert_eq!(None, bids.next());
        assert_eq!(0, bids.len());
        let mut asks = update.asks().into_iter();
        assert_eq!(1, update.asks_count());
        assert_eq!(1, asks.len());
        assert_eq!(Some((Price(2.6461), Quantity(6404.9))), asks.next());
        assert_eq!(0, asks.len());
        assert_eq!(None, asks.next());
        assert_eq!(0, asks.len());

        let update = L2UpdateDecoder::decode(br#"{"e":"depthUpdate","b":[],"a":[]}"#).unwrap();
        let mut bids = update.bids().into_iter();
        assert_eq!(0, update.bids_count());
        assert_eq!(0, bids.len());
        assert_eq!(None, bids.next());
        assert_eq!(0, bids.len());
        let mut asks = update.asks().into_iter();
        assert_eq!(0, update.asks_count());
        assert_eq!(0, asks.len());
        assert_eq!(None, asks.next());
        assert_eq!(0, asks.len());
    }

    #[test]
    fn should_convert_to_owned() {
        let update = L2UpdateDecoder::decode(
            br#"{"e":"depthUpdate","b":[["2.6461","6404.9"],["2.6468","22540.8"]],"a":[["2.6461","6404.9"],["2.6468","22540.8"]]}"#,
        ).unwrap();
        let update: L2Update = update.into();
        assert_eq!("depthUpdate", update.event_type);

        let mut bids = update.bids.into_iter();
        assert_eq!(Some((Price(2.6461), Quantity(6404.9))), bids.next());
        assert_eq!(Some((Price(2.6468), Quantity(22540.8))), bids.next());
        assert_eq!(None, bids.next());

        let mut asks = update.asks.into_iter();
        assert_eq!(Some((Price(2.6461), Quantity(6404.9))), asks.next());
        assert_eq!(Some((Price(2.6468), Quantity(22540.8))), asks.next());
        assert_eq!(None, asks.next());
    }
}
