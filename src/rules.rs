#[macro_export]
macro_rules! define_rule {
    ($name:ident, $impact:ident) => {
        pub trait $name {
            const IMPACT: crate::impact::Impact = crate::impact::Impact::$impact;

            fn name(&self) -> &'static str {
                stringify!($name)
            }

            fn execute(&self) -> Result<bool, Box<dyn std::error::Error>>; // to be implemented by Backend
        }
    };
}
