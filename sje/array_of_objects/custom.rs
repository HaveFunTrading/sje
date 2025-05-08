use sje_derive::Decoder;
use std::str::FromStr;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct Price(u64);

impl FromStr for Price {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse().map_err(|_| ())?))
    }
}

#[derive(Decoder)]
#[sje(object)]
#[allow(dead_code)]
pub struct Trade {
    #[sje(rename = "p", ty = "string")]
    price: Price,
}

#[test]
fn should_parse_custom_field() {
    let json = r#"{"p":"12345"}"#;
    let trade = TradeDecoder::decode(json.as_bytes()).unwrap();
    assert_eq!(&Price(12345), trade.price_as_lazy_field().get_ref().unwrap());
    assert_eq!(Price(12345), trade.price_as_lazy_field().get().unwrap());
    assert_eq!(Price(12345), trade.price());
}
