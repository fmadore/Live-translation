//! The session's names and terms as Whisper's decoder prompt. Whisper takes no vocabulary list:
//! text it is told came before a window is what steers that window's spelling of names and
//! jargon. Pure apart from the tokenizer passed in, so the budget and the echo check are tested
//! without a model.

use super::reconcile::normalise;

/// A prompt for one source, built once when it starts.
#[derive(Debug, Default)]
pub struct Prompt {
    /// Fed to every window with `FullParams::set_tokens`, which, unlike `set_initial_prompt`,
    /// neither tokenizes again per window nor leaks a C string per call.
    pub tokens: Vec<i32>,
    /// The terms that made it in, in order and normalised, for `is_echo`.
    terms: Vec<String>,
}

impl Prompt {
    /// The first terms of `glossary`, in the operator's order, that fit `budget` tokens as one
    /// comma-separated list. whisper.cpp keeps the *last* tokens of a prompt that is too long;
    /// stopping at the first term that does not fit keeps the top of the list instead. A term
    /// the tokenizer refuses ends the list there too.
    pub fn fit(
        glossary: &[String],
        budget: usize,
        tokenize: impl Fn(&str) -> Option<Vec<i32>>,
    ) -> Self {
        let mut prompt = Self::default();
        let mut text = String::new();
        for term in glossary {
            // The tokenizer reads a C string: a NUL would end it early or be refused.
            let term = term.replace('\0', "");
            let candidate = if text.is_empty() {
                term.clone()
            } else {
                format!("{text}, {term}")
            };
            match tokenize(&candidate) {
                Some(tokens) if tokens.len() <= budget => {
                    prompt.tokens = tokens;
                    prompt.terms.push(normalise(&term));
                    text = candidate;
                }
                _ => break,
            }
        }
        prompt
    }

    /// How many of the operator's terms made it in.
    pub fn len(&self) -> usize {
        self.terms.len()
    }

    /// Whether a decoded segment is the prompt read back rather than speech: two or more of its
    /// terms in a row, in the list's order, and nothing else, as Whisper writes over noise it
    /// cannot make out. A term said on its own is kept: that is what the list is for.
    pub fn is_echo(&self, text: &str) -> bool {
        let parts: Vec<String> = text
            .split([',', '、', '，', '،'])
            .map(normalise)
            .filter(|part| !part.is_empty())
            .collect();
        parts.len() >= 2
            && self
                .terms
                .windows(parts.len())
                .any(|run| run.iter().eq(parts.iter()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One token per word: enough to count a budget by.
    fn words(text: &str) -> Vec<i32> {
        text.split([' ', ','])
            .filter(|word| !word.is_empty())
            .map(|_| 1)
            .collect()
    }

    fn fit(glossary: &[String], budget: usize) -> Prompt {
        Prompt::fit(glossary, budget, |text| Some(words(text)))
    }

    fn terms(list: &[&str]) -> Vec<String> {
        list.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn the_top_of_the_list_is_kept_when_it_does_not_all_fit() {
        let glossary = terms(&["IWAC", "Université Joseph Ki-Zerbo", "Voxtral", "Bayreuth"]);
        let prompt = fit(&glossary, 5);
        // "IWAC, Université Joseph Ki-Zerbo" is four words; adding Voxtral makes five.
        assert_eq!(prompt.len(), 3);
        assert_eq!(prompt.tokens.len(), 5);
        assert_eq!(fit(&glossary, 100).len(), 4);
    }

    #[test]
    fn nothing_fits_an_empty_budget_and_an_empty_list_has_no_prompt() {
        let prompt = fit(&terms(&["IWAC"]), 0);
        assert!(prompt.tokens.is_empty() && prompt.len() == 0);
        assert!(fit(&[], 100).tokens.is_empty());
    }

    #[test]
    fn a_term_the_tokenizer_refuses_ends_the_list() {
        let glossary = terms(&["IWAC", "refused", "Voxtral"]);
        let prompt = Prompt::fit(&glossary, 100, |text| {
            (!text.contains("refused")).then(|| vec![0; 1])
        });
        assert_eq!(prompt.len(), 1);
    }

    #[test]
    fn nul_characters_never_reach_the_tokenizer() {
        let prompt = Prompt::fit(&terms(&["IW\0AC"]), 100, |text| {
            assert!(!text.contains('\0'));
            Some(words(text))
        });
        assert_eq!(prompt.len(), 1);
    }

    #[test]
    fn the_list_read_back_is_an_echo_and_speech_is_not() {
        let glossary = terms(&["IWAC", "Université Joseph Ki-Zerbo", "Voxtral"]);
        let prompt = fit(&glossary, 100);
        assert!(prompt.is_echo(" IWAC, Université Joseph Ki-Zerbo, Voxtral."));
        assert!(prompt.is_echo("université joseph ki-zerbo, voxtral"));
        // One term on its own is what the speaker said.
        assert!(!prompt.is_echo("IWAC."));
        // Out of the list's order, or with other words, it is speech.
        assert!(!prompt.is_echo("Voxtral, IWAC"));
        assert!(!prompt.is_echo("IWAC, which Voxtral transcribes"));
        assert!(!prompt.is_echo(""));
        assert!(!Prompt::default().is_echo("IWAC, Voxtral"));
    }
}
