use crate::models::ResponseItem;

const DEFAULT_MAX_ITEMS: usize = 1024;
const DEFAULT_MAX_TOKENS: usize = 128_000; // rough token budget

/// Transcript of conversation history with simple size guards
#[derive(Debug, Clone)]
pub(crate) struct ConversationHistory {
    /// The oldest items are at the beginning of the vector.
    items: Vec<ResponseItem>,
    approx_tokens: usize,
    max_tokens: usize,
    max_items: usize,
}

impl Default for ConversationHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl ConversationHistory {
    pub(crate) fn new() -> Self {
        Self {
            items: Vec::new(),
            approx_tokens: 0,
            max_tokens: DEFAULT_MAX_TOKENS,
            max_items: DEFAULT_MAX_ITEMS,
        }
    }

    /// Returns a clone of the contents in the transcript.
    pub(crate) fn contents(&self) -> Vec<ResponseItem> {
        self.items.clone()
    }

    /// `items` is ordered from oldest to newest.
    pub(crate) fn record_items<I>(&mut self, items: I)
    where
        I: IntoIterator,
        I::Item: std::ops::Deref<Target = ResponseItem>,
    {
        for item in items {
            if !is_api_message(&item) {
                continue;
            }

            // Merge adjacent assistant messages into a single history entry.
            match (&*item, self.items.last_mut()) {
                (
                    ResponseItem::Message {
                        role: new_role,
                        content: new_content,
                        ..
                    },
                    Some(ResponseItem::Message {
                        role: last_role,
                        content: last_content,
                        ..
                    }),
                ) if new_role == "assistant" && last_role == "assistant" => {
                    let delta_tokens = estimate_tokens_in_content(new_content);
                    append_text_content(last_content, new_content);
                    self.approx_tokens = self.approx_tokens.saturating_add(delta_tokens);
                }
                _ => {
                    self.approx_tokens = self.approx_tokens.saturating_add(estimate_tokens(&item));
                    self.items.push(item.clone());
                }
            }
            self.enforce_limits();
        }
    }

    /// Append a text `delta` to the latest assistant message, creating a new
    /// assistant entry if none exists yet (e.g. first delta for this turn).
    pub(crate) fn append_assistant_text(&mut self, delta: &str) {
        let delta_tokens = estimate_text_tokens(delta);
        match self.items.last_mut() {
            Some(ResponseItem::Message { role, content, .. }) if role == "assistant" => {
                append_text_delta(content, delta);
                self.approx_tokens = self.approx_tokens.saturating_add(delta_tokens);
            }
            _ => {
                // Start a new assistant message with the delta.
                self.items.push(ResponseItem::Message {
                    id: None,
                    role: "assistant".to_string(),
                    content: vec![crate::models::ContentItem::OutputText {
                        text: delta.to_string(),
                    }],
                });
                self.approx_tokens = self.approx_tokens.saturating_add(delta_tokens);
                self.enforce_limits();
            }
        }
    }

    pub(crate) fn keep_last_messages(&mut self, n: usize) {
        if n == 0 {
            self.items.clear();
            self.approx_tokens = 0;
            return;
        }

        // Collect the last N message items (assistant/user), newest to oldest.
        let mut kept: Vec<ResponseItem> = Vec::with_capacity(n);
        for item in self.items.iter().rev() {
            if let ResponseItem::Message { role, content, .. } = item {
                kept.push(ResponseItem::Message {
                    id: None,
                    role: role.clone(),
                    content: content.clone(),
                });
                if kept.len() == n {
                    break;
                }
            }
        }

        kept.reverse();
        self.items = kept;
        self.recompute_tokens();
    }

    fn enforce_limits(&mut self) {
        while self.items.len() > self.max_items || self.approx_tokens > self.max_tokens {
            if let Some(removed) = self.items.first().cloned() {
                self.items.remove(0);
                let t = estimate_tokens(&removed);
                self.approx_tokens = self.approx_tokens.saturating_sub(t);
            } else {
                self.approx_tokens = 0;
                break;
            }
        }
    }

    fn recompute_tokens(&mut self) {
        self.approx_tokens = self.items.iter().map(estimate_tokens).sum();
    }
}

/// Anything that is not a system message or "reasoning" message is considered
/// an API message.
fn is_api_message(message: &ResponseItem) -> bool {
    match message {
        ResponseItem::Message { role, .. } => role.as_str() != "system",
        ResponseItem::FunctionCallOutput { .. }
        | ResponseItem::FunctionCall { .. }
        | ResponseItem::LocalShellCall { .. }
        | ResponseItem::Reasoning { .. } => true,
        ResponseItem::Other => false,
    }
}

