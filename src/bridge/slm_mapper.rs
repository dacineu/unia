//! The intent mapper, with the seam on the right side of the abstraction.
//!
//! # What was wrong
//!
//! The old signature was
//!
//! ```text
//! async fn map_intent_semantic<F>(intent, actions, fetch_fn: F) -> Result<String, String>
//! where F: Fn(String, String) -> Result<String, String>
//! ```
//!
//! and three defects followed from it being written that way.
//!
//! **The answer was never checked against the menu.** The function returned
//! `response.trim()`, so any reply became the action ID. A model that replied
//! *"I'm sorry, I cannot map that"*, or invented `close_valve_v2`, or
//! paraphrased despite being told *"Return ONLY the action ID"* — all of it was
//! a success carrying a string that named nothing. The `available_actions` list
//! was used to *write the prompt* and for nothing else, which is why an off-menu
//! answer surfaced later as a resource that does not exist rather than as a
//! rejected intent.
//!
//! **The injection point was on the wrong side.** `F` received a prompt string
//! and returned a string, so the caller could not express `temperature`,
//! `num_predict`, `model` or streaming — all of which were declared in
//! [`SlmRequest`] and [`SlmOptions`] and **never sent**. And a transport failure
//! was an `Err` while a model declining to answer was an `Ok`, so the type said
//! "the model does not know" and "the network is down" were the same event.
//!
//! **Nothing had ever called it.** `build_prompt` had zero callers and no
//! `SemanticSLM` was constructed anywhere in the tree. This is the same finding
//! as [`crate::slm::MockSlm`]'s fabricated costs: the part that is most declared
//! is the least exercised.
//!
//! # What replaced it
//!
//! - **A refusal is a variant, not an absence.** [`Mapped::Refused`] carries the
//!   model's own words, because a parser that discards an off-menu answer cannot
//!   be debugged and a parser that keeps it can.
//! - **The menu is enforced.** An answer must be *exactly* one of the declared
//!   ids, and every other outcome is a refusal with a reason that distinguishes
//!   "invented an action" from "did not answer at all".
//! - **The seam carries a request and returns a reply,** so what the caller
//!   injects is the thing the endpoint would receive, and temperature and budget
//!   are expressible for the first time.
//!
//! **The exact-match rule is a contract, not a convenience.** `build_prompt`
//! instructs the model to return only the id, so exact match is what the prompt
//! promises. A real integration that wants to be lenient about formatting has to
//! *say so and test it*, because the alternative is a parser that quietly accepts
//! anything, which is the defect this file was rewritten to remove.

use serde::{Deserialize, Serialize};

/// What to send. Previously declared and never sent; now it is the thing the
/// injected transport receives, so a caller can finally set the knobs.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SlmRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
    pub options: SlmOptions,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct SlmOptions {
    /// `0.0` is a claim about the engine that only the engine can make, so this
    /// is a request and the reply is what reports the truth.
    pub temperature: f32,
    pub num_predict: usize,
}

impl Default for SlmOptions {
    fn default() -> Self {
        SlmOptions {
            temperature: 0.0,
            num_predict: 64,
        }
    }
}

/// What came back, before it is judged.
///
/// Distinct from the outcome on purpose: judging is this module's job and the
/// transport's job is to have fetched something.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Reply {
    /// The model the endpoint says answered. Kept so a refusal can name it, since
    /// "no action" and "`close_valve_v2` from model X" are different events.
    pub model: String,
    pub created_at: String,
    /// The raw text, untrimmed. Judgement happens in [`SemanticSLM::map`], and it
    /// happens on exactly what arrived.
    pub response: String,
    /// Tokens the engine reported, if it reported any. `Option`, because many do
    /// not, and a zero would be a claim that it took none.
    pub tokens_used: Option<usize>,
}

