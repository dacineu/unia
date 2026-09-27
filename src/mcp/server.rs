//! MCP server exposing the `.ure` corpus to a coding agent.
//!
//! Purpose: let an agent satisfy a repeated task from a locally-stored pattern
//! instead of spending a provider request on it. Every tool reports whether a
//! hit actually avoided a model call, because a manifest with no runnable
//! payload does not, and an agent that assumes otherwise will trust a pattern
//! that cannot execute.

use super::store::{new_trace, Match, Stats, Store};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock};
use rmcp::{tool, tool_handler, tool_router, ErrorData as McpError, ServerHandler};
use schemars::JsonSchema;
use serde::Deserialize;
use std::sync::{Arc, RwLock};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchArgs {
    /// What you are about to do, in natural language, e.g. "add a rust
    /// dependency to Cargo.toml and update the lockfile".
    pub intent: String,
    /// Maximum patterns to return. Defaults to 5.
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RecordArgs {
    /// The intent as it was actually issued.
    pub intent: String,
    /// The pattern that served it, if any.
    pub resource_id: Option<String>,
    /// `hit` if a pattern served the call, `miss` if a provider was called.
    pub outcome: String,
    /// Input tokens spent. Use 0 on a hit.
    pub tokens_in: Option<u64>,
    /// Output tokens spent. Use 0 on a hit.
    pub tokens_out: Option<u64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetArgs {
    pub id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PromoteArgs {
    /// Capability name this pattern is the champion of, e.g. "rust.add-dependency".
    pub capability: String,
    /// Pattern id, as returned by `unia_search`.
    pub id: String,
}

fn text_result(payload: serde_json::Value) -> CallToolResult {
    let pretty = serde_json::to_string_pretty(&payload).unwrap_or_else(|_| payload.to_string());
    CallToolResult::success(vec![ContentBlock::text(pretty)])
}

fn render_matches(matches: &[Match]) -> serde_json::Value {
    let runnable = matches.iter().filter(|m| m.saves_tokens).count();
    serde_json::json!({
        "match_count": matches.len(),
        "runnable_count": runnable,
        "provider_call_avoided": runnable > 0,
        "note": if runnable == 0 {
            "No runnable payload. These patterns describe capability but cannot be \
             executed locally, so acting on one still costs a provider request."
        } else {
            "Runnable payloads present. Executing one of these costs no provider tokens."
        },
        "matches": matches,
    })
}

#[derive(Clone)]
pub struct UniaServer {
    store: Arc<RwLock<Store>>,
}

impl UniaServer {
    pub fn new(store: Store) -> Self {
        Self { store: Arc::new(RwLock::new(store)) }
    }

    fn read(&self) -> Result<std::sync::RwLockReadGuard<'_, Store>, McpError> {
        self.store.read().map_err(|_| McpError::internal_error("store lock poisoned", None))
    }

    fn write(&self) -> Result<std::sync::RwLockWriteGuard<'_, Store>, McpError> {
        self.store.write().map_err(|_| McpError::internal_error("store lock poisoned", None))
    }
}

#[tool_router]
impl UniaServer {
    /// Find a stored `.ure` pattern for an intent. Call this before doing work
    /// you have done before. A match with `saves_tokens: true` carries a runnable
    /// payload and can be executed locally with no provider request.
    #[tool(description = "Find a stored .ure pattern for an intent. Call before repeating a \
                          task; a match with saves_tokens=true can be executed locally without \
                          spending provider tokens.")]
    fn unia_search(&self, Parameters(args): Parameters<SearchArgs>) -> Result<CallToolResult, McpError> {
        if args.intent.trim().is_empty() {
            return Err(McpError::invalid_params("intent must not be empty", None));
        }
        let store = self.read()?;
        let limit = args.limit.unwrap_or(5).clamp(1, 50);
        Ok(text_result(render_matches(&store.search(&args.intent, limit))))
    }

    /// Record how a call was served. `hit` means a pattern avoided a provider
    /// request; `miss` means tokens were spent. This is the only record the
    //  project has of what was learned, and without it no pattern can be
    //  induced from real usage.
    #[tool(description = "Record whether a call was served by a stored pattern (hit) or by a \
                          provider (miss), with the tokens spent. Always call this after acting.")]
    fn unia_record(&self, Parameters(args): Parameters<RecordArgs>) -> Result<CallToolResult, McpError> {
        if !matches!(args.outcome.as_str(), "hit" | "miss") {
            return Err(McpError::invalid_params(
                "outcome must be either \"hit\" or \"miss\"",
                None,
            ));
        }
        let trace = new_trace(
            args.intent,
            args.resource_id,
            &args.outcome,
            args.tokens_in.unwrap_or(0),
            args.tokens_out.unwrap_or(0),
        );
        let mut store = self.write()?;
        store
            .record(trace)
            .map_err(|e| McpError::internal_error(format!("could not append trace: {e}"), None))?;
        Ok(text_result(serde_json::json!({ "recorded": true })))
    }

    /// Fetch one manifest by id, including its actions and payload.
    #[tool(description = "Fetch a single .ure manifest by resource id, with its actions, \
                          aliases, constraints and payload.")]
    fn unia_get(&self, Parameters(args): Parameters<GetArgs>) -> Result<CallToolResult, McpError> {
        let store = self.read()?;
        match store.get(&args.id) {
            Some(p) => Ok(text_result(serde_json::json!({
                "id": p.id,
                "category": p.category,
                "guidance": p.guidance,
                "actions": p.actions,
                "payload": p.payload,
                "saves_tokens": p.payload.as_ref().is_some_and(|pl| pl.is_runnable()),
                "source": p.source.display().to_string(),
            }))),
            None => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "no pattern with id {} in the corpus",
                args.id
            ))])),
        }
    }

    /// Mark a pattern the champion for a capability, so it wins future ties.
    /// Only promote a pattern you have verified works.
    #[tool(description = "Mark a pattern the champion for a capability so it wins ranking ties. \
                          Only promote patterns you have verified.")]
    fn unia_promote(&self, Parameters(args): Parameters<PromoteArgs>) -> Result<CallToolResult, McpError> {
        let mut store = self.write()?;
        match store.promote(&args.capability, &args.id) {
            Ok(()) => Ok(text_result(serde_json::json!({
                "promoted": args.id,
                "capability": args.capability,
            }))),
            Err(e) => Ok(CallToolResult::error(vec![ContentBlock::text(e.to_string())])),
        }
    }

    /// Corpus size, coverage, and the tokens saved so far.
    #[tool(description = "Corpus size, runnable count, champion count, and tokens saved by \
                          patterns serving calls instead of a provider.")]
    fn unia_stats(&self) -> Result<CallToolResult, McpError> {
        let store = self.read()?;
        let s: Stats = store.stats();
        Ok(text_result(serde_json::json!(s)))
    }

    /// Re-read the corpus from disk, picking up manifests written since startup.
    #[tool(description = "Reload the .ure corpus from disk to pick up newly written manifests.")]
    fn unia_reload(&self) -> Result<CallToolResult, McpError> {
        let mut store = self.write()?;
        store.reload();
        let s = store.stats();
        Ok(text_result(serde_json::json!(s)))
    }
}

// `version` must be a literal: the macro rejects an `env!` expansion here, so
// this is kept in step with the crate version by hand.
#[tool_handler(name = "unia", version = "0.1.0", instructions = "Stores .ure \
    patterns so a repeated task can be served locally instead of costing a provider request. \
    Intended loop: unia_search before repeating work, unia_record after acting, unia_promote \
    only for patterns verified to work. A match with saves_tokens=false still costs tokens to \
    act on.")]
impl ServerHandler for UniaServer {
    fn get_info(&self) -> rmcp::model::ServerConfig {
        // Name and version come from the #[tool_handler] attribute above.
        rmcp::model::ServerConfig::new(
            rmcp::model::ServerCapabilities::builder().enable_tools().build(),
        )
    }
}
