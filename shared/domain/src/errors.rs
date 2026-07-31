#[derive(Debug)]
pub enum DomainError {
    NotFound,
    InvalidInput(String),
    InternalError(String),
}