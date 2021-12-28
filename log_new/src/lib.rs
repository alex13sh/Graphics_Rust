mod value;
mod utils;

pub(crate) type MyResult<T=()> = Result<T, Box<dyn std::error::Error>>;
