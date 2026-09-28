//! Evaluation of a manifest's declared `constraints`.
//!
//! This module is the **verifier**. Until it existed, `constraints` was an array
//! of free strings that the bridge printed for operator visibility and never
//! checked, which meant a manifest could declare a precondition the system
//! silently ignored. That is divergence D5, and it was recorded for most of the
//! project's life as a safety gap rather than as the defect it actually is: with
//! no evaluated precondition, a learning loop has no reward signal, and a
//! verifier that does not check anything is close to a random verifier.
//!
//! The formalism is already normative and the code was behind it. From
//! `docs/ure-specification-formal.md` §2.4: "Constraints are predicates that must
//! return `True` for an action to be dispatched to the Nucleus", with
//! $\mathcal{C} = \{p_1 \dots p_k\}$ where each $p: \mathcal{S} \to \{0,1\}$.
//! This module is $p$. It is not a general expression evaluator, and it is
//! deliberately small: a constraint is a comparison against declared state, and
//! anything a manifest author writes that is not a comparison is rejected rather
//! than interpreted.
//!
//! # What is not implemented
//!
//! The formal specification's own example is prose — `SetBrightness(v) requires
//! power == True` — while the corpus uses `status != 'fault'`. Only the corpus
//! form parses. The `requires` form is recorded as divergence D12 rather than
//! guessed at, because a constraint evaluator that silently accepts a syntax it
//! does not understand is a verifier that passes things it did not check.

use std::collections::BTreeMap;
use std::fmt;

/// Current values of a resource's declared state, as strings.
///
/// The nucleus stores runtime state this way already, so this is the shape the
/// verifier consumes rather than a new one. A field absent from the map is
/// unknown, which is **not** the same as false: see [`Verdict::UnknownField`].
pub type State = BTreeMap<String, String>;

/// The comparison a constraint makes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Comparison {
    Equal,
    NotEqual,
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
}

impl Comparison {
    fn parse(sym: &str) -> Option<Self> {
        Some(match sym {
            "==" => Comparison::Equal,
            "!=" => Comparison::NotEqual,
            "<" => Comparison::Less,
            "<=" => Comparison::LessOrEqual,
            ">" => Comparison::Greater,
            ">=" => Comparison::GreaterOrEqual,
            _ => return None,
        })
    }
}

/// A right-hand side literal.
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Text(String),
    Number(f64),
    Bool(bool),
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Text(s) => write!(f, "'{s}'"),
            Literal::Number(n) => write!(f, "{n}"),
            Literal::Bool(b) => write!(f, "{b}"),
        }
    }
}

/// A parsed constraint: `<field> <comparison> <literal>`.
#[derive(Debug, Clone, PartialEq)]
pub struct Constraint {
    pub field: String,
    pub comparison: Comparison,
    pub literal: Literal,
    /// The source text, kept so a failure can quote what the author wrote.
    pub source: String,
}

impl Constraint {
    /// Parses one constraint, or explains why it could not be parsed.
    ///
    /// Unparseable is an error rather than a pass, because a verifier that
    /// cannot read a constraint and a verifier that read it and found it true
    /// must not be indistinguishable to the caller.
    pub fn parse(text: &str) -> Result<Self, String> {
        let source = text.trim();
        if source.is_empty() {
            return Err("constraint is empty".to_string());
        }

        // Longest operator first: `<=` must not be read as `<` then `=`.
        let (index, length) = ["==", "!=", "<=", ">=", "<", ">"]
            .iter()
            .find_map(|op| source.find(op).map(|i| (i, op.len())))
            .ok_or_else(|| {
                format!(
                    "no comparison in constraint {source:?}. Expected \
                     `field <op> value` with op one of ==, !=, <, <=, >, >=."
                )
            })?;

        let field = source[..index].trim().to_string();
        let symbol = &source[index..index + length];
        let rest = source[index + length..].trim();

        if field.is_empty() {
            return Err(format!("constraint {source:?} has no field on its left"));
        }
        if !is_identifier(&field) {
            return Err(format!(
                "constraint {source:?} has {field:?} as its field, which is not a \
                 declared state key. A constraint may only test a name that \
                 appears in the resource's state_space."
            ));
        }
        if rest.is_empty() {
            return Err(format!("constraint {source:?} has no value on its right"));
        }

        let comparison = Comparison::parse(symbol)
            .ok_or_else(|| format!("unknown comparison {symbol:?} in {source:?}"))?;
        let literal = parse_literal(rest)?;

        Ok(Constraint {
            field,
            comparison,
            literal,
            source: source.to_string(),
        })
    }

