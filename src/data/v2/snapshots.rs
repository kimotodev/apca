use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;
use serde_json::from_slice as from_json;
use serde_urlencoded::to_string as to_query;

use crate::data::v2::Feed;
use crate::data::v2::bars::Bar;
use crate::data::v2::quotes::Quote;
use crate::data::v2::trades::Trade;

use crate::data::DATA_BASE_URL;
use crate::util::string_slice_to_str;
use crate::Str;

/// The snapshot endpoint for multiple tickers provides the latest trade,
/// latest quote, minute bar, daily bar, and previous daily bar data for
/// each given ticker symbol.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct Snapshot {
  /// The daily bar data.
  #[serde(rename = "dailyBar")]
  pub daily_bar: Option<Bar>,
  /// The latest quote data.
  #[serde(rename = "latestQuote")]
  pub latest_quote: Option<Quote>,
  /// The latest trade data.
  #[serde(rename = "latestTrade")]
  pub latest_trade: Option<Trade>,
  /// The minute bar data.
  #[serde(rename = "minuteBar")]
  pub minute_bar: Option<Bar>,
  /// The previous daily bar data.
  #[serde(rename = "prevDailyBar")]
  pub prev_daily_bar: Option<Bar>,
}

/// A GET request to be issues to the /v2/stocks/snapshots endpoint.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct GetReq {
  /// The symbols to retrieve the snapshot for.
  #[serde(rename = "symbols", serialize_with = "string_slice_to_str")]
  pub symbols: Vec<String>,
  /// The data feed to use.
  #[serde(rename = "feed")]
  pub feed: Option<Feed>,
  /// The type is non-exhaustive and open to extension.
  #[doc(hidden)]
  #[serde(skip)]
  pub _non_exhaustive: (),
}

/// A helper struct for initializing [`GetReq`] objects.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[allow(missing_copy_implementations)]
pub struct GetReqInit {
  /// See `GetReq::feed`.
  pub feed: Option<Feed>,
  /// The type is non-exhaustive and open to extension.
  #[doc(hidden)]
  pub _non_exhaustive: (),
}

impl GetReqInit {
  /// Create a [`GetReq`] from a `GetReqInit`.
  #[inline]
  pub fn init<I, S>(self, symbols: I) -> GetReq
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
  {
    GetReq {
      symbols: symbols.into_iter().map(S::into).collect(),
      feed: self.feed,
      _non_exhaustive: (),
    }
  }
}

EndpointNoParse! {
  /// The representation of a GET request to the /v2/stocks/snapshots endpoint.
  pub Get(GetReq),
  Ok => Vec<(String, Snapshot)>, [
    /// The last snapshots were retrieved successfully.
    /* 200 */ OK,
  ],
  Err => GetError, [
  /// The provided symbol was invalid or not found or the data feed is
  /// not supported.
  /* 400 */ BAD_REQUEST => InvalidInput,
]

  fn base_url() -> Option<Str> {
    Some(DATA_BASE_URL.into())
  }

  fn path(_input: &Self::Input) -> Str {
    "/v2/stocks/snapshots".into()
  }

  fn query(input: &Self::Input) -> Result<Option<Str>, Self::ConversionError> {
    Ok(Some(to_query(input)?.into()))
  }

  fn parse(body: &[u8]) -> Result<Self::Output, Self::ConversionError> {
    // TODO: Ideally we'd write our own deserialize implementation here
    //       to create a vector right away instead of going through a
    //       BTreeMap.

    /// A helper object for parsing the response to a `Get` request.
    #[derive(Deserialize)]
    struct Response(BTreeMap<String, Snapshot>);

    // We are not interested in the actual `Response` object. Clients
    // can keep track of what symbol they requested a quote for.
    from_json::<Response>(body)
      .map(|response| {
        response.0
          .into_iter()
          .collect()
      })
      .map_err(Self::ConversionError::from)
  }

  fn parse_err(body: &[u8]) -> Result<Self::ApiError, Vec<u8>> {
    from_json::<Self::ApiError>(body).map_err(|_| body.to_vec())
  }
}
