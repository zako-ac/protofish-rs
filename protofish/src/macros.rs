#[macro_export]
macro_rules! proto_schema {
    ($name:ident) => {
        pub mod $name {
            pub mod schema;
            pub mod transform;
        }
    };
}