    /// Evaluates against a state, yielding a verdict rather than a bool.
    ///
    /// Three-valued on purpose. A field the state does not carry cannot be
    /// compared, and collapsing that into `false` would refuse legitimate
    /// actions; collapsing it into `true` would dispatch them unchecked. Only
    /// [`Verdict::Holds`] permits dispatch.
    pub fn evaluate(&self, state: &State) -> Verdict {
        let Some(actual) = state.get(&self.field) else {
            return Verdict::UnknownField(self.field.clone());
        };

        // Both sides are read as the same kind of thing where possible, so that
        // `status != 'fault'` compares two strings and `flow_rate < 1.0` compares
        // two numbers, rather than string-comparing everything.
        //
        // The two sides use *different* literal parsers, on purpose. A
        // constraint is source text, where a bare word is an authoring mistake
        // and is refused. A state value is a runtime value, where a bare word
        // is simply a string: the nucleus stores `status` as `open` with no
        // quotes, and reading that as a parse error would refuse every
        // correctly-declared enum precondition, which is the opposite of a
        // working verifier.
        match (&self.literal, parse_state_value(actual)) {
            (Literal::Text(want), Ok(Literal::Text(got))) => {
                self.verdict(actual, &compare(&got, want))
            }
            (Literal::Number(want), Ok(Literal::Number(got))) => {
                self.verdict(actual, &compare(&got, want))
            }
            (Literal::Bool(want), Ok(Literal::Bool(got))) => self.verdict(actual, &compare(&got, want)),
            // A type mismatch is a manifest bug and is reported as one rather
            // than being coerced into a comparison that would silently pass.
            (_, Ok(other)) => Verdict::TypeMismatch {
                field: self.field.clone(),
                held: actual.clone(),
                required: self.literal.to_string(),
                found: other.to_string(),
            },
            (_, Err(e)) => Verdict::UnreadableValue {
                field: self.field.clone(),
                value: actual.clone(),
                reason: e,
            },
        }
    }

    fn verdict(&self, observed: &str, ordering: &std::cmp::Ordering) -> Verdict {
        use std::cmp::Ordering::*;
        let holds = match (self.comparison.clone(), ordering) {
            (Comparison::Equal, Equal) => true,
            (Comparison::NotEqual, Equal) => false,
            (Comparison::NotEqual, _) => true,
            (Comparison::Less, Less) => true,
            (Comparison::LessOrEqual, Less | Equal) => true,
            (Comparison::Greater, Greater) => true,
            (Comparison::GreaterOrEqual, Greater | Equal) => true,
            // The only remaining cases are a mismatch between an ordering
            // operator and an equality result, which is false by definition.
            _ => false,
        };
        if holds {
            Verdict::Holds
        } else {
            Verdict::Fails {
                field: self.field.clone(),
                required: self.literal.to_string(),
                observed: observed.to_string(),
            }
        }
    }
}

fn compare<T: PartialOrd + PartialEq>(left: &T, right: &T) -> std::cmp::Ordering {
    if left == right {
        std::cmp::Ordering::Equal
    } else if left < right {
        std::cmp::Ordering::Less
    } else {
        std::cmp::Ordering::Greater
    }
}

fn is_identifier(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '.')
        && !s.starts_with(|c: char| c.is_ascii_digit())
}

