//! The operator's names and terms: speaker names, institutions, acronyms and jargon the speech
//! engine is asked to favour for a whole session. One list in the setup, sent the way each
//! engine takes it: Voxtral's `context_bias`, Gemini Transcribe's `customVocabulary` and local
//! Whisper's decoder prompt (`whisper::prompt`). The translation engines take none.

use anyhow::Result;

/// The list's limits, the same in the setup sheet (`src/lib/glossary.ts`). Voxtral shares its
/// context window between the list and the audio, and Gemini documents its best results at up
/// to 100 terms, so a longer list costs recognition rather than helping it.
pub const MAX_TERMS: usize = 100;
pub const MAX_TERM_CHARS: usize = 200;

/// Reject a list the setup sheet would not have produced.
pub fn validate(terms: &[String]) -> Result<()> {
    anyhow::ensure!(
        terms.len() <= MAX_TERMS,
        "The glossary holds at most {MAX_TERMS} names and terms"
    );
    for term in terms {
        anyhow::ensure!(!term.trim().is_empty(), "A glossary entry is empty");
        anyhow::ensure!(
            term.chars().count() <= MAX_TERM_CHARS,
            "A glossary entry is longer than {MAX_TERM_CHARS} characters"
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_glossary_the_setup_sheet_would_not_make_is_refused() {
        let term = |chars: usize| "é".repeat(chars);
        assert!(validate(&[]).is_ok());
        assert!(validate(&[term(MAX_TERM_CHARS)]).is_ok());
        assert!(validate(&[term(MAX_TERM_CHARS + 1)]).is_err());
        assert!(validate(&["  ".into()]).is_err());
        assert!(validate(&vec!["IWAC".to_string(); MAX_TERMS]).is_ok());
        assert!(validate(&vec!["IWAC".to_string(); MAX_TERMS + 1]).is_err());
    }
}
