use std::fmt;

#[derive(Debug, Clone)]
pub struct QueryError {
    pub message: String,
    pub position: usize,
    pub suggestion: Option<String>,
}

impl QueryError {
    pub fn new(message: impl Into<String>, position: usize) -> Self {
        Self {
            message: message.into(),
            position,
            suggestion: None,
        }
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

impl fmt::Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Query Error at pos {}: {}", self.position, self.message)?;
        if let Some(ref sugg) = self.suggestion {
            write!(f, "\nSuggestion: {}", sugg)?;
        }
        Ok(())
    }
}

impl std::error::Error for QueryError {}
