use crate::config::Config;
use anyhow::Context;

#[derive(Debug, Clone)]
pub struct CliToolPayload {
    pub content: String,
}

/// Build a simple JSON payload for conv_* CLI tools.
/// This keeps the CLI behavior stable by echoing back the requested tool name
/// and parsed args as a compact JSON object. The CLI prints `payload.content`.
pub async fn invoke_conv_tool_for_cli(
    _cfg: Config,
    name: String,
    args: String,
) -> anyhow::Result<CliToolPayload> {
    // Validate that args is valid JSON (object or value). If not, fail early.
    let val: serde_json::Value =
        serde_json::from_str(&args).with_context(|| format!("invalid JSON for --args: {args}"))?;

    // Compose a compact JSON with the tool name and args for consumption by wrappers.
    let payload = serde_json::json!({
        "tool": name,
        "args": val,
    });
    let content = serde_json::to_string(&payload)?;
    Ok(CliToolPayload { content })
}
