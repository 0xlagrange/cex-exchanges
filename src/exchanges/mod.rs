pub mod normalized;

#[cfg(feature = "non-us")]
pub mod binance;

#[cfg(feature = "non-us")]
pub mod kucoin;

#[cfg(feature = "non-us")]
pub mod bybit;

#[cfg(feature = "us")]
pub mod coinbase;

#[cfg(feature = "us")]
pub mod okex;

use std::{
    fmt::{Debug, Display},
    str::FromStr
};

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use tokio::{net::TcpStream, sync::mpsc::UnboundedReceiver};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use self::normalized::{
    rest_api::NormalizedRestApiRequest,
    types::{NormalizedCurrency, NormalizedInstrument},
    ws::{CombinedWsMessage, NormalizedWsChannels}
};
#[cfg(feature = "non-us")]
use self::{
    binance::{ws::BinanceWsBuilder, Binance},
    bybit::{ws::BybitWsBuilder, Bybit},
    kucoin::{ws::KucoinWsBuilder, Kucoin}
};
#[cfg(feature = "us")]
use self::{
    coinbase::{ws::CoinbaseWsBuilder, Coinbase},
    okex::{ws::OkexWsBuilder, Okex}
};
use crate::{
    clients::{
        rest_api::{ExchangeApi, RestApiError},
        ws::{MutliWsStream, WsError}
    },
    exchanges::normalized::rest_api::CombinedRestApiResponse
};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord, EnumIter, ValueEnum)]
pub enum CexExchange {
    #[cfg(feature = "us")]
    Coinbase,
    #[cfg(feature = "us")]
    Okex,
    #[cfg(feature = "non-us")]
    Binance,
    #[cfg(feature = "non-us")]
    Kucoin,
    #[cfg(feature = "non-us")]
    Bybit
}

impl CexExchange {
    pub fn vec_all() -> Vec<Self> {
        Self::iter().collect()
    }

    pub(crate) fn build_multistream_ws_from_normalized(
        self,
        map: Vec<NormalizedWsChannels>,
        _exch_currency_proxy: Option<CexExchange>
    ) -> eyre::Result<MutliWsStream> {
        let res = match self {
            #[cfg(feature = "us")]
            CexExchange::Coinbase => CoinbaseWsBuilder::make_from_normalized_map(map)?
                .build_many_packed()?
                .build_multistream_unconnected(),
            #[cfg(feature = "us")]
            CexExchange::Okex => OkexWsBuilder::make_from_normalized_map(
                map,
                #[cfg(not(feature = "non-us"))]
                _exch_currency_proxy.unwrap_or(CexExchange::Coinbase),
                #[cfg(feature = "non-us")]
                _exch_currency_proxy.unwrap_or(CexExchange::Binance)
            )?
            .build_many_packed()?
            .build_multistream_unconnected(),
            #[cfg(feature = "non-us")]
            CexExchange::Binance => BinanceWsBuilder::make_from_normalized_map(map)?
                .build_many_packed()?
                .build_multistream_unconnected(),
            #[cfg(feature = "non-us")]
            CexExchange::Kucoin => KucoinWsBuilder::make_from_normalized_map(map)?
                .build_many_packed()?
                .build_multistream_unconnected(),
            #[cfg(feature = "non-us")]
            CexExchange::Bybit => BybitWsBuilder::make_from_normalized_map(map)?
                .build_many_packed()?
                .build_multistream_unconnected()
        };

        Ok(res)
    }

    pub(crate) fn build_multithreaded_multistream_ws_from_normalized(
        self,
        map: Vec<NormalizedWsChannels>,
        _exch_currency_proxy: Option<CexExchange>,
        handle: tokio::runtime::Handle,
        number_threads: usize
    ) -> eyre::Result<UnboundedReceiver<CombinedWsMessage>> {
        let res = match self {
            #[cfg(feature = "us")]
            CexExchange::Coinbase => CoinbaseWsBuilder::make_from_normalized_map(map)?
                .build_many_packed()?
                .spawn_multithreaded(number_threads, handle),
            #[cfg(feature = "us")]
            CexExchange::Okex => OkexWsBuilder::make_from_normalized_map(
                map,
                #[cfg(not(feature = "non-us"))]
                _exch_currency_proxy.unwrap_or(CexExchange::Coinbase),
                #[cfg(feature = "non-us")]
                _exch_currency_proxy.unwrap_or(CexExchange::Binance)
            )?
            .build_many_packed()?
            .spawn_multithreaded(number_threads, handle),
            #[cfg(feature = "non-us")]
            CexExchange::Binance => BinanceWsBuilder::make_from_normalized_map(map)?
                .build_many_packed()?
                .spawn_multithreaded(number_threads, handle),
            #[cfg(feature = "non-us")]
            CexExchange::Kucoin => KucoinWsBuilder::make_from_normalized_map(map)?
                .build_many_packed()?
                .spawn_multithreaded(number_threads, handle),
            #[cfg(feature = "non-us")]
            CexExchange::Bybit => BybitWsBuilder::make_from_normalized_map(map)?
                .build_many_packed()?
                .spawn_multithreaded(number_threads, handle)
        };

        Ok(res)
    }

