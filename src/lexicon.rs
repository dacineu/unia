//! Surface forms, held beside identity rather than inside it.
//!
//! An artifact's address is computed from its skeleton, so the same capability
//! has one address no matter which language describes it. That leaves a question
//! the address cannot answer: *how is this thing said?* This module holds the
//! answer, separately, and maps each address to the phrasings known in each
//! language.
//!
//! # Why this is separate rather than inside the manifest
//!
//! Because identity and vocabulary change at different rates and for different
//! reasons. A capability is fixed by what it can do. Its phrasings grow every time
//! somebody says it a new way, in any language, forever. Folding the phrasings
//! into the identity would make every new phrasing a new artifact, and two
//! creatures that learned the same act in different languages would never meet.
//!
//! # What it makes possible
//!
//! Two creatures can hold the same address and different vocabularies. One can be
//! taught English and the other Romanian, and they can still understand each
//! other, because what they exchange is the address and the address is not
//! linguistic. A creature asked in English renders its answer in whatever language
//! it was given, and says so plainly when it has no rendering to give.
//!
//! # What it does not do
//!
//! It does not translate. There is no model here, and nothing infers that
//! `toarnă porumb` means `pour some kibble`. A form is present because somebody
//! supplied it. That is deliberate: an inferred synonym that turns out to be an
//! antonym is worse than a missing one, and a corpus that cannot distinguish
//! `open` from `close` cannot be repaired by looking harder at it.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

/// A content address.
pub type Address = String;

/// A language tag. Deliberately unvalidated: this crate has no locale database
/// and inventing one would imply a canonicalisation the rest of the system does
/// not perform, so `ro` and `ro-RO` are distinct keys and that is visible rather
/// than silently merged.
pub type Language = String;

/// Every known way of saying a set of things, keyed by address and language.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lexicon {
    /// address -> language -> phrasings
    entries: BTreeMap<Address, BTreeMap<Language, BTreeSet<String>>>,
}

impl Lexicon {
    /// An empty lexicon. Nothing is known, and saying so is a valid answer.
    pub fn new() -> Self {
        Lexicon::default()
    }

    /// Records that `address` can be said as `surface` in `language`.
    ///
    /// Returns whether this added something. Idempotent by construction, because
    /// a lexicon is a set: a phrase recorded twice is still one phrase, and
    /// learning a phrasing must never change the address it belongs to.
    pub fn learn(&mut self, address: &str, language: &str, surface: &str) -> bool {
        let surface = surface.trim();
        if surface.is_empty() {
            return false;
        }
        self.entries
            .entry(address.to_string())
            .or_default()
            .entry(language.to_string())
            .or_default()
            .insert(surface.to_string())
    }

    /// Records a whole set of phrasings for one address and language.
    pub fn learn_all<I, S>(&mut self, address: &str, language: &str, surfaces: I) -> usize
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut added = 0;
        for s in surfaces {
            if self.learn(address, language, s.as_ref()) {
                added += 1;
            }
        }
        added
    }

    /// The phrasings known for an address in a language.
    ///
    /// `None` rather than an empty set when the language is unknown, so a caller
    /// can tell "I know this act but not in your language" from "I have never
    /// heard of this act". Those need different replies and conflating them
    /// produces a confident, wrong answer.
    pub fn surface(&self, address: &str, language: &str) -> Option<&BTreeSet<String>> {
        self.entries.get(address)?.get(language)
    }

    /// Every language this address is known in, sorted.
    pub fn languages(&self, address: &str) -> Vec<&str> {
        self.entries
            .get(address)
            .map(|m| m.keys().map(String::as_str).collect())
            .unwrap_or_default()
    }

    /// Whether the lexicon knows this address at all.
    pub fn knows(&self, address: &str) -> bool {
        self.entries.contains_key(address)
    }

    /// How many addresses are known.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the lexicon is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// What a creature can say about one act.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Utterance {
    /// It knows the act and has a phrasing in the requested language.
    Known(String),
    /// It knows the act, but not in the requested language. The languages it
    /// does have are listed, because "I do not have that" is a much weaker and
    /// less useful answer without them.
    UnknownLanguage {
        /// Languages the act is known in.
        available: Vec<Language>,
    },
    /// It has never heard of the act.
    UnknownAct,
}

