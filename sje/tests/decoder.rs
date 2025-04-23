use sje_derive::Decoder;

#[derive(Decoder)]
#[sje(object)]
#[allow(dead_code)]
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

#[derive(Decoder, Debug)]
#[sje(object)]
#[allow(dead_code)]
struct ListenKeyExpired {
    #[sje(rename = "e", len = 16, offset = 1)]
    event_type: String,
    #[sje(rename = "E", ty = "string", len = 13, offset = 1)]
    event_time: u64,
    #[sje(rename = "listenKey", offset = 1)]
    listen_key: String,
}

#[cfg(test)]
mod tests {
    use crate::{ListenKeyExpiredDecoder, Trade, TradeDecoder};
    use std::str::from_utf8_unchecked;

    #[test]
    fn should_decode_trade() {
        let trade = TradeDecoder::decode(br#"{"e":"trade","E":1705085312569,"s":"BTCUSDT","t":3370034463,"p":"43520.00000000","q":"0.00022000","b":24269765071,"a":24269767699,"T":1705085312568,"m":true,"M":true}"#).unwrap();
        assert_eq!("trade", trade.event_type());
        assert_eq!("BTCUSDT", unsafe { from_utf8_unchecked(trade.symbol_as_slice()) });
        assert_eq!("BTCUSDT", trade.symbol_as_str());
        assert_eq!("BTCUSDT", trade.symbol());

        let trade: Trade = trade.into();
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
    }

    #[test]
    fn should_decode_listen_key_expired() {
        let listen_key_expired = ListenKeyExpiredDecoder::decode(br#"{"e": "listenKeyExpired","E": "1743606297156","listenKey": "FdffIUjdfd343DtLMw2tKS87iL2HpYRniDWpkoxWCb4fwP2yzJXalBlBNnz471cE"}"#).unwrap();
        assert_eq!("listenKeyExpired", listen_key_expired.event_type());
        assert_eq!(1743606297156, listen_key_expired.event_time());
        assert_eq!("FdffIUjdfd343DtLMw2tKS87iL2HpYRniDWpkoxWCb4fwP2yzJXalBlBNnz471cE", listen_key_expired.listen_key());
    }
}
