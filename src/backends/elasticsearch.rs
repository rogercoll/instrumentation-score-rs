use elasticsearch::{esql::{Esql, EsqlAsyncQueryGetParts}, http::transport::Transport, Elasticsearch, Error};
use serde_json::json;
use tokio::runtime::{Builder, Runtime};

const OTEL_LOGS_INDEX: &'static str = "logs-*otel-*";
const OTEL_METRICS_INDEX: &'static str = "metrics-*otel-*";

pub struct ElasticsearchBackend<'a> {
    client: Elasticsearch,
    esql_client: Esql<'a>,
    tokio_runtime: Runtime,
}

impl<'a> ElasticsearchBackend<'a> {
    pub fn new(es_transport: &'a Transport) -> Result<Self, Error> {
        Ok(Self {
            client: Elasticsearch::new(es_transport.clone()),
            esql_client: Esql::new(es_transport),
            tokio_runtime: Builder::new_multi_thread()
                .worker_threads(1)
                .enable_all()
                .build()?,
        })
    }
}

// implement rules for the Elasticsearch search client
// JSON Pointer: https://datatracker.ietf.org/doc/html/rfc6901
macro_rules! search_rule {
    ($rule:ident, $query:expr, $index:expr, ($json_pointer:expr, $threshold:expr)) => {
        impl<'a> crate::$rule for ElasticsearchBackend<'a> {
            fn is_compliant(&self) -> Result<bool, Box<dyn std::error::Error>> {
                let response = self.tokio_runtime.block_on(
                    self.client
                        .search(elasticsearch::SearchParts::Index(&[$index]))
                        .body($query)
                        .send(),
                )?;
                let response_body = self
                    .tokio_runtime
                    .block_on(response.json::<serde_json::Value>())?;
                println!("Response: {:?}", response_body);
                Ok(response_body
                    .pointer($json_pointer)
                    .ok_or::<Box<dyn std::error::Error>>(
                        "could not get JSON Pointer from response".into(),
                    )?
                    .as_i64()
                    .ok_or::<Box<dyn std::error::Error>>(
                        "could not read total hits from response".into(),
                    )?
                    < $threshold)
            }
        }
    };
}

// implement rules for the Elasticsearch search client
// JSON Pointer: https://datatracker.ietf.org/doc/html/rfc6901
impl<'a> ElasticsearchBackend<'a> {
            fn is_compliant(&self) -> Result<bool, Box<dyn std::error::Error>> {
                let response = self.tokio_runtime.block_on(
                    self.esql_client.async_query().body(json!({ "query": "from warnings-test | keep *" })).send()
                )?;
                let response_body = self
                    .tokio_runtime
                    .block_on(response.json::<serde_json::Value>())?;
                println!("Response: {:?}", response_body);
                Ok(response_body
                    .pointer($json_pointer)
                    .ok_or::<Box<dyn std::error::Error>>(
                        "could not get JSON Pointer from response".into(),
                    )?
                    .as_i64()
                    .ok_or::<Box<dyn std::error::Error>>(
                        "could not read total hits from response".into(),
                    )?
                    < $threshold)
            }
        }
}

search_rule!(
    LOG001,
    json!({
        "size": 0,
        "query": {
            "bool": {
                "must": [
                    { "term": { "severity.text": "DEBUG" } },
                    { "term": { "deployment.environment.name": "production" } },
                    {
                        "range": {
                            "@timestamp": {
                                "gte": "now-14d/d",
                                "lt": "now/d"
                            }
                        }
                    }
                ]
            }
        },
        "aggs": {
            "debug_log_days": {
                "date_histogram": {
                    "field": "@timestamp",
                    "calendar_interval": "day"
                }
            }
        }
    }),
    OTEL_LOGS_INDEX,
    ("/hits/total/value", 1)
);

search_rule!(
    LOG002,
    json!({
        "size": 0,
        "query": {
            "term": {
                "severity.text": "UNSET"
            }
        },
        "aggs": {
            "by_day": {
                "date_histogram": {
                    "field": "@timestamp",
                    "calendar_interval": "day"
                }
            }
        }
    }),
    OTEL_LOGS_INDEX,
    ("/hits/total/value", 1)
);

search_rule!(
    MET001,
    json!({
      "size": 0,
      "query": {
          "range": {
              "@timestamp": {
                  "gte": "now-1h"
              }
          }
      },
      "aggs": {
          "unique_attribute_keys": {
              "cardinality": {
                  "script": {
                      "lang": "painless",
                      "source": "if (params._source.attributes != null) { return params._source.attributes.keySet(); } return [];"
                  }
              }
          }
      }
    }),
    OTEL_METRICS_INDEX,
    ("/aggregations/unique_attribute_keys/value", 10001)
);

search_rule!(
    MET002,
    json!({
      "size": 0,
      "query": {
        "bool": {
          "must": {
            "range": {
              "@timestamp": {
                "gte": "now-1h"
              }
            }
          },
          "should": [
            { "term": { "unit": "1" }},
            { "term": { "unit": "none" }},
            { "term": { "unit": "" }},
            { "bool": { "must_not": { "exists": { "field": "unit" }}}}
          ],
          "minimum_should_match": 1
        }
      }
    }),
    OTEL_METRICS_INDEX,
    ("/hits/total/value", 1)
);