/// Reads a runtime state value.
///
/// The inverse of [`parse_literal`] in the one respect that matters: a value that
/// is neither a number nor a boolean is text, because that is what an enum field
/// holds. The empty string is refused, because a field reported as carrying no
/// value has told us nothing and must not read as the empty string.
fn parse_state_value(raw: &str) -> Result<Literal, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("no value has been reported for this field".to_string());
    }
    if let Ok(n) = trimmed.parse::<f64>() {
        return Ok(Literal::Number(n));
    }
    match trimmed.to_ascii_lowercase().as_str() {
        "true" => Ok(Literal::Bool(true)),
        "false" => Ok(Literal::Bool(false)),
        _ => Ok(Literal::Text(trimmed.to_string())),
    }
}

fn parse_literal(raw: &str) -> Result<Literal, String> {
    let trimmed = raw.trim();
    if trimmed.len() >= 2 && trimmed.starts_with('\'') && trimmed.ends_with('\'') {
        return Ok(Literal::Text(trimmed[1..trimmed.len() - 1].to_string()));
    }
    if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
        return Ok(Literal::Text(trimmed[1..trimmed.len() - 1].to_string()));
    }
    if let Ok(n) = trimmed.parse::<f64>() {
        return Ok(Literal::Number(n));
    }
    match trimmed.to_ascii_lowercase().as_str() {
        "true" => Ok(Literal::Bool(true)),
        "false" => Ok(Literal::Bool(false)),
        _ => Err(format!("{trimmed:?} is not a quoted string, a number, or a boolean")),
    }
}

/// The outcome of evaluating one constraint.
#[derive(Debug, Clone, PartialEq)]
pub enum Verdict {
    /// The precondition is satisfied. Dispatch is permitted.
    Holds,
    /// The precondition is violated. Dispatch is refused.
    ///
    /// Carries the observed value as well as the required one, because this is
    /// the phrase the creature turns into its question and a player cannot act on
    /// "status must not be fault" without being told what it currently is.
    Fails {
        field: String,
        required: String,
        observed: String,
    },
    /// The state carries no value for this field, so it cannot be judged.
    UnknownField(String),
    /// The state carries a value that is not the kind the constraint compares.
    TypeMismatch {
        field: String,
        held: String,
        required: String,
        found: String,
    },
    /// The state's value for this field is not readable as a literal.
    UnreadableValue {
        field: String,
        value: String,
        reason: String,
    },
}

impl Verdict {
    /// Whether dispatch may proceed. Only a satisfied predicate does.
    pub fn permits_dispatch(&self) -> bool {
        matches!(self, Verdict::Holds)
    }

    /// A phrase naming what is wrong, in the manifest author's terms.
    ///
    /// This is what the creature turns into its question, so it must state the
    /// contradiction as a fact the player can check rather than as a score.
    pub fn reason(&self) -> Option<String> {
        match self {
            Verdict::Holds => None,
            Verdict::Fails {
                field,
                required,
                observed,
            } => Some(format!("{field} is {observed}, and this needs it to be {required}")),
            Verdict::UnknownField(field) => Some(format!(
                "{field} is a declared field, but nothing has told me its value yet"
            )),
            Verdict::TypeMismatch {
                field,
                held,
                required,
                found,
            } => Some(format!(
                "{field} is {held}, which cannot be compared with {required} \
                 because it is {found}"
            )),
            Verdict::UnreadableValue {
                field,
                value,
                reason,
            } => Some(format!("{field} is {value:?}, which is not readable: {reason}")),
        }
    }
}

/// The combined verdict over every constraint an action declares.
#[derive(Debug, Clone, PartialEq)]
pub struct Report {
    pub checked: usize,
    pub first_refusal: Option<String>,
}

impl Report {
    pub fn permits_dispatch(&self) -> bool {
        self.first_refusal.is_none()
    }
}

/// Parses and evaluates every constraint an action declares, in order.
///
/// Stops at the first refusal, because a precondition that already blocks
/// dispatch makes the ones after it irrelevant, and reporting all of them would
/// bury the one the player has to act on.
pub fn check(constraints: &[String], state: &State) -> Report {
    let mut first_refusal = None;
    for raw in constraints {
        match Constraint::parse(raw) {
            Err(reason) => {
                first_refusal.get_or_insert_with(|| {
                    format!("I cannot read the precondition {raw:?}: {reason}")
                });
            }
            Ok(constraint) => {
                if let Some(reason) = constraint.evaluate(state).reason() {
                    first_refusal.get_or_insert(reason);
                }
            }
        }
    }
    Report {
        checked: constraints.len(),
        first_refusal,
    }
}

