use elasticsearch::http::transport::Transport;
use instrumentation_score_rs::{backends::elasticsearch::ElasticsearchBackend, calculate_score};

#[test]
fn elasticsearch_score() {
    let transport =
        Transport::single_node("https://test-otel-cee089.es.us-east-1.aws.elastic.cloud:443")
            .unwrap();
    transport.set_auth(elasticsearch::auth::Credentials::EncodedApiKey(
        "Zl9ZbVFaZ0JCdFdZUGVDNS1OYUo6MGV5SWF1UWZ3Wnh0VlVTQWQ2akN5QQ==".to_string(),
    ));
    assert_eq!(
        calculate_score(&ElasticsearchBackend::new(&transport).unwrap()).unwrap(),
        100.0
    )
}