    pub async fn get_all_currencies(self) -> Result<Vec<NormalizedCurrency>, RestApiError> {
        let exchange_api = ExchangeApi::new();
        let out = match self {
            #[cfg(feature = "us")]
            CexExchange::Coinbase => exchange_api
                .all_currencies::<Coinbase>()
                .await?
                .normalize()
                .take_currencies()
                .unwrap(),
            #[cfg(feature = "non-us")]
            CexExchange::Binance => exchange_api
                .all_currencies::<Binance>()
                .await?
                .normalize()
                .take_currencies()
                .unwrap(),
            #[cfg(feature = "us")]
            CexExchange::Okex => exchange_api
                .all_currencies::<Okex>()
                .await?
                .normalize()
                .take_currencies()
                .unwrap(),
            #[cfg(feature = "non-us")]
            CexExchange::Kucoin => exchange_api
                .all_currencies::<Kucoin>()
                .await?
                .normalize()
                .take_currencies()
                .unwrap(),
            #[cfg(feature = "non-us")]
            CexExchange::Bybit => exchange_api
                .all_currencies::<Bybit>()
                .await?
                .normalize()
                .take_currencies()
                .unwrap()
        };

        Ok(out)
    }

    pub async fn get_all_instruments(self) -> Result<Vec<NormalizedInstrument>, RestApiError> {
        let exchange_api = ExchangeApi::new();
        let out = match self {
            #[cfg(feature = "us")]
            CexExchange::Coinbase => exchange_api
                .all_instruments::<Coinbase>()
                .await?
                .normalize()
                .take_instruments()
                .unwrap(),
            #[cfg(feature = "non-us")]
            CexExchange::Binance => exchange_api
                .all_instruments::<Binance>()
                .await?
                .normalize()
                .take_instruments()
                .unwrap(),
            #[cfg(feature = "us")]
            CexExchange::Okex => exchange_api
                .all_instruments::<Okex>()
                .await?
                .normalize()
                .take_instruments()
                .unwrap(),
            #[cfg(feature = "non-us")]
            CexExchange::Kucoin => exchange_api
                .all_instruments::<Kucoin>()
                .await?
                .normalize()
                .take_instruments()
                .unwrap(),
            #[cfg(feature = "non-us")]
            CexExchange::Bybit => exchange_api
                .all_instruments::<Bybit>()
                .await?
                .normalize()
                .take_instruments()
                .unwrap()
        };

        Ok(out)
    }
}

impl Display for CexExchange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "us")]
            CexExchange::Coinbase => write!(f, "coinbase"),
            #[cfg(feature = "us")]
            CexExchange::Okex => write!(f, "okex"),
            #[cfg(feature = "non-us")]
            CexExchange::Binance => write!(f, "binance"),
            #[cfg(feature = "non-us")]
            CexExchange::Kucoin => write!(f, "kucoin"),
            #[cfg(feature = "non-us")]
            CexExchange::Bybit => write!(f, "bybit")
        }
    }
}

impl FromStr for CexExchange {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let str = s.to_lowercase();

        match str.as_str() {
            #[cfg(feature = "us")]
            "coinbase" => Ok(CexExchange::Coinbase),
            #[cfg(feature = "us")]
            "okex" => Ok(CexExchange::Okex),
            #[cfg(feature = "non-us")]
            "binance" => Ok(CexExchange::Binance),
            #[cfg(feature = "non-us")]
            "kucoin" => Ok(CexExchange::Kucoin),
            #[cfg(feature = "non-us")]
            "bybit" => Ok(CexExchange::Bybit),
            _ => Err(eyre::ErrReport::msg(format!("'{s}' is not a valid exchange")))
        }
    }
}

#[async_trait::async_trait]
pub trait Exchange: Clone + Default {
    const EXCHANGE: CexExchange;
    type WsMessage: for<'de> Deserialize<'de> + Into<CombinedWsMessage> + Debug;
    type RestApiResult: for<'de> Deserialize<'de> + Into<CombinedRestApiResponse> + Debug;

    async fn make_ws_connection(&self) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, WsError>;

    async fn make_owned_ws_connection(self) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, WsError> {
        self.make_ws_connection().await
    }

    async fn rest_api_call(&self, web_client: &reqwest::Client, api_channel: NormalizedRestApiRequest) -> Result<Self::RestApiResult, RestApiError>;
}