/// Why an intent did not map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// The reply named something, and it is not on the menu.
    OffMenu {
        /// What it said, verbatim. A parser that discards this cannot be fixed by
        /// anyone but the person who wrote it.
        offered: String,
    },
    /// The reply was not an action id in any recognisable form.
    NotAnId { text: String },
    /// The transport failed, so no reply exists at all. Separate from the two
    /// above because a network failure and a model declining are different
    /// events and only one of them is the model's fault.
    Transport { reason: String },
}

impl Refusal {
    /// The reply's own words, when there were any.
    pub fn text(&self) -> Option<&str> {
        match self {
            Refusal::OffMenu { offered } | Refusal::NotAnId { text: offered } => Some(offered),
            Refusal::Transport { .. } => None,
        }
    }
}

/// The result of mapping: one action, or a typed reason and the words.
#[derive(Debug, Clone, PartialEq)]
pub enum Mapped {
    /// Exactly one of the declared actions.
    Action { id: String },
    Refused {
        why: Refusal,
        /// The engine that was asked, so a refusal is attributable.
        model: String,
        /// What came back, when anything did. `None` for a transport failure,
        /// which is the only case where there is nothing to quote.
        reply: Option<Reply>,
    },
}

impl Mapped {
    /// The action, if there is one.
    pub fn action(&self) -> Option<&str> {
        match self {
            Mapped::Action { id } => Some(id),
            Mapped::Refused { .. } => None,
        }
    }

    pub fn is_refused(&self) -> bool {
        matches!(self, Mapped::Refused { .. })
    }
}

/// A client for a local Small Language Model.
///
/// Browser-compatible by construction: the transport is injected rather than
/// chosen here, so the same mapper runs natively and in a wasm event loop.
pub struct SemanticSLM {
    pub endpoint: String,
    pub model_name: String,
    /// Sent with every request. Previously unexpressible, which is why
    /// `SlmOptions` was a struct nobody could fill.
    pub options: SlmOptions,
}

impl SemanticSLM {
    pub fn new(endpoint: String, model_name: String) -> Self {
        SemanticSLM {
            endpoint,
            model_name,
            options: SlmOptions::default(),
        }
    }

    /// The prompt, and it names the menu it is answering from.
    pub fn build_prompt(&self, intent: &str, available_actions: &[String]) -> String {
        let actions_list = available_actions.join(", ");
        format!(
            "You are a unia Semantic Mapper. Map the following user intent to EXACTLY ONE of these action IDs: [{}].\n\nIntent: '{}'\n\nReturn ONLY the action ID, nothing else.",
            actions_list, intent
        )
    }

    /// The request, built once so the transport receives a request and not a
    /// string.
    pub fn request(&self, intent: &str, available_actions: &[String]) -> SlmRequest {
        SlmRequest {
            model: self.model_name.clone(),
            prompt: self.build_prompt(intent, available_actions),
            stream: false,
            options: self.options,
        }
    }

    /// Judges a reply against the menu.
    ///
    /// Separated from the transport so the rule is testable without a network,
    /// and so the rule is stated in exactly one place. Every other path into this
    /// module goes through here, which is what makes "an off-menu answer cannot
    /// become an action" a property rather than a hope.
    pub fn judge(&self, reply: Reply, available_actions: &[String]) -> Mapped {
        let text = reply.response.trim().to_string();
        if available_actions.iter().any(|a| *a == text) {
            return Mapped::Action { id: text };
        }
        // An answer that is *nearly* an id is an off-menu answer, not a formatting
        // problem: `close_valve_v2` is the model telling us about an action that
        // does not exist, and reporting it as "not an id" would hide that.
        let looks_like_an_id = !text.is_empty()
            && text
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
            && text
                .chars()
                .next()
                .map(|c| c.is_ascii_alphabetic() || c == '_')
                .unwrap_or(false);
        let why = if looks_like_an_id {
            Refusal::OffMenu { offered: text }
        } else {
            Refusal::NotAnId { text }
        };
        Mapped::Refused {
            why,
            model: reply.model.clone(),
            reply: Some(reply),
        }
    }

