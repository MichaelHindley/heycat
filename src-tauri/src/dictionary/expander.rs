// Dictionary expander - applies dictionary expansions to transcription text
// Uses case-insensitive, whole-word matching with regex

use regex::Regex;

use super::DictionaryEntry;

/// Compiled pattern for a single dictionary entry
struct CompiledPattern {
    regex: Regex,
    expansion: String,
}

/// Expander that applies dictionary expansions to text
pub struct DictionaryExpander {
    patterns: Vec<CompiledPattern>,
}

impl DictionaryExpander {
    /// Create a new expander from a list of dictionary entries
    /// Pre-compiles regex patterns for each entry for efficient reuse
    pub fn new(entries: &[DictionaryEntry]) -> Self {
        let patterns = entries
            .iter()
            .filter_map(|entry| {
                // Build case-insensitive, whole-word pattern
                let pattern = format!(r"(?i)\b{}\b", regex::escape(&entry.trigger));
                match Regex::new(&pattern) {
                    Ok(regex) => Some(CompiledPattern {
                        regex,
                        expansion: entry.expansion.clone(),
                    }),
                    Err(e) => {
                        crate::warn!(
                            "Failed to compile regex for trigger '{}': {}",
                            entry.trigger,
                            e
                        );
                        None
                    }
                }
            })
            .collect();

        Self { patterns }
    }

    /// Apply all expansions to the input text
    /// Returns the expanded text, or the original text if no matches
    pub fn expand(&self, text: &str) -> String {
        let mut result = text.to_string();

        for pattern in &self.patterns {
            result = pattern.regex.replace_all(&result, &pattern.expansion).into_owned();
        }

        result
    }
}

#[cfg(test)]
#[path = "expander_test.rs"]
mod tests;
