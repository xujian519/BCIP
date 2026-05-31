//! Detection, extraction, and rendering of `codex_follow_up` JSON blocks
//! embedded at the end of agent markdown messages.
//!
//! These blocks are produced by the AGENTS.md Follow-up tweak and contain
//! actionable follow-up prompts. This module strips them from the raw
//! markdown before regular rendering and renders them as a styled section.

use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::text::Span;
use serde::Deserialize;
use unicode_width::UnicodeWidthStr;

/// Follow-up payload as defined by the Codex++ Follow-up tweak.
#[derive(Debug, Deserialize)]
pub(crate) struct FollowUp {
    pub codex_follow_up: bool,
    pub title: String,
    pub items: Vec<FollowUpItem>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct FollowUpItem {
    pub prompt: String,
}

/// Result of extracting a follow-up block from markdown source.
pub(crate) struct ExtractedFollowUp {
    /// The original markdown with the follow-up fenced block removed.
    pub clean_markdown: String,
    /// Parsed follow-up data, if a valid block was found.
    pub follow_up: Option<FollowUp>,
}

/// Try to extract a `codex_follow_up` JSON block from the end of a markdown
/// string. Returns cleaned markdown and the parsed payload if found.
pub(crate) fn extract_follow_up(markdown_source: &str) -> ExtractedFollowUp {
    let Some(block) = find_last_json_fence(markdown_source) else {
        return ExtractedFollowUp {
            clean_markdown: markdown_source.to_string(),
            follow_up: None,
        };
    };

    match serde_json::from_str::<FollowUp>(&block.json_content) {
        Ok(follow_up) if follow_up.codex_follow_up => {
            let clean = strip_block(markdown_source, &block);
            ExtractedFollowUp {
                clean_markdown: clean,
                follow_up: Some(follow_up),
            }
        }
        _ => ExtractedFollowUp {
            clean_markdown: markdown_source.to_string(),
            follow_up: None,
        },
    }
}

struct FenceBlock {
    opening_start: usize,
    closing_end: usize,
    json_content: String,
}

/// Find the last fenced code block with "json" language tag in the source.
fn find_last_json_fence(source: &str) -> Option<FenceBlock> {
    let mut last_fence: Option<FenceBlock> = None;
    let mut pos = 0usize;

    while let Some(rel_start) = source[pos..].find("```") {
        let abs_start = pos + rel_start;
        let after_fence = &source[abs_start + 3..];

        let opening_end = after_fence
            .find('\n')
            .map(|n| abs_start + 3 + n + 1)
            .unwrap_or(source.len());

        let info_start = abs_start + 3;
        if info_start >= opening_end {
            pos = opening_end;
            continue;
        }
        let info_end = opening_end
            .checked_sub(1)
            .filter(|&end| end >= info_start && source.as_bytes().get(end) == Some(&b'\n'))
            .unwrap_or(opening_end);
        if info_start >= info_end {
            pos = opening_end;
            continue;
        }
        let info = source[info_start..info_end].trim();

        if info != "json" {
            pos = opening_end;
            continue;
        }

        let body_start = opening_end;
        let Some(closing_rel) = source[body_start..].find("\n```") else {
            pos = body_start;
            continue;
        };

        let closing_start = body_start + closing_rel + 1;
        let closing_end = if source.as_bytes().get(closing_start + 3) == Some(&b'\n') {
            closing_start + 4
        } else {
            closing_start + 3
        };

        let json_content = source[body_start..closing_start].to_string();

        last_fence = Some(FenceBlock {
            opening_start: abs_start,
            closing_end,
            json_content,
        });

        pos = closing_end;
    }

    last_fence
}

/// Strip the identified fenced block from the source, preserving
/// surrounding whitespace sensibly.
fn strip_block(source: &str, block: &FenceBlock) -> String {
    let mut result = String::with_capacity(source.len());
    result.push_str(&source[..block.opening_start]);

    let trimmed = result.trim_end().len();
    result.truncate(trimmed);

    result.push_str(&source[block.closing_end..]);
    result
}

/// Render follow-up items as styled lines within an agent message.
///
/// The rendering matches the agent message prefix style: the first line
/// gets a "• " prefix (applied by the caller via `prefix_lines`), but we
/// already output a styled bordered box so the caller's prefix is not
/// applied to follow-up lines. Instead, we use a consistent "  " left
/// margin to align with the agent message indentation.
pub(crate) fn render_follow_up(follow_up: &FollowUp, width: u16) -> Vec<Line<'static>> {
    // Usable content width after 2-char message prefix.
    let inner = width.saturating_sub(4).max(20) as usize;
    let mut lines: Vec<Line<'static>> = Vec::new();

