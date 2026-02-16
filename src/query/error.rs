use std::fmt;

#[derive(Debug, Clone)]
pub struct QueryError {
    pub message: String,
    pub position: usize,
    pub context: Option<String>,
    pub suggestion: Option<String>,
}

impl QueryError {
    pub fn new(message: impl Into<String>, position: usize) -> Self {
        Self {
            message: message.into(),
            position,
            context: None,
            suggestion: None,
        }
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }
}

impl fmt::Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Query Error at pos {}: {}", self.position, self.message)?;
        if let Some(ref ctx) = self.context {
            write!(f, "\nContext:\n{}", ctx)?;
            // Add visual pointer if position is within context
            // Note: This assumes context is centered around position
            if let Some(line) = ctx.lines().next() {
                if let Some(_p) = line.find('^') {
                    // Marker already exists in context (prepared by helper)
                } else {
                    // Try to calculate marker position assuming ctx is the raw query
                    // But usually, get_context returns a snippet.
                    // Let's modify get_context to include the marker line.
                }
            }
        }
        if let Some(ref sugg) = self.suggestion {
            write!(f, "\nSuggestion: {}", sugg)?;
        }
        Ok(())
    }
}

impl std::error::Error for QueryError {}
