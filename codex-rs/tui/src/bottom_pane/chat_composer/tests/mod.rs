use super::attachment_state::AttachedImage;
use super::*;
use crate::test_support::PathBufExt;
use crate::test_support::test_path_buf;
use image::ImageBuffer;
use image::Rgba;
use pretty_assertions::assert_eq;
use std::path::PathBuf;
use tempfile::tempdir;

use crate::app_event::AppEvent;

use crate::bottom_pane::AppEventSender;
use crate::bottom_pane::ChatComposer;
use crate::bottom_pane::InputResult;
use crate::bottom_pane::chat_composer::LARGE_PASTE_CHAR_THRESHOLD;
use crate::bottom_pane::textarea::TextArea;
use codex_protocol::models::local_image_label_text;
use tokio::sync::mpsc::unbounded_channel;

include!("basics.rs");

include!("footer.rs");

include!("history.rs");

include!("images.rs");

include!("mentions.rs");

include!("paste.rs");

include!("slash_commands.rs");

include!("submission.rs");

include!("vim.rs");
