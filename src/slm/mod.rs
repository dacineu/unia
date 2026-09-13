use crate::orchestrator::{ActivationVector, BehavioralVector};

#[derive(Debug)]
pub struct SlmResponse {
    pub text: String,
    pub tokens_used: usize,
    pub reasoning_steps: usize,
}

pub struct MockSlm;

impl MockSlm {
    pub fn new() -> Self {
        Self
    }

    /// Executes a request using the provided Activation Vector.
    /// This simulates the behavioral effects of the different modes.
    pub fn execute(&self, user_input: &str, activation: &ActivationVector) -> Result<SlmResponse, Box<dyn std::error::Error>> {
        let (response_text, tokens, steps) = match activation.behavioral_mode {
            BehavioralVector::Quickest => {
                (format!("FAST_EXECUTION: [Direct answer to '{}']", user_input), 30, 1)
            }
            BehavioralVector::Smartest => {
                (format!("DEEP_REASONING: [Step 1: Analysis] -> [Step 2: Validation] -> [Step 3: Final answer to '{}']", user_input), 450, 3)
            }
            BehavioralVector::Direct => {
                (format!("RAW_EXECUTION: [Technical result for '{}']", user_input), 100, 1)
            }
        };

        // Budget Enforcement
        if tokens > activation.token_budget {
            return Err("TOKEN_BUDGET_EXCEEDED: The actuator's budget was too small for this execution.".into());
        }

        Ok(SlmResponse {
            text: response_text,
            tokens_used: tokens,
            reasoning_steps: steps,
        })
    }
}