/// Renders one act for one audience.
///
/// The three outcomes are kept apart because they are three different facts. A
/// creature that conflates "I do not speak your language" with "I do not know
/// this" is worse than useless to the other side: it will either apologise for
/// knowledge it has or claim knowledge it lacks.
pub fn render(lexicon: &Lexicon, address: &str, language: &str) -> Utterance {
    if !lexicon.knows(address) {
        return Utterance::UnknownAct;
    }
    match lexicon.surface(address, language) {
        Some(set) if !set.is_empty() => {
            // Lowest sort order is deterministic, so two creatures rendering the
            // same act in the same language say the same thing. Anything else
            // would make a conversation unrepeatable for reasons that have
            // nothing to do with the content.
            let phrase = set.iter().next().cloned().unwrap_or_default();
            Utterance::Known(phrase)
        }
        _ => Utterance::UnknownLanguage {
            available: lexicon.languages(address).into_iter().map(str::to_string).collect(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(super) fn bilingual() -> (Lexicon, String) {
        // One act, described in two languages, addressed once.
        let address = "9f2b1c44-0000-4000-8000-000000000001".to_string();
        let mut l = Lexicon::new();
        l.learn(&address, "en", "pour some kibble");
        l.learn(&address, "ro", "toarnă porumb");
        (l, address)
    }

}

#[cfg(test)]
mod describe_learn {
    use super::*;

    #[test]
    fn records_a_phrase_under_its_address_and_language() {
        let mut l = Lexicon::new();
        assert!(l.learn("a", "en", "hello"));
        assert_eq!(l.surface("a", "en").unwrap().len(), 1);
    }

    #[test]
    fn is_idempotent_so_relearning_never_grows_the_set() {
        let mut l = Lexicon::new();
        assert!(l.learn("a", "en", "hello"));
        assert!(!l.learn("a", "en", "hello"), "a repeated phrase adds nothing");
        assert_eq!(l.surface("a", "en").unwrap().len(), 1);
    }

    #[test]
    fn ignores_an_empty_phrase() {
        // An empty string is not a phrasing, and storing it would make a
        // renderer emit nothing while believing it had answered.
        let mut l = Lexicon::new();
        assert!(!l.learn("a", "en", "   "));
        assert!(!l.knows("a"));
    }

    #[test]
    fn records_a_whole_set_and_reports_how_many_were_new() {
        let mut l = Lexicon::new();
        let added = l.learn_all("a", "en", ["one", "two", "three"]);
        assert_eq!(added, 3);
        assert_eq!(l.learn_all("a", "en", ["one", "four"]), 1, "one is new");
    }
}

#[cfg(test)]
mod describe_surface {
    use super::tests::bilingual;
    use super::*;

    #[test]
    fn distinguishes_an_unknown_language_from_an_unknown_address() {
        let (l, a) = bilingual();
        assert!(l.surface(&a, "en").is_some());
        assert!(l.surface(&a, "de").is_none(), "act is known, language is not");
        assert!(l.surface("nope", "en").is_none(), "address is not known at all");
    }

    #[test]
    fn lists_the_languages_an_address_is_known_in() {
        let (l, a) = bilingual();
        assert_eq!(l.languages(&a), vec!["en", "ro"]);
    }
}

#[cfg(test)]
mod describe_render {
    use super::tests::bilingual;
    use super::*;

    #[test]
    fn renders_a_phrase_when_the_language_is_known() {
        let (l, a) = bilingual();
        assert_eq!(render(&l, &a, "ro"), Utterance::Known("toarnă porumb".into()));
    }

    #[test]
    fn reports_an_unknown_language_and_says_what_it_has() {
        // "I do not have that" is much less useful without the list of what it
        // does have, since the other side can simply switch.
        let (l, a) = bilingual();
        let u = render(&l, &a, "de");
        assert_eq!(
            u,
            Utterance::UnknownLanguage {
                available: vec!["en".into(), "ro".into()]
            }
        );
    }

    #[test]
    fn reports_an_unknown_act_distinctly_from_an_unknown_language() {
        // Conflating these would make a creature apologise for knowledge it has,
        // or claim knowledge it lacks.
        let (l, _) = bilingual();
        assert_eq!(render(&l, "never-heard-of-it", "en"), Utterance::UnknownAct);
    }

    #[test]
    fn renders_deterministically_so_a_conversation_is_repeatable() {
        let mut l = Lexicon::new();
        l.learn_all("a", "en", ["zeta", "alpha", "mu"]);
        let first = render(&l, "a", "en");
        let second = render(&l, "a", "en");
        assert_eq!(first, second);
    }
}

#[cfg(test)]
mod describe_conversation {
    use super::tests::bilingual;
    use super::*;

    #[test]
    fn two_creatures_understand_each_other_across_languages() {
        // The whole point. One act, one address, two vocabularies. Neither side
        // needs to know the other's language, because what they exchange is the
        // address and the address is not linguistic.
        let (lex, address) = bilingual();

        // The English creature is asked in English.
        let asked_en = render(&lex, &address, "en");
        assert_eq!(asked_en, Utterance::Known("pour some kibble".into()));

        // The same act is answered in Romanian, from the same address.
        let answered_ro = render(&lex, &address, "ro");
        assert_eq!(answered_ro, Utterance::Known("toarnă porumb".into()));

        // One act, two surfaces, one identity.
        assert_eq!(lex.languages(&address).len(), 2);
    }

    #[test]
    fn a_learned_phrase_never_disturbs_the_address_it_belongs_to() {
        // The address is supplied by the caller and the lexicon does not compute
        // it, so the only way this can fail is if something started deriving one
        // from the other. Growing the vocabulary must leave identity alone.
        let mut l = Lexicon::new();
        l.learn("addr", "en", "first");
        l.learn("addr", "ro", "al doilea");
        l.learn("addr", "de", "das zweite");
        assert_eq!(l.len(), 1, "still one act");
        assert_eq!(l.languages("addr").len(), 3, "now known in three languages");
    }

    #[test]
    fn a_creature_answers_in_the_language_it_was_asked_in() {
        // Asking in Romanian and answering in English would be a bug, not a
        // choice: the point is that the surface follows the audience.
        let (lex, address) = bilingual();
        assert_eq!(
            render(&lex, &address, "ro"),
            Utterance::Known("toarnă porumb".into())
        );
    }
}

#[cfg(test)]
mod crosslingual_identity_tests {
    use super::*;
    use crate::identifiers::DuUuid;
    use serde_json::json;

    fn valve_with(aliases_en: [&str; 3], aliases_ro: [&str; 3]) -> serde_json::Value {
        json!({
            "resource_id": "valve-001",
            "category": "actuator",
            "guidance": "A motorised valve controlling fluid flow.",
            "state_space": {
                "flow_rate": {"type": "float", "range": [0.0, 1.0], "unit": "percentage"}
            },
            "action_primitives": [
                {"id": "emergency_shutdown", "aliases": aliases_en, "params": {},
                 "target_state": "flow_rate = 0.0", "constraints": []},
                {"id": "adjust_flow", "aliases": aliases_ro, "params": {"target": "float"},
                 "target_state": "flow_rate", "constraints": []}
            ]
        })
    }

    #[test]
    fn the_same_capability_has_one_address_in_two_languages() {
        // This is the claim the whole multilingual design rests on, and it fails
        // if identity is computed over the whole manifest rather than its
        // skeleton. Two artifacts stated in different languages are one artifact.
        let en = valve_with(
            ["emergency shutdown", "halt", "stop"],
            ["adjust flow", "set flow", "regulate"],
        );
        let ro = valve_with(
            ["oprit de urgenta", "inchide", "deteneaza"],
            ["regleaza debitul", "ajusteaza", "setare"],
        );

        let a = DuUuid::generate(&en, None).unwrap();
        let b = DuUuid::generate(&ro, None).unwrap();
        assert_eq!(
            a, b,
            "the same capability expressed in Romanian must keep the English address"
        );
    }

    #[test]
    fn learning_a_new_phrase_does_not_change_what_a_thing_is() {
        // A rule, and that same rule after one player used a phrase nobody had
        // used before, must be one artifact. Under whole-body hashing these were
        // two, which is what made convergence fragile in a single language.
        let base = valve_with(
            ["emergency shutdown", "halt", "stop"],
            ["adjust flow", "set flow", "regulate"],
        );
        let mut grown = base.clone();
        grown["action_primitives"][0]["aliases"] =
            json!(["emergency shutdown", "halt", "stop", "cut power"]);

        assert_eq!(
            DuUuid::generate(&base, None).unwrap(),
            DuUuid::generate(&grown, None).unwrap()
        );
    }

    #[test]
    fn a_different_capability_still_has_a_different_address() {
        // Removing the surface must not collapse genuinely different things, or
        // "identity is language-independent" would just mean "identity is
        // useless".
        let two_actions = valve_with(
            ["emergency shutdown", "halt", "stop"],
            ["adjust flow", "set flow", "regulate"],
        );
        let mut one_action = two_actions.clone();
        one_action["action_primitives"] = json!([{
            "id": "emergency_shutdown", "aliases": ["halt"], "params": {},
            "target_state": "", "constraints": []
        }]);
        assert_ne!(
            DuUuid::generate(&two_actions, None).unwrap(),
            DuUuid::generate(&one_action, None).unwrap()
        );
    }

    #[test]
    fn guidance_written_for_a_human_is_not_part_of_identity() {
        // The guidance is prose addressed to a reader and is the most likely
        // thing to be translated, so it must not move the address.
        let mut english = valve_with(
            ["emergency shutdown", "halt", "stop"],
            ["adjust flow", "set flow", "regulate"],
        );
        let mut romanian = english.clone();
        romanian["guidance"] = json!("O supapa motorizata care controleaza debitul.");

        english["guidance"] = json!("A totally different sentence about a valve.");
        assert_eq!(
            DuUuid::generate(&english, None).unwrap(),
            DuUuid::generate(&romanian, None).unwrap()
        );
    }

    #[test]
    fn a_resource_id_never_enters_its_own_identity() {
        // Externalising the id is what stops a resource containing its own
        // identity, and the skeleton drops it for that reason.
        let mut a = valve_with(["halt", "stop", "off"], ["set flow", "set", "regulate"]);
        let mut b = a.clone();
        a["resource_id"] = json!("valve-001");
        b["resource_id"] = json!("something-else-entirely");
        assert_eq!(
            DuUuid::generate(&a, None).unwrap(),
            DuUuid::generate(&b, None).unwrap()
        );
    }

    #[test]
    fn two_creatures_meet_on_an_address_however_each_one_was_taught() {
        // End to end: the English creature and the Romanian creature each supply
        // their own phrasings, both land on one address, and a conversation
        // across the two languages is expressible without either side knowing
        // the other's vocabulary in advance.
        let en = valve_with(
            ["emergency shutdown", "halt", "stop"],
            ["adjust flow", "set flow", "regulate"],
        );
        let ro = valve_with(
            ["oprit de urgenta", "inchide", "deteneaza"],
            ["regleaza debitul", "ajusteaza", "setare"],
        );
        let address = DuUuid::generate(&en, None).unwrap().to_string();
        assert_eq!(address, DuUuid::generate(&ro, None).unwrap().to_string());

        let mut lexicon = Lexicon::new();
        lexicon.learn(&address, "en", "emergency shutdown");
        lexicon.learn(&address, "ro", "oprit de urgenta");

        // The English creature is asked in English...
        assert_eq!(
            render(&lexicon, &address, "en"),
            Utterance::Known("emergency shutdown".into())
        );
        // ...and answers in Romanian, from the same address, to the same act.
        assert_eq!(
            render(&lexicon, &address, "ro"),
            Utterance::Known("oprit de urgenta".into())
        );
        // Neither side invented the other's phrasing; both were supplied.
        assert_eq!(lexicon.len(), 1);
    }
}
