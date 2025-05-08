use sje_derive::Decoder;

#[derive(Decoder)]
#[sje(object)]
#[allow(dead_code)]
struct Position {
    #[sje(rename = "s")]
    symbol: String,
    #[sje(rename = "a")]
    amount: u32,
}

#[derive(Decoder)]
#[sje(object)]
#[allow(dead_code)]
struct PositionUpdate {
    #[sje(rename = "t")]
    timestamp: u64,
    #[sje(rename = "u", decoder = true)]
    updates: Vec<Position>,
}

#[test]
fn should_decode_array_of_objects() {
    let json = r#"{"t":1746699621,"u":[{"s":"btcusdt","a":100},{"s":"ethusdt","a":200}]}"#;

    let update = PositionUpdateDecoder::decode(json.as_bytes()).unwrap();
    assert_eq!(2, update.updates_count());

    let mut positions = update.updates().into_iter();

    let position = positions.next().unwrap();
    assert_eq!("btcusdt", position.symbol_as_str());
    assert_eq!(100, position.amount());

    let position = positions.next().unwrap();
    assert_eq!("ethusdt", position.symbol_as_str());
    assert_eq!(200, position.amount());

    assert!(positions.next().is_none());
}
