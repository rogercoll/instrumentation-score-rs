mod impact;
mod rules;
mod score;

// TODO: move each "backend" under a feature flag
pub mod backends;

macro_rules! calculate_score {
    ($(($name:ident, $impact:ident)),* $(,)?) => {
        $(
            $crate::define_rule!($name, $impact);
        )*

        pub fn calculate_score<B>(backend: &B) -> Result<f64 ,Box<dyn std::error::Error>>
        where
            B: $( $name + )* Sized
        {
            let mut impacts = [(0,0); 4];
            $(
                let impact_pos = match <B as $name>::IMPACT {
                        crate::impact::Impact::Critical => 0,
                        crate::impact::Impact::Important => 1,
                        crate::impact::Impact::Normal => 2,
                        crate::impact::Impact::Low => 3,
                };
                impacts[impact_pos].0 += <B as $name>::is_compliant(backend)? as u8;
                impacts[impact_pos].1 += 1;
            )*


            Ok(score::score(impacts[0], impacts[1], impacts[2], impacts[3]))
        }
    };
}

calculate_score!(
    // https://github.com/instrumentation-score/spec/blob/main/rules/LOG-001.md
    (LOG001, Important),
    // https://github.com/instrumentation-score/spec/blob/main/rules/LOG-002.md
    (LOG002, Important),
    (MET001, Important)
);

#[cfg(test)]
mod tests {

    use super::*;

    struct BackendMock;

    impl LOG001 for BackendMock {
        fn is_compliant(&self) -> Result<bool, Box<dyn std::error::Error>> {
            Ok(true)
        }
    }

    impl LOG002 for BackendMock {
        fn is_compliant(&self) -> Result<bool, Box<dyn std::error::Error>> {
            Ok(true)
        }
    }

    impl MET001 for BackendMock {
        fn is_compliant(&self) -> Result<bool, Box<dyn std::error::Error>> {
            Ok(true)
        }
    }

    #[test]
    fn valid_score_caluclation() {
        let result = calculate_score(&BackendMock {});
        assert_eq!(result.unwrap(), 3.0);
    }
}
