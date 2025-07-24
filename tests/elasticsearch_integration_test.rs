use instrumentation_score_rs::{backends, calculate_score};

#[test]
fn elasticsearch_score() {
    assert_eq!(
        calculate_score(&backends::elasticsearch::ElasticsearchBackend::new().unwrap()).unwrap(),
        3.0
    )
}