    // Top border: "  ┌─ Title ─────────────┐"
    let title_label = format!(" {} ", follow_up.title);
    let title_w = UnicodeWidthStr::width(title_label.as_str());
    let bar_len = inner.saturating_sub(1).saturating_sub(title_w);
    lines.push(Line::from(vec![
        "  ┌─".dim(),
        title_label.cyan().bold(),
        "─".repeat(bar_len).dim(),
        "┐".dim(),
    ]));

    // Items.
    for (i, item) in follow_up.items.iter().enumerate() {
        let num = format!(" {}. ", i + 1);
        let num_w = UnicodeWidthStr::width(num.as_str());
        let avail = inner.saturating_sub(2).saturating_sub(num_w).max(20);
        let wrapped = textwrap::fill(&item.prompt, avail);

        for (j, wline) in wrapped.lines().enumerate() {
            if j == 0 {
                lines.push(Line::from(vec![
                    "  │ ".dim(),
                    Span::from(num.clone()),
                    wline.to_string().into(),
                ]));
            } else {
                lines.push(Line::from(vec![
                    "  │ ".dim(),
                    " ".repeat(num_w).into(),
                    wline.to_string().dim(),
                ]));
            }
        }
    }

    // Bottom border.
    lines.push(Line::from(vec![
        "  └".dim(),
        "─".repeat(inner).dim(),
        "┘".dim(),
    ]));

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_follow_up_basic() {
        let src = concat!(
            "Some message.\n\n",
            "```json\n",
            "{\n",
            "  \"codex_follow_up\": true,\n",
            "  \"title\": \"Follow-up\",\n",
            "  \"items\": [\n",
            "    { \"prompt\": \"Do thing A\" },\n",
            "    { \"prompt\": \"Do thing B\" }\n",
            "  ]\n",
            "}\n",
            "```\n",
        );
        let result = extract_follow_up(src);
        assert!(result.follow_up.is_some());
        let fu = result.follow_up.unwrap();
        assert_eq!(fu.title, "Follow-up");
        assert_eq!(fu.items.len(), 2);
        assert_eq!(fu.items[0].prompt, "Do thing A");
        assert!(!result.clean_markdown.contains("codex_follow_up"));
        assert!(result.clean_markdown.contains("Some message"));
    }

    #[test]
    fn test_no_follow_up_for_non_json_fence() {
        let src = "```rust\nfn main() {}\n```\n";
        let result = extract_follow_up(src);
        assert!(result.follow_up.is_none());
    }

    #[test]
    fn test_no_follow_up_for_non_matching_json() {
        let src = "```json\n{\"foo\": 1}\n```\n";
        let result = extract_follow_up(src);
        assert!(result.follow_up.is_none());
    }

    #[test]
    fn test_extract_renders_lines() {
        let fu = FollowUp {
            codex_follow_up: true,
            title: "Follow-up".into(),
            items: vec![
                FollowUpItem {
                    prompt: "Check the logs".into(),
                },
                FollowUpItem {
                    prompt: "Run the tests".into(),
                },
            ],
        };
        let lines = render_follow_up(&fu, 80);
        assert!(!lines.is_empty());
        assert!(lines.len() >= 4);
    }

    #[test]
    fn test_follow_up_title_in_output() {
        let fu = FollowUp {
            codex_follow_up: true,
            title: "Next Steps".into(),
            items: vec![FollowUpItem {
                prompt: "Verify".into(),
            }],
        };
        let lines = render_follow_up(&fu, 80);
        let text: Vec<String> = lines
            .iter()
            .map(|l| {
                l.spans
                    .iter()
                    .map(|s| s.content.as_ref())
                    .collect::<String>()
            })
            .collect();
        let combined = text.join("\n");
        assert!(combined.contains("Next Steps"));
        assert!(combined.contains("Verify"));
    }
}
