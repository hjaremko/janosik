#[derive(Debug, PartialEq)]
pub enum RunnerError {
    NoInput,
    Timeout,
    NotFound,
    NoOutput,
    Crash,
    Other(String),
}
