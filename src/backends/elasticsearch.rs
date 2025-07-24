use elasticsearch::{http::transport::Transport, Elasticsearch, Error};
use futures::executor::block_on;
use serde_json::{json, Value};

const OTEL_LOGS_INDEX: &'static str = "jklk";
const OTEL_METRICS_INDEX: &'static str = "jklk";

#[derive(Debug)]
pub struct ElasticsearchBackend {
    client: Elasticsearch,
}

impl ElasticsearchBackend {
    pub fn new() -> Result<Self, Error> {
        let transport = Transport::single_node("http://example.com")?;
        transport.set_auth(elasticsearch::auth::Credentials::EncodedApiKey(
            "hello".to_string(),
        ));
        Ok(Self {
            client: Elasticsearch::new(transport),
        })
    }
}

// implement rules for the Elasticsearch client
macro_rules! elasticsearch_rule {
    ($rule:ident, $query:expr, $index:expr) => {
        impl crate::$rule for ElasticsearchBackend {
            fn execute(&self) -> Result<bool, Box<dyn std::error::Error>> {
                let response = ::futures::executor::block_on(
                    self.client
                        .search(elasticsearch::SearchParts::Index(&[$index]))
                        .body($query)
                        .send(),
                )?;
                let response_body =
                    ::futures::executor::block_on(response.json::<serde_json::Value>())?;
                Ok(response_body["hits"]["total"]["value"]
                    .as_i64()
                    .ok_or::<Box<dyn std::error::Error>>(
                        "could not read total hits from response".into(),
                    )?
                    == 0)
            }
        }
    };
}

elasticsearch_rule!(
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
    OTEL_LOGS_INDEX
);

elasticsearch_rule!(
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
    OTEL_LOGS_INDEX
);

elasticsearch_rule!(
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
            "by_attribute_key": {
                "nested": {
                    "path": "attributes"
                },
                "aggs": {
                    "keys": {
                        "terms": {
                            "field": "attributes.key",
                            "size": 100
                        },
                        "aggs": {
                            "cardinality_values": {
                                "cardinality": {
                                    "field": "attributes.value.keyword"
                                }
                            }
                        }
                    }
                }
            }
        }
    }),
    OTEL_METRICS_INDEX
);
