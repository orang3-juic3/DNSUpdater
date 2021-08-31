use std::fmt::{Display, Formatter};
use std::error::Error;

#[derive(Debug)]
pub enum Errors {
    ApiError,
    NoMatchError,
}
impl Display for Errors {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            Errors::ApiError => { write!(f, "`{:?}`: A cloudflare API response was unsuccessful.", self)}
            Errors::NoMatchError => { write!(f, "`{:?}`: No records matched your criteria.", self)}
        }
    }
}
impl Error for Errors {}