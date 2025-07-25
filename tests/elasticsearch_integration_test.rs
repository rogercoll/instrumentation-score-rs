use instrumentation_score_rs::{backends::elasticsearch::ElasticsearchBackend, calculate_score};

#[test]
fn elasticsearch_score() {
    assert_eq!(
        calculate_score(
            &ElasticsearchBackend::new("http://localhost:9200", "YOUR_API_KEY").unwrap()
        )
        .unwrap(),
        3.0
    )
}
