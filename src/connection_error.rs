use std::{error, fmt};

#[derive(Debug, Clone)]
pub struct ConnectionError {
    pub(crate) message: String,
}

impl fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "¡Error personalizado: {}", self.message)
    }
}

impl error::Error for ConnectionError {}