    /// Maps an intent, by asking the injected transport.
    ///
    /// `fetch` receives the whole request and returns the whole reply, so a
    /// transport can set a timeout, read a token count, or fail. Its `Err` is a
    /// transport failure and nothing else: a model that declines returns an
    /// `Ok(Reply)` and is then refused *with its words intact*, which is the
    /// distinction the old `Result<String, String>` could not express.
    pub fn map<F>(&self, intent: &str, available_actions: &[String], fetch: F) -> Mapped
    where
        F: FnOnce(SlmRequest) -> Result<Reply, String>,
    {
        match fetch(self.request(intent, available_actions)) {
            Ok(reply) => self.judge(reply, available_actions),
            Err(reason) => Mapped::Refused {
                why: Refusal::Transport { reason },
                model: self.model_name.clone(),
                reply: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    //! The rule under test is "an off-menu answer cannot become an action", and
    //! every case here is a way that could have gone wrong.

    use super::*;

    fn menu() -> Vec<String> {
        vec![
            "emergency_shutdown".into(),
            "read_state".into(),
            "set_value".into(),
        ]
    }

    fn mapper() -> SemanticSLM {
        SemanticSLM::new("http://127.0.0.1:11434".into(), "test-model".into())
    }

    fn reply(text: &str) -> Reply {
        Reply {
            model: "test-model".into(),
            created_at: "2026-09-28T00:00:00Z".into(),
            response: text.into(),
            tokens_used: Some(38),
        }
    }

    /// **The flag: a legitimate answer maps, and it is one of the declared ones.**
    #[test]
    fn an_action_from_the_menu_maps() {
        let m = mapper();
        let out = m.map("shut it down", &menu(), |_| Ok(reply("emergency_shutdown")));
        assert_eq!(out.action(), Some("emergency_shutdown"));
        assert!(!out.is_refused());
    }

    /// **The defect, closed.** The old code returned whatever came back, so an
    /// invented action *was* the action and the failure surfaced as a resource
    /// that does not exist.
    #[test]
    fn an_invented_action_is_a_refusal_and_keeps_what_the_model_said() {
        let m = mapper();
        let out = m.map("close it", &menu(), |_| Ok(reply("close_valve_v2")));
        assert_eq!(out.action(), None, "an invented action is not an action");
        match out {
            Mapped::Refused { why, reply: r, .. } => {
                assert_eq!(
                    why,
                    Refusal::OffMenu {
                        offered: "close_valve_v2".into()
                    }
                );
                assert_eq!(
                    r.expect("the words are kept").response,
                    "close_valve_v2",
                    "a parser that discards an off-menu answer cannot be fixed by \
                     anyone but the person who wrote it"
                );
            }
            _ => unreachable!("expected a refusal"),
        }
    }

    /// **A decline is a refusal, not a transport error.** This is the distinction
    /// the old `Result<String, String>` could not make: there, a shrug and a dead
    /// network were the same `Err` and here they are not.
    #[test]
    fn a_model_that_declines_is_a_refusal_with_its_own_words() {
        let m = mapper();
        let out = m.map("do something impossible", &menu(), |_| {
            Ok(reply("I'm sorry, I cannot map that intent"))
        });
        match out {
            Mapped::Refused { why, reply, model } => {
                assert_eq!(
                    why,
                    Refusal::NotAnId {
                        text: "I'm sorry, I cannot map that intent".into()
                    },
                    "and it is NotAnId rather than OffMenu, because it named \
                     nothing that could be an action"
                );
                assert!(reply.is_some(), "the words are kept");
                assert_eq!(model, "test-model", "and the refusal is attributable");
            }
            _ => panic!("a decline mapped to an action: {:?}", out.action()),
        }
    }

    /// A dead network and a model declining are both refusals and they are *not
    /// the same refusal*, which is the thing worth asserting.
    #[test]
    fn a_transport_failure_is_distinguishable_from_a_decline() {
        let m = mapper();
        let down = m.map("q", &menu(), |_| Err("connection refused".into()));
        let decline = m.map("q", &menu(), |_| Ok(reply("no")));
        match (&down, &decline) {
            (
                Mapped::Refused {
                    why: Refusal::Transport { .. },
                    reply,
                    ..
                },
                Mapped::Refused { reply: other, .. },
            ) => {
                assert!(reply.is_none(), "no reply exists to quote");
                assert!(other.is_some(), "and the decline does have one");
            }
            other => panic!("expected two refusals of different kinds: {other:?}"),
        }
    }

    /// **The seam carries a request, so the knobs are finally expressible.**
    /// `SlmOptions` used to be a struct that could not be reached from a call
    /// site, and a `temperature` nobody can set is a constant with extra steps.
    #[test]
    fn the_request_carries_the_options_that_could_not_be_sent_before() {
        let mut m = mapper();
        m.options = SlmOptions {
            temperature: 0.2,
            num_predict: 128,
        };
        let r = m.request("shut it down", &menu());
        assert_eq!(r.options.temperature, 0.2);
        assert_eq!(r.options.num_predict, 128);
        assert_eq!(r.model, "test-model");
        assert!(!r.stream);
        assert!(r.prompt.contains("emergency_shutdown"));
    }

    /// The default asks for a deterministic answer, which is the right default
    /// for a mapper: the answer is a lookup, not prose.
    #[test]
    fn the_default_options_ask_for_determinism() {
        let r = mapper().request("q", &menu());
        assert_eq!(
            r.options.temperature, 0.0,
            "a lookup is not a creative task"
        );
    }

    /// Whitespace is trimmed before judging, because a trailing newline is a
    /// transport artefact and not a different action. Trimming *after* judging
    /// would make the menu check fail on a correct answer.
    #[test]
    fn surrounding_whitespace_is_not_a_different_answer() {
        let m = mapper();
        let out = m.map("q", &menu(), |_| Ok(reply("  emergency_shutdown\n")));
        assert_eq!(out.action(), Some("emergency_shutdown"));
    }

    /// The menu is the menu: a *prefix* of a declared action is not that action.
    /// A fuzzy match here would reintroduce exactly the misrouting the old
    /// `resolve_primitive` was fixed for.
    #[test]
    fn a_prefix_of_a_declared_action_is_not_it() {
        let m = mapper();
        for wrong in ["emergency", "read", "set", "EMERGENCY_SHUTDOWN"] {
            let out = m.map("q", &menu(), |_| Ok(reply(wrong)));
            assert_eq!(out.action(), None, "{wrong:?} is not on the menu");
        }
    }

    /// An empty reply is a refusal, not an empty action. `String::new()` as an
    /// action id would be the old defect wearing a different hat.
    #[test]
    fn an_empty_reply_is_a_refusal() {
        let m = mapper();
        let out = m.map("q", &menu(), |_| Ok(reply("   ")));
        assert!(out.is_refused());
        assert_eq!(out.action(), None);
    }

    /// The fetch is called exactly once, with the model's own name, so a
    /// multi-turn conversation is not happening by accident.
    #[test]
    fn the_transport_is_called_once_with_a_named_request() {
        use std::cell::RefCell;
        let calls = RefCell::new(Vec::new());
        let m = mapper();
        let out = m.map("q", &menu(), |r: SlmRequest| {
            calls.borrow_mut().push(r);
            Ok(reply("read_state"))
        });
        assert_eq!(out.action(), Some("read_state"));
        let calls = calls.into_inner();
        assert_eq!(calls.len(), 1, "one intent, one request");
        assert_eq!(calls[0].model, "test-model");
    }

    /// **And the thing that made this file suspect: it is now exercised.** The
    /// old module had zero callers anywhere in the tree, which is how
    /// `SemanticSLM::new` came to be the only way anyone learned it existed.
    #[test]
    fn the_mapper_is_constructible_and_usable_from_a_test() {
        let m = SemanticSLM::new("http://127.0.0.1:11434".into(), "llama3".into());
        assert_eq!(m.endpoint, "http://127.0.0.1:11434");
        let out = m.map("read the valve", &menu(), |_| Ok(reply("read_state")));
        assert_eq!(out.action(), Some("read_state"));
    }
}