fn estimate_tokens(item: &ResponseItem) -> usize {
    match item {
        ResponseItem::Message { content, .. } => estimate_tokens_in_content(content),
        ResponseItem::FunctionCall {
            name, arguments, ..
        } => estimate_text_tokens(name) + estimate_text_tokens(arguments),
        ResponseItem::FunctionCallOutput { output, .. } => estimate_text_tokens(&output.content),
        ResponseItem::LocalShellCall { .. } => 8,
        ResponseItem::Reasoning { .. } => 0,
        ResponseItem::Other => 0,
    }
}

fn estimate_tokens_in_content(content: &Vec<crate::models::ContentItem>) -> usize {
    content
        .iter()
        .map(|c| match c {
            crate::models::ContentItem::InputText { text }
            | crate::models::ContentItem::OutputText { text } => estimate_text_tokens(text),
            _ => 8,
        })
        .sum()
}

fn estimate_text_tokens(s: &str) -> usize {
    // very rough: ~4 chars per token
    s.len().div_ceil(4)
}

/// Helper to append the textual content from `src` into `dst` in place.
fn append_text_content(
    dst: &mut Vec<crate::models::ContentItem>,
    src: &Vec<crate::models::ContentItem>,
) {
    for c in src {
        if let crate::models::ContentItem::OutputText { text } = c {
            append_text_delta(dst, text);
        }
    }
}

/// Append a single text delta to the last OutputText item in `content`, or
/// push a new OutputText item if none exists.
fn append_text_delta(content: &mut Vec<crate::models::ContentItem>, delta: &str) {
    if let Some(crate::models::ContentItem::OutputText { text }) = content
        .iter_mut()
        .rev()
        .find(|c| matches!(c, crate::models::ContentItem::OutputText { .. }))
    {
        text.push_str(delta);
    } else {
        content.push(crate::models::ContentItem::OutputText {
            text: delta.to_string(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ContentItem;

    fn assistant_msg(text: &str) -> ResponseItem {
        ResponseItem::Message {
            id: None,
            role: "assistant".to_string(),
            content: vec![ContentItem::OutputText {
                text: text.to_string(),
            }],
        }
    }

    fn user_msg(text: &str) -> ResponseItem {
        ResponseItem::Message {
            id: None,
            role: "user".to_string(),
            content: vec![ContentItem::OutputText {
                text: text.to_string(),
            }],
        }
    }

    #[test]
    fn merges_adjacent_assistant_messages() {
        let mut h = ConversationHistory::default();
        let a1 = assistant_msg("Hello");
        let a2 = assistant_msg(", world!");
        h.record_items([&a1, &a2]);

        let items = h.contents();
        assert_eq!(
            items,
            vec![ResponseItem::Message {
                id: None,
                role: "assistant".to_string(),
                content: vec![ContentItem::OutputText {
                    text: "Hello, world!".to_string()
                }]
            }]
        );
    }

    #[test]
    fn append_assistant_text_creates_and_appends() {
        let mut h = ConversationHistory::default();
        h.append_assistant_text("Hello");
        h.append_assistant_text(", world");

        // Now record a final full assistant message and verify it merges.
        let final_msg = assistant_msg("!");
        h.record_items([&final_msg]);

        let items = h.contents();
        assert_eq!(
            items,
            vec![ResponseItem::Message {
                id: None,
                role: "assistant".to_string(),
                content: vec![ContentItem::OutputText {
                    text: "Hello, world!".to_string()
                }]
            }]
        );
    }

    #[test]
    fn filters_non_api_messages() {
        let mut h = ConversationHistory::default();
        // System message is not an API message; Other is ignored.
        let system = ResponseItem::Message {
            id: None,
            role: "system".to_string(),
            content: vec![ContentItem::OutputText {
                text: "ignored".to_string(),
            }],
        };
        h.record_items([&system, &ResponseItem::Other]);

        // User and assistant should be retained.
        let u = user_msg("hi");
        let a = assistant_msg("hello");
        h.record_items([&u, &a]);

        let items = h.contents();
        assert_eq!(
            items,
            vec![
                ResponseItem::Message {
                    id: None,
                    role: "user".to_string(),
                    content: vec![ContentItem::OutputText {
                        text: "hi".to_string()
                    }]
                },
                ResponseItem::Message {
                    id: None,
                    role: "assistant".to_string(),
                    content: vec![ContentItem::OutputText {
                        text: "hello".to_string()
                    }]
                }
            ]
        );
    }
}