#[cfg(test)]
mod constraint_tests {
    use super::*;

    fn state(pairs: &[(&str, &str)]) -> State {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    mod parse_works {
        use super::*;

        #[test]
        fn reads_a_string_inequality_as_written_in_the_corpus() {
            let c = Constraint::parse("status != 'fault'").unwrap();
            assert_eq!(c.field, "status");
            assert_eq!(c.comparison, Comparison::NotEqual);
            assert_eq!(c.literal, Literal::Text("fault".to_string()));
        }

        #[test]
        fn reads_a_bounded_number_comparison() {
            let c = Constraint::parse("flow_rate <= 1.0").unwrap();
            assert_eq!(c.comparison, Comparison::LessOrEqual);
            assert_eq!(c.literal, Literal::Number(1.0));
        }

        #[test]
        fn reads_a_boolean_equality() {
            let c = Constraint::parse("power == true").unwrap();
            assert_eq!(c.literal, Literal::Bool(true));
        }

        #[test]
        fn reads_a_dotted_field_path() {
            let c = Constraint::parse("sensor.pressure < 0.5").unwrap();
            assert_eq!(c.field, "sensor.pressure");
        }
    }

    mod parse_refuses {
        use super::*;

        #[test]
        fn an_empty_constraint_is_refused_rather_than_passing() {
            // The failure this guards: a verifier that cannot read a constraint
            // and one that read it and found it true must not look alike.
            assert!(Constraint::parse("   ").is_err());
        }

        #[test]
        fn a_constraint_with_no_comparison_is_refused() {
            let err = Constraint::parse("status is fault").unwrap_err();
            assert!(err.contains("no comparison"), "{err}");
        }

        #[test]
        fn a_constraint_with_no_value_is_refused() {
            assert!(Constraint::parse("status ==").is_err());
        }

        #[test]
        fn a_field_that_is_not_an_identifier_is_refused() {
            // A constraint may only test a declared state key, so prose on the
            // left is a manifest bug and must not be interpreted.
            let err = Constraint::parse("the valve status != 'fault'").unwrap_err();
            assert!(err.contains("not a\ndeclared state key") || err.contains("not a declared state key"), "{err}");
        }

        #[test]
        fn an_unreadable_value_is_refused() {
            assert!(Constraint::parse("status != maybe").is_err());
        }

        #[test]
        fn the_specifications_own_requires_syntax_is_not_silently_accepted() {
            // `SetBrightness(v) requires power == True` is the example in
            // ure-specification-formal.md §2.4. Parsing its comparison and
            // ignoring its subject would pass a precondition nobody checked.
            let err = Constraint::parse("SetBrightness(v) requires power == True").unwrap_err();
            assert!(err.contains("declared state key"), "{err}");
        }
    }

    mod evaluates {
        use super::*;

        #[test]
        fn a_satisfied_precondition_permits_dispatch() {
            let c = Constraint::parse("status != 'fault'").unwrap();
            let v = c.evaluate(&state(&[("status", "open")]));
            assert!(v.permits_dispatch());
            assert_eq!(v.reason(), None);
        }

        #[test]
        fn a_violated_precondition_refuses_dispatch() {
            let c = Constraint::parse("status != 'fault'").unwrap();
            let v = c.evaluate(&state(&[("status", "fault")]));
            assert!(!v.permits_dispatch());
            let reason = v.reason().unwrap();
            assert!(reason.contains("status is fault"), "{reason}");
            assert!(reason.contains("needs it to be 'fault'"), "{reason}");
        }

        #[test]
        fn a_numeric_boundary_is_inclusive_where_the_operator_says_so() {
            let at = Constraint::parse("flow_rate <= 1.0").unwrap();
            assert!(at.evaluate(&state(&[("flow_rate", "1.0")])).permits_dispatch());

            let over = Constraint::parse("flow_rate < 1.0").unwrap();
            assert!(!over
                .evaluate(&state(&[("flow_rate", "1.0")]))
                .permits_dispatch());
        }

        #[test]
        fn an_unknown_field_blocks_dispatch_rather_than_passing() {
            // A precondition on a field nobody has reported a value for has not
            // been shown to hold. Passing it would make an unreadable state look
            // like a satisfied one.
            let c = Constraint::parse("status != 'fault'").unwrap();
            let v = c.evaluate(&State::new());
            assert!(!v.permits_dispatch());
            assert!(matches!(v, Verdict::UnknownField(_)), "{v:?}");
        }

        #[test]
        fn a_type_mismatch_is_reported_rather_than_coerced() {
            // A text value against a numeric constraint is the mismatch that
            // matters: coercing it would compare "wide open" as a number and
            // pass or fail for a reason the manifest never wrote.
            let c = Constraint::parse("flow_rate <= 1.0").unwrap();
            let v = c.evaluate(&state(&[("flow_rate", "wide open")]));
            assert!(!v.permits_dispatch());
            assert!(matches!(v, Verdict::TypeMismatch { .. }), "{v:?}");
        }

        #[test]
        fn a_bare_enum_value_in_state_is_read_as_a_string() {
            // The nucleus stores `status` as `open` with no quotes. Reading that
            // as a parse error would refuse every correctly-declared enum
            // precondition, which is the opposite of a working verifier.
            let c = Constraint::parse("status != 'fault'").unwrap();
            assert!(c.evaluate(&state(&[("status", "open")])).permits_dispatch());
            assert!(!c.evaluate(&state(&[("status", "fault")])).permits_dispatch());
        }

        #[test]
        fn a_state_value_of_the_wrong_shape_is_reported() {
            let c = Constraint::parse("status != 'fault'").unwrap();
            let v = c.evaluate(&state(&[("status", "")]));
            assert!(!v.permits_dispatch());
            assert!(matches!(v, Verdict::UnreadableValue { .. }), "{v:?}");
        }
    }

    mod over_many {
        use super::*;

        #[test]
        fn no_constraints_permits_dispatch() {
            let report = check(&[], &State::new());
            assert_eq!(report.checked, 0);
            assert!(report.permits_dispatch());
        }

        #[test]
        fn every_satisfied_constraint_permits_dispatch() {
            let cs = vec!["status != 'fault'".to_string(), "power == true".to_string()];
            let report = check(&cs, &state(&[("status", "open"), ("power", "true")]));
            assert_eq!(report.checked, 2);
            assert!(report.permits_dispatch());
        }

        #[test]
        fn one_violation_among_satisfied_ones_refuses() {
            let cs = vec!["status != 'fault'".to_string(), "power == true".to_string()];
            let report = check(&cs, &state(&[("status", "open"), ("power", "false")]));
            assert!(!report.permits_dispatch());
            assert!(
                report.first_refusal.as_deref().unwrap_or_default().contains("power"),
                "{report:?}"
            );
        }

        #[test]
        fn an_unreadable_constraint_refuses_rather_than_being_skipped() {
            let cs = vec!["status is fine".to_string()];
            let report = check(&cs, &State::new());
            assert!(!report.permits_dispatch());
            assert!(
                report
                    .first_refusal
                    .as_deref()
                    .unwrap_or_default()
                    .contains("cannot read"),
                "{report:?}"
            );
        }

        #[test]
        fn the_first_refusal_is_the_one_reported() {
            // A precondition that already blocks dispatch makes the rest
            // irrelevant, and reporting all of them buries what to act on.
            let cs = vec![
                "status != 'fault'".to_string(),
                "power == true".to_string(),
            ];
            let report = check(&cs, &state(&[("status", "fault"), ("power", "false")]));
            assert!(
                report.first_refusal.as_deref().unwrap_or_default().contains("status"),
                "{report:?}"
            );
        }
    }
}
