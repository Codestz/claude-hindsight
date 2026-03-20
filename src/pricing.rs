//! Model-aware pricing for Claude API usage.
//!
//! Calculates costs from token counts using per-model pricing tables.
//! Supports all current Claude models with automatic fallback to Sonnet rates.

use crate::parser::models::TokenUsage;

/// Pricing rates per million tokens for a Claude model.
#[derive(Debug, Clone, Copy)]
pub struct ModelPricing {
    pub input: f64,
    pub output: f64,
    pub cache_creation: f64,
    pub cache_read: f64,
}

/// All known Claude model pricing (per million tokens).
/// Source: https://docs.anthropic.com/en/docs/about-claude/models
///
/// Rates are updated manually — Claude pricing changes infrequently.
const PRICING: &[(&str, ModelPricing)] = &[
    // Opus 4 / 4.5 / 4.6
    (
        "claude-opus-4",
        ModelPricing {
            input: 15.0,
            output: 75.0,
            cache_creation: 18.75,
            cache_read: 1.50,
        },
    ),
    // Sonnet 4 / 4.5 / 4.6
    (
        "claude-sonnet-4",
        ModelPricing {
            input: 3.0,
            output: 15.0,
            cache_creation: 3.75,
            cache_read: 0.30,
        },
    ),
    // Haiku 3.5 / 4.5
    (
        "claude-haiku-4",
        ModelPricing {
            input: 0.80,
            output: 4.0,
            cache_creation: 1.0,
            cache_read: 0.08,
        },
    ),
    (
        "claude-haiku-3-5",
        ModelPricing {
            input: 0.80,
            output: 4.0,
            cache_creation: 1.0,
            cache_read: 0.08,
        },
    ),
    // Sonnet 3.5 (legacy)
    (
        "claude-sonnet-3-5",
        ModelPricing {
            input: 3.0,
            output: 15.0,
            cache_creation: 3.75,
            cache_read: 0.30,
        },
    ),
    // Sonnet 3 (legacy)
    (
        "claude-sonnet-3",
        ModelPricing {
            input: 3.0,
            output: 15.0,
            cache_creation: 3.75,
            cache_read: 0.30,
        },
    ),
    // Opus 3 (legacy)
    (
        "claude-opus-3",
        ModelPricing {
            input: 15.0,
            output: 75.0,
            cache_creation: 18.75,
            cache_read: 1.50,
        },
    ),
    // Haiku 3 (legacy)
    (
        "claude-haiku-3",
        ModelPricing {
            input: 0.25,
            output: 1.25,
            cache_creation: 0.30,
            cache_read: 0.03,
        },
    ),
];

/// Default pricing when model is unknown (Sonnet rates — most common in Claude Code).
const DEFAULT_PRICING: ModelPricing = ModelPricing {
    input: 3.0,
    output: 15.0,
    cache_creation: 3.75,
    cache_read: 0.30,
};

/// Look up pricing for a model name (date-suffix already stripped).
///
/// Uses prefix matching: "claude-sonnet-4-5" matches "claude-sonnet-4".
/// Falls back to Sonnet rates if no match found.
pub fn lookup_pricing(model: Option<&str>) -> &'static ModelPricing {
    let model = match model {
        Some(m) => m,
        None => return &DEFAULT_PRICING,
    };

    // Try exact match first, then progressively shorter prefixes.
    // "claude-sonnet-4-5" → "claude-sonnet-4" → match
    for &(prefix, ref pricing) in PRICING {
        if model.starts_with(prefix) {
            return pricing;
        }
    }

    &DEFAULT_PRICING
}

/// Calculate cost in USD from token usage and model.
///
/// `model` should be the date-stripped model name (e.g. "claude-sonnet-4-5").
pub fn calculate_cost(usage: &TokenUsage, model: Option<&str>) -> f64 {
    let pricing = lookup_pricing(model);

    let input = usage.input_tokens.unwrap_or(0) as f64;
    let output = usage.output_tokens.unwrap_or(0) as f64;
    let cache_creation = usage.cache_creation_input_tokens.unwrap_or(0) as f64;
    let cache_read = usage.cache_read_input_tokens.unwrap_or(0) as f64;

    (input * pricing.input
        + output * pricing.output
        + cache_creation * pricing.cache_creation
        + cache_read * pricing.cache_read)
        / 1_000_000.0
}

/// Calculate cost for a session by summing per-node costs with model awareness.
///
/// Each node's model is checked individually — handles mixed-model sessions
/// (e.g., main agent on Sonnet, subagents on Haiku).
/// Falls back to `session_model` when a node has no model.
pub fn calculate_session_cost(
    nodes: &[crate::parser::ExecutionNode],
    session_model: Option<&str>,
) -> f64 {
    let mut total = 0.0;
    for node in nodes {
        if let Some(tu) = node.effective_token_usage() {
            let node_model = node
                .message
                .as_ref()
                .and_then(|m| m.model_short())
                .or(session_model);
            total += calculate_cost(tu, node_model);
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_known_models() {
        let sonnet = lookup_pricing(Some("claude-sonnet-4-5"));
        assert!((sonnet.input - 3.0).abs() < f64::EPSILON);

        let opus = lookup_pricing(Some("claude-opus-4-5"));
        assert!((opus.input - 15.0).abs() < f64::EPSILON);

        let haiku = lookup_pricing(Some("claude-haiku-4-5"));
        assert!((haiku.input - 0.80).abs() < f64::EPSILON);
    }

    #[test]
    fn test_lookup_fallback() {
        let unknown = lookup_pricing(Some("gpt-4o"));
        assert!((unknown.input - 3.0).abs() < f64::EPSILON);

        let none = lookup_pricing(None);
        assert!((none.input - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_cost() {
        let usage = TokenUsage {
            input_tokens: Some(1_000_000),
            output_tokens: Some(100_000),
            cache_creation_input_tokens: Some(0),
            cache_read_input_tokens: Some(0),
        };

        // Sonnet: $3 input + $1.5 output = $4.50
        let cost = calculate_cost(&usage, Some("claude-sonnet-4-5"));
        assert!((cost - 4.5).abs() < 0.001);

        // Opus: $15 input + $7.5 output = $22.50
        let cost = calculate_cost(&usage, Some("claude-opus-4-5"));
        assert!((cost - 22.5).abs() < 0.001);
    }

    #[test]
    fn test_cache_tokens_pricing() {
        let usage = TokenUsage {
            input_tokens: Some(0),
            output_tokens: Some(0),
            cache_creation_input_tokens: Some(1_000_000),
            cache_read_input_tokens: Some(1_000_000),
        };

        // Sonnet: $3.75 creation + $0.30 read = $4.05
        let cost = calculate_cost(&usage, Some("claude-sonnet-4-5"));
        assert!((cost - 4.05).abs() < 0.001);
    }
}
