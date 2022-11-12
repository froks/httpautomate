use std::fmt::{Debug, Display, Formatter};

pub struct AutomateError {
    details: String,
}

impl AutomateError {
    pub fn new(msg: &str) -> AutomateError {
        return AutomateError { details: msg.to_string() };
    }
}

impl Display for AutomateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Debug for AutomateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.details)
    }
}

