#[test]
fn large_paste_numbering_reuses_after_ctrl_c_clear() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    let paste = "x".repeat(LARGE_PASTE_CHAR_THRESHOLD + 4);
    let base = format!("[Pasted ~{} chars]", paste.chars().count());

    composer.handle_paste(paste.clone());
    assert_eq!(composer.draft.textarea.text(), base);
    assert_eq!(composer.draft.pending_pastes.len(), 1);

    assert_eq!(composer.clear_for_ctrl_c(), Some(base.clone()));
    assert!(composer.draft.textarea.text().is_empty());
    assert!(composer.draft.pending_pastes.is_empty());

    composer.handle_paste(paste);
    assert_eq!(composer.draft.textarea.text(), base);
    assert_eq!(composer.draft.pending_pastes.len(), 1);
    assert_eq!(composer.draft.pending_pastes[0].0, base);
}
#[test]
fn ascii_prefix_survives_non_ascii_followup() {
    use crossterm::event::KeyCode;
    use crossterm::event::KeyEvent;
    use crossterm::event::KeyModifiers;

    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    let _ = composer.handle_key_event(KeyEvent::new(KeyCode::Char('1'), KeyModifiers::NONE));
    assert!(composer.is_in_paste_burst());

    let _ = composer.handle_key_event(KeyEvent::new(KeyCode::Char('あ'), KeyModifiers::NONE));

    let (result, _) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    match result {
        InputResult::Submitted { text, .. } => assert_eq!(text, "1あ"),
        _ => panic!("expected Submitted"),
    }
}
#[test]
fn non_ascii_char_inserts_immediately_without_burst_state() {
    use crossterm::event::KeyCode;
    use crossterm::event::KeyEvent;
    use crossterm::event::KeyModifiers;

    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    let _ = composer.handle_key_event(KeyEvent::new(KeyCode::Char('あ'), KeyModifiers::NONE));

    assert_eq!(composer.draft.textarea.text(), "あ");
    assert!(!composer.is_in_paste_burst());
}
#[test]
fn non_ascii_burst_buffers_enter_and_flushes_multiline() {
    use crossterm::event::KeyCode;
    use crossterm::event::KeyEvent;
    use crossterm::event::KeyModifiers;

    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    composer
        .draft
        .paste_burst
        .begin_with_retro_grabbed(String::new(), Instant::now());

    let _ = composer.handle_key_event(KeyEvent::new(KeyCode::Char('你'), KeyModifiers::NONE));
    let _ = composer.handle_key_event(KeyEvent::new(KeyCode::Char('好'), KeyModifiers::NONE));
    let _ = composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    let _ = composer.handle_key_event(KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE));
    let _ = composer.handle_key_event(KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE));

    assert!(composer.draft.textarea.text().is_empty());
    let _ = flush_after_paste_burst(&mut composer);
    assert_eq!(composer.draft.textarea.text(), "你好\nhi");
}
#[test]
fn non_ascii_burst_preserves_ideographic_space_and_ascii() {
    use crossterm::event::KeyCode;
    use crossterm::event::KeyEvent;
    use crossterm::event::KeyModifiers;

    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    composer
        .draft
        .paste_burst
        .begin_with_retro_grabbed(String::new(), Instant::now());

    for ch in ['你', '　', '好'] {
        let _ = composer.handle_key_event(KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE));
    }
    let _ = composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    for ch in ['h', 'i'] {
        let _ = composer.handle_key_event(KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE));
    }

    assert!(composer.draft.textarea.text().is_empty());
    let _ = flush_after_paste_burst(&mut composer);
    assert_eq!(composer.draft.textarea.text(), "你　好\nhi");
}
#[test]
fn non_ascii_burst_buffers_large_multiline_mixed_ascii_and_unicode() {
    use crossterm::event::KeyCode;
    use crossterm::event::KeyEvent;
    use crossterm::event::KeyModifiers;

    const LARGE_MIXED_PAYLOAD: &str = "天地玄黄 宇宙洪荒\n\
日月盈昃 辰宿列张\n\
寒来暑往 秋收冬藏\n\
\n\
你好世界 编码测试\n\
汉字处理 UTF-8\n\
终端显示 正确无误\n\
\n\
风吹竹林 月照大江\n\
白云千载 青山依旧\n\
程序员 与 Unicode 同行";

    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    // Force an active burst so the test doesn't depend on timing heuristics.
    composer
        .draft
        .paste_burst
        .begin_with_retro_grabbed(String::new(), Instant::now());

    for ch in LARGE_MIXED_PAYLOAD.chars() {
        let code = if ch == '\n' {
            KeyCode::Enter
        } else {
            KeyCode::Char(ch)
        };
        let _ = composer.handle_key_event(KeyEvent::new(code, KeyModifiers::NONE));
    }

    assert!(composer.draft.textarea.text().is_empty());
    let _ = flush_after_paste_burst(&mut composer);
    assert_eq!(composer.draft.textarea.text(), LARGE_MIXED_PAYLOAD);
}
#[test]
fn ascii_burst_treats_enter_as_newline() {
    use crossterm::event::KeyCode;
    use crossterm::event::KeyEvent;
    use crossterm::event::KeyModifiers;

    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    let mut now = Instant::now();
    let step = Duration::from_millis(1);

    let _ = composer.handle_input_basic_with_time(
        KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE),
        now,
    );
    now += step;
    let _ = composer.handle_input_basic_with_time(
        KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE),
        now,
    );
    now += step;

    let (result, _) = composer.handle_submission_with_time(/*should_queue*/ false, now);
    assert!(
        matches!(result, InputResult::None),
        "Enter during a burst should insert newline, not submit"
    );

    for ch in ['t', 'h', 'e', 'r', 'e'] {
        now += step;
        let _ = composer.handle_input_basic_with_time(
            KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE),
            now,
        );
    }

    assert!(composer.draft.textarea.text().is_empty());
    let flush_time = now + PasteBurst::recommended_active_flush_delay() + step;
    let flushed = composer.handle_paste_burst_flush(flush_time);
    assert!(flushed, "expected paste burst to flush");
    assert_eq!(composer.draft.textarea.text(), "hi\nthere");
}
#[test]
fn queued_submission_flushes_ascii_burst_instead_of_inserting_newline() {
    use crossterm::event::KeyCode;
    use crossterm::event::KeyEvent;
    use crossterm::event::KeyModifiers;

    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    let mut now = Instant::now();
    let step = Duration::from_millis(1);
    for ch in ['h', 'i'] {
        let _ = composer.handle_input_basic_with_time(
            KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE),
            now,
        );
        now += step;
    }
    assert!(composer.is_in_paste_burst());

    let (result, _) = composer.handle_submission_with_time(/*should_queue*/ true, now);

    assert_eq!(
        result,
        InputResult::Queued {
            text: "hi".to_string(),
            text_elements: Vec::new(),
            action: QueuedInputAction::Plain,
        }
    );
    assert!(composer.draft.textarea.text().is_empty());
    assert!(!composer.is_in_paste_burst());
}
#[test]
fn slash_context_enter_ignores_paste_burst_enter_suppression() {
    use crate::slash_command::SlashCommand;
    use crossterm::event::KeyCode;
    use crossterm::event::KeyEvent;
    use crossterm::event::KeyModifiers;

    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    composer.draft.textarea.set_text_clearing_elements("/diff");
    composer.draft.textarea.set_cursor("/diff".len());
    composer
        .draft
        .paste_burst
        .begin_with_retro_grabbed(String::new(), Instant::now());

    let (result, _) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert!(matches!(result, InputResult::Command(SlashCommand::Diff)));
}
#[test]
fn non_char_key_flushes_active_burst_before_input() {
    use crossterm::event::KeyCode;
    use crossterm::event::KeyEvent;
    use crossterm::event::KeyModifiers;

    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    // Force an active burst so we can deterministically buffer characters without relying on
    // timing.
    composer
        .draft
        .paste_burst
        .begin_with_retro_grabbed(String::new(), Instant::now());

    let _ = composer.handle_key_event(KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE));
    let _ = composer.handle_key_event(KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE));
    assert!(composer.draft.textarea.text().is_empty());
    assert!(composer.is_in_paste_burst());

    let _ = composer.handle_key_event(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE));
    assert_eq!(composer.draft.textarea.text(), "hi");
    assert_eq!(composer.draft.textarea.cursor(), 1);
    assert!(!composer.is_in_paste_burst());
}
#[test]
fn disable_paste_burst_flushes_pending_first_char_and_inserts_immediately() {
    use crossterm::event::KeyCode;
    use crossterm::event::KeyEvent;
    use crossterm::event::KeyModifiers;

    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    // First ASCII char is normally held briefly. Flip the config mid-stream and ensure the
    // held char is not dropped.
    let _ = composer.handle_key_event(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));
    assert!(composer.is_in_paste_burst());
    assert!(composer.draft.textarea.text().is_empty());

    composer.set_disable_paste_burst(/*disabled*/ true);
    assert_eq!(composer.draft.textarea.text(), "a");
    assert!(!composer.is_in_paste_burst());

    let _ = composer.handle_key_event(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE));
    assert_eq!(composer.draft.textarea.text(), "ab");
    assert!(!composer.is_in_paste_burst());
}
#[test]
fn handle_paste_small_inserts_text() {
    use crossterm::event::KeyCode;
    use crossterm::event::KeyEvent;
    use crossterm::event::KeyModifiers;

    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    let needs_redraw = composer.handle_paste("hello".to_string());
    assert!(needs_redraw);
    assert_eq!(composer.draft.textarea.text(), "hello");
    assert!(composer.draft.pending_pastes.is_empty());

    let (result, _) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    match result {
        InputResult::Submitted { text, .. } => assert_eq!(text, "hello"),
        _ => panic!("expected Submitted"),
    }
}
#[test]
fn handle_paste_large_uses_placeholder_and_replaces_on_submit() {
    use crossterm::event::KeyCode;
    use crossterm::event::KeyEvent;
    use crossterm::event::KeyModifiers;

    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    let large = "x".repeat(LARGE_PASTE_CHAR_THRESHOLD + 10);
    let needs_redraw = composer.handle_paste(large.clone());
    assert!(needs_redraw);
    let placeholder = format!("[Pasted ~{} chars]", large.chars().count());
    assert_eq!(composer.draft.textarea.text(), placeholder);
    assert_eq!(composer.draft.pending_pastes.len(), 1);
    assert_eq!(composer.draft.pending_pastes[0].0, placeholder);
    assert_eq!(composer.draft.pending_pastes[0].1, large);

    let (result, _) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    match result {
        InputResult::Submitted { text, .. } => assert_eq!(text, large),
        _ => panic!("expected Submitted"),
    }
    assert!(composer.draft.pending_pastes.is_empty());
}
#[test]
fn large_paste_numbering_continues_with_same_length_placeholder() {
    use crossterm::event::KeyCode;
    use crossterm::event::KeyEvent;
    use crossterm::event::KeyModifiers;

    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    let paste = "x".repeat(LARGE_PASTE_CHAR_THRESHOLD + 4);
    let base = format!("[Pasted ~{} chars]", paste.chars().count());
    let second = format!("{base} #2");
    let third = format!("{base} #3");

    composer.handle_paste(paste.clone());
    composer.handle_paste(paste.clone());
    assert_eq!(composer.draft.textarea.text(), format!("{base}{second}"));

    composer.draft.textarea.set_cursor(base.len());
    composer.handle_key_event(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
    assert_eq!(composer.draft.textarea.text(), second);
    assert_eq!(composer.draft.pending_pastes.len(), 1);
    assert_eq!(composer.draft.pending_pastes[0].0, second);

    composer
        .draft
        .textarea
        .set_cursor(composer.draft.textarea.text().len());
    composer.handle_paste(paste);

    assert_eq!(composer.draft.textarea.text(), format!("{second}{third}"));
    assert_eq!(composer.draft.pending_pastes.len(), 2);
    assert_eq!(composer.draft.pending_pastes[0].0, second);
    assert_eq!(composer.draft.pending_pastes[1].0, third);
}
#[test]
fn large_paste_numbering_reuses_after_all_deleted() {
    use crossterm::event::KeyCode;
    use crossterm::event::KeyEvent;
    use crossterm::event::KeyModifiers;

    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    let paste = "x".repeat(LARGE_PASTE_CHAR_THRESHOLD + 4);
    let base = format!("[Pasted ~{} chars]", paste.chars().count());

    composer.handle_paste(paste.clone());
    assert_eq!(composer.draft.textarea.text(), base);
    assert_eq!(composer.draft.pending_pastes.len(), 1);

    composer
        .draft
        .textarea
        .set_cursor(composer.draft.textarea.text().len());
    composer.handle_key_event(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
    assert!(composer.draft.textarea.text().is_empty());
    assert!(composer.draft.pending_pastes.is_empty());

    composer.handle_paste(paste);
    assert_eq!(composer.draft.textarea.text(), base);
    assert_eq!(composer.draft.pending_pastes.len(), 1);
    assert_eq!(composer.draft.pending_pastes[0].0, base);
}
#[test]
fn large_paste_preserves_image_text_elements_on_submit() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    let large_content = "x".repeat(LARGE_PASTE_CHAR_THRESHOLD + 5);
    composer.handle_paste(large_content.clone());
    composer.handle_paste(" ".into());
    let path = PathBuf::from("/tmp/image_with_paste.png");
    composer.attach_image(path.clone());

    let (result, _) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    match result {
        InputResult::Submitted {
            text,
            text_elements,
        } => {
            let expected = format!("{large_content} [Image #1]");
            assert_eq!(text, expected);
            assert_eq!(text_elements.len(), 1);
            assert_eq!(text_elements[0].placeholder(&text), Some("[Image #1]"));
            assert_eq!(
                text_elements[0].byte_range,
                ByteRange {
                    start: large_content.len() + 1,
                    end: large_content.len() + 1 + "[Image #1]".len(),
                }
            );
        }
        _ => panic!("expected Submitted"),
    }
    let imgs = composer.take_recent_submission_images();
    assert_eq!(vec![path], imgs);
}
#[test]
fn large_paste_with_leading_whitespace_trims_and_shifts_elements() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    let large_content = format!("  {}", "x".repeat(LARGE_PASTE_CHAR_THRESHOLD + 5));
    composer.handle_paste(large_content.clone());
    composer.handle_paste(" ".into());
    let path = PathBuf::from("/tmp/image_with_trim.png");
    composer.attach_image(path.clone());

    let (result, _) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    match result {
        InputResult::Submitted {
            text,
            text_elements,
        } => {
            let trimmed = large_content.trim().to_string();
            assert_eq!(text, format!("{trimmed} [Image #1]"));
            assert_eq!(text_elements.len(), 1);
            assert_eq!(text_elements[0].placeholder(&text), Some("[Image #1]"));
            assert_eq!(
                text_elements[0].byte_range,
                ByteRange {
                    start: trimmed.len() + 1,
                    end: trimmed.len() + 1 + "[Image #1]".len(),
                }
            );
        }
        _ => panic!("expected Submitted"),
    }
    let imgs = composer.take_recent_submission_images();
    assert_eq!(vec![path], imgs);
}
#[test]
fn pending_first_ascii_char_flushes_as_typed() {
    use crossterm::event::KeyCode;
    use crossterm::event::KeyEvent;
    use crossterm::event::KeyModifiers;

    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    let _ = composer.handle_key_event(KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE));
    assert!(composer.is_in_paste_burst());
    assert!(composer.draft.textarea.text().is_empty());

    std::thread::sleep(ChatComposer::recommended_paste_flush_delay());
    let flushed = composer.flush_paste_burst_if_due();
    assert!(flushed, "expected pending first char to flush");
    assert_eq!(composer.draft.textarea.text(), "h");
    assert!(!composer.is_in_paste_burst());
}
#[test]
fn burst_paste_fast_small_buffers_and_flushes_on_stop() {
    use crossterm::event::KeyCode;
    use crossterm::event::KeyEvent;
    use crossterm::event::KeyModifiers;

    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    let count = 32;
    let mut now = Instant::now();
    let step = Duration::from_millis(1);
    for _ in 0..count {
        let _ = composer.handle_input_basic_with_time(
            KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE),
            now,
        );
        assert!(
            composer.is_in_paste_burst(),
            "expected active paste burst during fast typing"
        );
        assert!(
            composer.draft.textarea.text().is_empty(),
            "text should not appear during burst"
        );
        now += step;
    }

    assert!(
        composer.draft.textarea.text().is_empty(),
        "text should remain empty until flush"
    );
    let flush_time = now + PasteBurst::recommended_active_flush_delay() + step;
    let flushed = composer.handle_paste_burst_flush(flush_time);
    assert!(flushed, "expected buffered text to flush after stop");
    assert_eq!(composer.draft.textarea.text(), "a".repeat(count));
    assert!(
        composer.draft.pending_pastes.is_empty(),
        "no placeholder for small burst"
    );
}
#[test]
fn burst_paste_fast_large_inserts_placeholder_on_flush() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    let count = LARGE_PASTE_CHAR_THRESHOLD + 1; // > threshold to trigger placeholder
    let mut now = Instant::now();
    let step = Duration::from_millis(1);
    for _ in 0..count {
        let _ = composer.handle_input_basic_with_time(
            KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE),
            now,
        );
        now += step;
    }

    // Nothing should appear until we stop and flush
    assert!(composer.draft.textarea.text().is_empty());
    let flush_time = now + PasteBurst::recommended_active_flush_delay() + step;
    let flushed = composer.handle_paste_burst_flush(flush_time);
    assert!(flushed, "expected flush after stopping fast input");

    let expected_placeholder = format!("[Pasted ~{count} chars]");
    assert_eq!(composer.draft.textarea.text(), expected_placeholder);
    assert_eq!(composer.draft.pending_pastes.len(), 1);
    assert_eq!(composer.draft.pending_pastes[0].0, expected_placeholder);
    assert_eq!(composer.draft.pending_pastes[0].1.len(), count);
    assert!(composer.draft.pending_pastes[0].1.chars().all(|c| c == 'x'));
}
#[test]
fn humanlike_typing_1000_chars_appears_live_no_placeholder() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    let count = LARGE_PASTE_CHAR_THRESHOLD; // 1000 in current config
    let chars: Vec<char> = vec!['z'; count];
    type_chars_humanlike(&mut composer, &chars);

    assert_eq!(composer.draft.textarea.text(), "z".repeat(count));
    assert!(composer.draft.pending_pastes.is_empty());
}