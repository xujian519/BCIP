#[test]
fn slash_opens_command_popup_in_vim_normal_mode() {
    use crossterm::event::KeyCode;
    use crossterm::event::KeyEvent;
    use crossterm::event::KeyModifiers;

    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ true,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ true,
    );
    composer.set_vim_enabled(/*enabled*/ true);

    let (result, needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE));

    assert!(matches!(result, InputResult::None));
    assert!(needs_redraw);
    assert_eq!(composer.draft.textarea.text(), "/");
    assert_eq!(composer.draft.textarea.cursor(), "/".len());
    assert!(matches!(composer.popups.active, ActivePopup::Command(_)));
    assert_eq!(
        composer.vim_mode_indicator_span(),
        Some("Vim: Insert".green())
    );
}
#[test]
fn inline_slash_command_dispatch_resets_vim_mode_to_normal() {
    use crossterm::event::KeyCode;
    use crossterm::event::KeyEvent;
    use crossterm::event::KeyModifiers;

    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ true,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ true,
    );
    composer.set_collaboration_modes_enabled(/*enabled*/ true);
    composer.set_vim_enabled(/*enabled*/ true);

    composer.handle_key_event(KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE));
    composer.set_text_content("/plan investigate this".to_string(), Vec::new(), Vec::new());
    composer.popups.active = ActivePopup::None;
    let (result, needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    assert!(needs_redraw);
    assert_eq!(
        composer.vim_mode_indicator_span(),
        Some("Vim: Normal".magenta())
    );
    match result {
        InputResult::CommandWithArgs(cmd, args, text_elements) => {
            assert_eq!(cmd, SlashCommand::Plan);
            assert_eq!(args, "investigate this");
            assert!(text_elements.is_empty());
        }
        _ => panic!("expected CommandWithArgs"),
    }
}
#[test]
fn bang_enters_shell_mode_in_vim_normal_mode() {
    use crossterm::event::KeyCode;
    use crossterm::event::KeyEvent;
    use crossterm::event::KeyModifiers;

    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ true,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ true,
    );
    composer.set_vim_enabled(/*enabled*/ true);

    let (result, needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Char('!'), KeyModifiers::NONE));

    assert!(matches!(result, InputResult::None));
    assert!(needs_redraw);
    assert!(composer.draft.is_bash_mode);
    assert_eq!(composer.current_text(), "!");
    assert_eq!(composer.draft.textarea.text(), "");
    assert_eq!(
        composer.vim_mode_indicator_span(),
        Some("Vim: Insert".green())
    );
}
#[test]
fn shell_command_can_be_typed_after_vim_normal_bang() {
    use crossterm::event::KeyCode;
    use crossterm::event::KeyEvent;
    use crossterm::event::KeyModifiers;

    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ true,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ true,
    );
    composer.set_vim_enabled(/*enabled*/ true);

    for ch in ['!', 'e', 'c', 'h', 'o'] {
        let _ = composer.handle_key_event(KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE));
    }

    assert!(composer.draft.is_bash_mode);
    assert_eq!(composer.current_text(), "!echo");
    assert_eq!(composer.draft.textarea.text(), "echo");
    assert!(matches!(composer.popups.active, ActivePopup::None));
}
#[test]
fn clear_for_ctrl_c_records_cleared_draft() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    composer.set_text_content("draft text".to_string(), Vec::new(), Vec::new());
    assert_eq!(composer.clear_for_ctrl_c(), Some("draft text".to_string()));
    assert!(composer.is_empty());

    assert_eq!(
        composer.history.navigate_up(&composer.app_event_tx),
        Some(HistoryEntry::new("draft text".to_string()))
    );
}
#[test]
fn clear_for_ctrl_c_preserves_pending_paste_history_entry() {
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

    let large = "x".repeat(LARGE_PASTE_CHAR_THRESHOLD + 5);
    composer.handle_paste(large.clone());
    let char_count = large.chars().count();
    let placeholder = format!("[Pasted ~{char_count} chars]");
    assert_eq!(composer.draft.textarea.text(), placeholder);
    assert_eq!(
        composer.draft.pending_pastes,
        vec![(placeholder.clone(), large.clone())]
    );

    composer.clear_for_ctrl_c();
    assert!(composer.is_empty());

    let history_entry = composer
        .history
        .navigate_up(&composer.app_event_tx)
        .expect("expected history entry");
    let text_elements = vec![TextElement::new(
        (0..placeholder.len()).into(),
        Some(placeholder.clone()),
    )];
    assert_eq!(
        history_entry,
        HistoryEntry::with_pending(
            placeholder.clone(),
            text_elements,
            Vec::new(),
            vec![(placeholder.clone(), large.clone())]
        )
    );

    composer.apply_history_entry(history_entry);
    assert_eq!(composer.draft.textarea.text(), placeholder);
    assert_eq!(
        composer.draft.pending_pastes,
        vec![(placeholder.clone(), large)]
    );
    assert_eq!(
        composer.draft.textarea.element_payloads(),
        vec![placeholder]
    );

    let (result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    match result {
        InputResult::Submitted {
            text,
            text_elements,
        } => {
            assert_eq!(text, "x".repeat(LARGE_PASTE_CHAR_THRESHOLD + 5));
            assert!(text_elements.is_empty());
        }
        _ => panic!("expected Submitted"),
    }
}
#[test]
fn clear_for_ctrl_c_preserves_image_draft_state() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    let path = PathBuf::from("example.png");
    composer.attach_image(path.clone());
    let placeholder = local_image_label_text(/*label_number*/ 1);

    composer.clear_for_ctrl_c();
    assert!(composer.is_empty());

    let history_entry = composer
        .history
        .navigate_up(&composer.app_event_tx)
        .expect("expected history entry");
    let text_elements = vec![TextElement::new(
        (0..placeholder.len()).into(),
        Some(placeholder.clone()),
    )];
    assert_eq!(
        history_entry,
        HistoryEntry::with_pending(
            placeholder.clone(),
            text_elements,
            vec![path.clone()],
            Vec::new()
        )
    );

    composer.apply_history_entry(history_entry);
    assert_eq!(composer.draft.textarea.text(), placeholder);
    assert_eq!(composer.local_image_paths(), vec![path]);
    assert_eq!(
        composer.draft.textarea.element_payloads(),
        vec![placeholder]
    );
}
#[test]
fn clear_for_ctrl_c_preserves_remote_offset_image_labels() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );
    let remote_image_url = "https://example.com/one.png".to_string();
    composer.set_remote_image_urls(vec![remote_image_url.clone()]);
    let text = "[Image #2] draft".to_string();
    let text_elements = vec![TextElement::new(
        (0.."[Image #2]".len()).into(),
        Some("[Image #2]".to_string()),
    )];
    let local_image_path = PathBuf::from("/tmp/local-draft.png");
    composer.set_text_content(text, text_elements, vec![local_image_path.clone()]);
    let expected_text = composer.current_text();
    let expected_elements = composer.text_elements();
    assert_eq!(expected_text, "[Image #2] draft");
    assert_eq!(
        expected_elements[0].placeholder(&expected_text),
        Some("[Image #2]")
    );

    assert_eq!(composer.clear_for_ctrl_c(), Some(expected_text.clone()));

    assert_eq!(
        composer.history.navigate_up(&composer.app_event_tx),
        Some(HistoryEntry::with_pending_and_remote(
            expected_text,
            expected_elements,
            vec![local_image_path],
            Vec::new(),
            vec![remote_image_url],
        ))
    );
}
#[test]
fn apply_history_entry_preserves_local_placeholders_after_remote_prefix() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    let remote_image_url = "https://example.com/one.png".to_string();
    let local_image_path = PathBuf::from("/tmp/local-draft.png");
    composer.apply_history_entry(HistoryEntry::with_pending_and_remote(
        "[Image #2] draft".to_string(),
        vec![TextElement::new(
            (0.."[Image #2]".len()).into(),
            Some("[Image #2]".to_string()),
        )],
        vec![local_image_path.clone()],
        Vec::new(),
        vec![remote_image_url.clone()],
    ));

    let restored_text = composer.current_text();
    assert_eq!(restored_text, "[Image #2] draft");
    let restored_elements = composer.text_elements();
    assert_eq!(restored_elements.len(), 1);
    assert_eq!(
        restored_elements[0].placeholder(&restored_text),
        Some("[Image #2]")
    );
    assert_eq!(composer.local_image_paths(), vec![local_image_path]);
    assert_eq!(composer.remote_image_urls(), vec![remote_image_url]);
}
#[test]
fn question_mark_only_toggles_on_first_char() {
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

    let (result, needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE));
    assert_eq!(result, InputResult::None);
    assert!(needs_redraw, "toggling overlay should request redraw");
    assert_eq!(composer.footer.mode, FooterMode::ShortcutOverlay);

    // Toggle back to prompt mode so subsequent typing captures characters.
    let _ = composer.handle_key_event(KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE));
    assert_eq!(composer.footer.mode, FooterMode::ComposerEmpty);

    type_chars_humanlike(&mut composer, &['h']);
    assert_eq!(composer.draft.textarea.text(), "h");
    assert_eq!(composer.footer_mode(), FooterMode::ComposerHasDraft);

    let (result, needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE));
    assert_eq!(result, InputResult::None);
    assert!(needs_redraw, "typing should still mark the view dirty");
    let _ = flush_after_paste_burst(&mut composer);
    assert_eq!(composer.draft.textarea.text(), "h?");
    assert_eq!(composer.footer.mode, FooterMode::ComposerEmpty);
    assert_eq!(composer.footer_mode(), FooterMode::ComposerHasDraft);
}
#[test]
fn shift_question_mark_toggles_shortcut_overlay_when_empty() {
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
    composer.set_steer_enabled(true);

    let (result, needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Char('?'), KeyModifiers::SHIFT));
    assert_eq!(result, InputResult::None);
    assert!(needs_redraw, "toggling overlay should request redraw");
    assert_eq!(composer.footer.mode, FooterMode::ShortcutOverlay);
}
#[test]
fn question_mark_does_not_toggle_during_paste_burst() {
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

    // Force an active paste burst so this test doesn't depend on tight timing.
    composer
        .draft
        .paste_burst
        .begin_with_retro_grabbed(String::new(), Instant::now());

    for ch in ['h', 'i', '?', 't', 'h', 'e', 'r', 'e'] {
        let _ = composer.handle_key_event(KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE));
    }
    assert!(composer.is_in_paste_burst());
    assert_eq!(composer.draft.textarea.text(), "");

    let _ = flush_after_paste_burst(&mut composer);

    assert_eq!(composer.draft.textarea.text(), "hi?there");
    assert_ne!(composer.footer.mode, FooterMode::ShortcutOverlay);
}
#[test]
fn shortcut_overlay_persists_while_task_running() {
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

    let _ = composer.handle_key_event(KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE));
    assert_eq!(composer.footer.mode, FooterMode::ShortcutOverlay);

    composer.set_task_running(/*running*/ true);

    assert_eq!(composer.footer.mode, FooterMode::ShortcutOverlay);
    assert_eq!(composer.footer_mode(), FooterMode::ShortcutOverlay);
}
#[test]
fn test_current_at_token_basic_cases() {
    let test_cases = vec![
        // Valid @ tokens
        ("@hello", 3, Some("hello".to_string()), "Basic ASCII token"),
        (
            "@file.txt",
            4,
            Some("file.txt".to_string()),
            "ASCII with extension",
        ),
        (
            "hello @world test",
            8,
            Some("world".to_string()),
            "ASCII token in middle",
        ),
        (
            "@test123",
            5,
            Some("test123".to_string()),
            "ASCII with numbers",
        ),
        // Unicode examples
        ("@İstanbul", 3, Some("İstanbul".to_string()), "Turkish text"),
        (
            "@testЙЦУ.rs",
            8,
            Some("testЙЦУ.rs".to_string()),
            "Mixed ASCII and Cyrillic",
        ),
        ("@诶", 2, Some("诶".to_string()), "Chinese character"),
        ("@👍", 2, Some("👍".to_string()), "Emoji token"),
        // Invalid cases (should return None)
        ("hello", 2, None, "No @ symbol"),
        (
            "@",
            1,
            Some("".to_string()),
            "Only @ symbol triggers empty query",
        ),
        ("@ hello", 2, None, "@ followed by space"),
        ("test @ world", 6, None, "@ with spaces around"),
    ];

    for (input, cursor_pos, expected, description) in test_cases {
        let mut textarea = TextArea::new();
        textarea.insert_str(input);
        textarea.set_cursor(cursor_pos);

        let result = ChatComposer::current_at_token(&textarea);
        assert_eq!(
            result, expected,
            "Failed for case: {description} - input: '{input}', cursor: {cursor_pos}"
        );
    }
}
#[test]
fn test_current_at_token_cursor_positions() {
    let test_cases = vec![
        // Different cursor positions within a token
        ("@test", 0, Some("test".to_string()), "Cursor at @"),
        ("@test", 1, Some("test".to_string()), "Cursor after @"),
        ("@test", 5, Some("test".to_string()), "Cursor at end"),
        // Multiple tokens - cursor determines which token
        ("@file1 @file2", 0, Some("file1".to_string()), "First token"),
        (
            "@file1 @file2",
            8,
            Some("file2".to_string()),
            "Second token",
        ),
        // Edge cases
        ("@", 0, Some("".to_string()), "Only @ symbol"),
        ("@a", 2, Some("a".to_string()), "Single character after @"),
        ("", 0, None, "Empty input"),
    ];

    for (input, cursor_pos, expected, description) in test_cases {
        let mut textarea = TextArea::new();
        textarea.insert_str(input);
        textarea.set_cursor(cursor_pos);

        let result = ChatComposer::current_at_token(&textarea);
        assert_eq!(
            result, expected,
            "Failed for cursor position case: {description} - input: '{input}', cursor: {cursor_pos}",
        );
    }
}
#[test]
fn test_current_at_token_whitespace_boundaries() {
    let test_cases = vec![
        // Space boundaries
        (
            "aaa@aaa",
            4,
            None,
            "Connected @ token - no completion by design",
        ),
        (
            "aaa @aaa",
            5,
            Some("aaa".to_string()),
            "@ token after space",
        ),
        (
            "test @file.txt",
            7,
            Some("file.txt".to_string()),
            "@ token after space",
        ),
        // Full-width space boundaries
        (
            "test　@İstanbul",
            8,
            Some("İstanbul".to_string()),
            "@ token after full-width space",
        ),
        (
            "@ЙЦУ　@诶",
            10,
            Some("诶".to_string()),
            "Full-width space between Unicode tokens",
        ),
        // Tab and newline boundaries
        (
            "test\t@file",
            6,
            Some("file".to_string()),
            "@ token after tab",
        ),
    ];

    for (input, cursor_pos, expected, description) in test_cases {
        let mut textarea = TextArea::new();
        textarea.insert_str(input);
        textarea.set_cursor(cursor_pos);

        let result = ChatComposer::current_at_token(&textarea);
        assert_eq!(
            result, expected,
            "Failed for whitespace boundary case: {description} - input: '{input}', cursor: {cursor_pos}",
        );
    }
}
#[test]
fn test_current_at_token_tracks_tokens_with_second_at() {
    let input = "npx -y @kaeawc/auto-mobile@latest";
    let token_start = input.find("@kaeawc").expect("scoped npm package present");
    let version_at = input
        .rfind("@latest")
        .expect("version suffix present in scoped npm package");
    let test_cases = vec![
        (token_start, "Cursor at leading @"),
        (token_start + 8, "Cursor inside scoped package name"),
        (version_at, "Cursor at version @"),
        (input.len(), "Cursor at end of token"),
    ];

    for (cursor_pos, description) in test_cases {
        let mut textarea = TextArea::new();
        textarea.insert_str(input);
        textarea.set_cursor(cursor_pos);

        let result = ChatComposer::current_at_token(&textarea);
        assert_eq!(
            result,
            Some("kaeawc/auto-mobile@latest".to_string()),
            "Failed for case: {description} - input: '{input}', cursor: {cursor_pos}"
        );
    }
}
#[test]
fn test_current_at_token_allows_file_queries_with_second_at() {
    let input = "@icons/icon@2x.png";
    let version_at = input
        .rfind("@2x")
        .expect("second @ in file token should be present");
    let test_cases = vec![
        (0, "Cursor at leading @"),
        (8, "Cursor before second @"),
        (version_at, "Cursor at second @"),
        (input.len(), "Cursor at end of token"),
    ];

    for (cursor_pos, description) in test_cases {
        let mut textarea = TextArea::new();
        textarea.insert_str(input);
        textarea.set_cursor(cursor_pos);

        let result = ChatComposer::current_at_token(&textarea);
        assert!(
            result.is_some(),
            "Failed for case: {description} - input: '{input}', cursor: {cursor_pos}"
        );
    }
}
#[test]
fn test_current_at_token_ignores_mid_word_at() {
    let input = "foo@bar";
    let at_pos = input.find('@').expect("@ present");
    let test_cases = vec![
        (at_pos, "Cursor at mid-word @"),
        (input.len(), "Cursor at end of word containing @"),
    ];

    for (cursor_pos, description) in test_cases {
        let mut textarea = TextArea::new();
        textarea.insert_str(input);
        textarea.set_cursor(cursor_pos);

        let result = ChatComposer::current_at_token(&textarea);
        assert_eq!(
            result, None,
            "Failed for case: {description} - input: '{input}', cursor: {cursor_pos}"
        );
    }
}
fn flush_after_paste_burst(composer: &mut ChatComposer) -> bool {
    std::thread::sleep(PasteBurst::recommended_active_flush_delay());
    composer.flush_paste_burst_if_due()
}
fn type_chars_humanlike(composer: &mut ChatComposer, chars: &[char]) {
    use crossterm::event::KeyCode;
    use crossterm::event::KeyEvent;
    use crossterm::event::KeyEventKind;
    use crossterm::event::KeyModifiers;
    for &ch in chars {
        let _ = composer.handle_key_event(KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE));
        std::thread::sleep(ChatComposer::recommended_paste_flush_delay());
        let _ = composer.flush_paste_burst_if_due();
        if ch == ' ' {
            let _ = composer.handle_key_event(KeyEvent::new_with_kind(
                KeyCode::Char(' '),
                KeyModifiers::NONE,
                KeyEventKind::Release,
            ));
        }
    }
}
#[test]
fn file_completion_preserves_large_paste_placeholder_elements() {
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

    let large = "x".repeat(LARGE_PASTE_CHAR_THRESHOLD + 5);
    let placeholder = format!("[Pasted ~{} chars]", large.chars().count());

    composer.handle_paste(large.clone());
    composer.insert_str(" @ma");
    composer.on_file_search_result(
        "ma".to_string(),
        vec![FileMatch {
            score: 1,
            path: PathBuf::from("src/main.rs"),
            match_type: codex_file_search::MatchType::File,
            root: PathBuf::from("/tmp"),
            indices: None,
        }],
    );

    let (_result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));

    let text = composer.draft.textarea.text().to_string();
    assert_eq!(text, format!("{placeholder} src/main.rs "));
    let elements = composer.draft.textarea.text_elements();
    assert_eq!(elements.len(), 1);
    assert_eq!(elements[0].placeholder(&text), Some(placeholder.as_str()));

    let (result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    match result {
        InputResult::Submitted {
            text,
            text_elements,
        } => {
            assert_eq!(text, format!("{large} src/main.rs"));
            assert!(text_elements.is_empty());
        }
        _ => panic!("expected Submitted"),
    }
}
#[test]
fn test_multiple_pastes_submission() {
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

    // Define test cases: (paste content, is_large)
    let test_cases = [
        ("x".repeat(LARGE_PASTE_CHAR_THRESHOLD + 3), true),
        (" and ".to_string(), false),
        ("y".repeat(LARGE_PASTE_CHAR_THRESHOLD + 7), true),
    ];

    // Expected states after each paste
    let mut expected_text = String::new();
    let mut expected_pending_count = 0;

    // Apply all pastes and build expected state
    let states: Vec<_> = test_cases
        .iter()
        .map(|(content, is_large)| {
            composer.handle_paste(content.clone());
            if *is_large {
                let placeholder = format!("[Pasted ~{} chars]", content.chars().count());
                expected_text.push_str(&placeholder);
                expected_pending_count += 1;
            } else {
                expected_text.push_str(content);
            }
            (expected_text.clone(), expected_pending_count)
        })
        .collect();

    // Verify all intermediate states were correct
    assert_eq!(
        states,
        vec![
            (
                format!("[Pasted ~{} chars]", test_cases[0].0.chars().count()),
                1
            ),
            (
                format!("[Pasted ~{} chars] and ", test_cases[0].0.chars().count()),
                1
            ),
            (
                format!(
                    "[Pasted ~{} chars] and [Pasted ~{} chars]",
                    test_cases[0].0.chars().count(),
                    test_cases[2].0.chars().count()
                ),
                2
            ),
        ]
    );

    // Submit and verify final expansion
    let (result, _) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    if let InputResult::Submitted { text, .. } = result {
        assert_eq!(text, format!("{} and {}", test_cases[0].0, test_cases[2].0));
    } else {
        panic!("expected Submitted");
    }
}
#[test]
fn test_placeholder_deletion() {
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

    // Define test cases: (content, is_large)
    let test_cases = [
        ("a".repeat(LARGE_PASTE_CHAR_THRESHOLD + 5), true),
        (" and ".to_string(), false),
        ("b".repeat(LARGE_PASTE_CHAR_THRESHOLD + 6), true),
    ];

    // Apply all pastes
    let mut current_pos = 0;
    let states: Vec<_> = test_cases
        .iter()
        .map(|(content, is_large)| {
            composer.handle_paste(content.clone());
            if *is_large {
                let placeholder = format!("[Pasted ~{} chars]", content.chars().count());
                current_pos += placeholder.len();
            } else {
                current_pos += content.len();
            }
            (
                composer.draft.textarea.text().to_string(),
                composer.draft.pending_pastes.len(),
                current_pos,
            )
        })
        .collect();

    // Delete placeholders one by one and collect states
    let mut deletion_states = vec![];

    // First deletion
    composer.draft.textarea.set_cursor(states[0].2);
    composer.handle_key_event(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
    deletion_states.push((
        composer.draft.textarea.text().to_string(),
        composer.draft.pending_pastes.len(),
    ));

    // Second deletion
    composer
        .draft
        .textarea
        .set_cursor(composer.draft.textarea.text().len());
    composer.handle_key_event(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
    deletion_states.push((
        composer.draft.textarea.text().to_string(),
        composer.draft.pending_pastes.len(),
    ));

    // Verify all states
    assert_eq!(
        deletion_states,
        vec![
            (" and [Pasted ~1006 chars]".to_string(), 1),
            (" and ".to_string(), 0),
        ]
    );
}
#[test]
fn deleting_duplicate_length_pastes_removes_only_target() {
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
    let placeholder_base = format!("[Pasted ~{} chars]", paste.chars().count());
    let placeholder_second = format!("{placeholder_base} #2");

    composer.handle_paste(paste.clone());
    composer.handle_paste(paste.clone());
    assert_eq!(
        composer.draft.textarea.text(),
        format!("{placeholder_base}{placeholder_second}")
    );
    assert_eq!(composer.draft.pending_pastes.len(), 2);

    composer
        .draft
        .textarea
        .set_cursor(composer.draft.textarea.text().len());
    composer.handle_key_event(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));

    assert_eq!(composer.draft.textarea.text(), placeholder_base);
    assert_eq!(composer.draft.pending_pastes.len(), 1);
    assert_eq!(composer.draft.pending_pastes[0].0, placeholder_base);
    assert_eq!(composer.draft.pending_pastes[0].1, paste);
}
#[test]
fn test_partial_placeholder_deletion() {
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

    // Define test cases: (cursor_position_from_end, expected_pending_count)
    let test_cases = [
        5, // Delete from middle - should clear tracking
        0, // Delete from end - should clear tracking
    ];

    let paste = "x".repeat(LARGE_PASTE_CHAR_THRESHOLD + 4);
    let placeholder = format!("[Pasted ~{} chars]", paste.chars().count());

    let states: Vec<_> = test_cases
        .into_iter()
        .map(|pos_from_end| {
            composer.handle_paste(paste.clone());
            composer
                .draft
                .textarea
                .set_cursor(placeholder.len() - pos_from_end);
            composer.handle_key_event(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
            let result = (
                composer.draft.textarea.text().contains(&placeholder),
                composer.draft.pending_pastes.len(),
            );
            composer.draft.textarea.set_text_clearing_elements("");
            result
        })
        .collect();

    assert_eq!(
        states,
        vec![
            (false, 0), // After deleting from middle
            (false, 0), // After deleting from end
        ]
    );
}
#[test]
fn set_text_content_reattaches_images_without_placeholder_metadata() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    let placeholder = local_image_label_text(/*label_number*/ 1);
    let text = format!("{placeholder} restored");
    let text_elements = vec![TextElement::new(
        (0..placeholder.len()).into(),
        /*placeholder*/ None,
    )];
    let path = PathBuf::from("/tmp/image1.png");

    composer.set_text_content(text, text_elements, vec![path.clone()]);

    assert_eq!(composer.local_image_paths(), vec![path]);
}
#[test]
fn pasted_crlf_normalizes_newlines_for_elements() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    let pasted = "line1\r\nline2\r\n".to_string();
    composer.handle_paste(pasted);
    composer.handle_paste(" ".into());
    let path = PathBuf::from("/tmp/image_crlf.png");
    composer.attach_image(path.clone());

    let (result, _) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    match result {
        InputResult::Submitted {
            text,
            text_elements,
        } => {
            assert_eq!(text, "line1\nline2\n [Image #1]");
            assert!(!text.contains('\r'));
            assert_eq!(text_elements.len(), 1);
            assert_eq!(text_elements[0].placeholder(&text), Some("[Image #1]"));
            assert_eq!(
                text_elements[0].byte_range,
                ByteRange {
                    start: "line1\nline2\n ".len(),
                    end: "line1\nline2\n [Image #1]".len(),
                }
            );
        }
        _ => panic!("expected Submitted"),
    }
    let imgs = composer.take_recent_submission_images();
    assert_eq!(vec![path], imgs);
}
#[test]
fn suppressed_submission_restores_pending_paste_payload() {
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
        .textarea
        .set_text_clearing_elements("/unknown ");
    composer.draft.textarea.set_cursor("/unknown ".len());
    let large_content = "x".repeat(LARGE_PASTE_CHAR_THRESHOLD + 5);
    composer.handle_paste(large_content.clone());
    let placeholder = composer
        .draft
        .pending_pastes
        .first()
        .expect("expected pending paste")
        .0
        .clone();

    let (result, _) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert!(matches!(result, InputResult::None));
    assert_eq!(composer.draft.pending_pastes.len(), 1);
    assert_eq!(
        composer.draft.textarea.text(),
        format!("/unknown {placeholder}")
    );

    composer.draft.textarea.set_cursor(/*pos*/ 0);
    composer.draft.textarea.insert_str(" ");
    let (result, _) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    match result {
        InputResult::Submitted {
            text,
            text_elements,
        } => {
            assert_eq!(text, format!("/unknown {large_content}"));
            assert!(text_elements.is_empty());
        }
        _ => panic!("expected Submitted"),
    }
    assert!(composer.draft.pending_pastes.is_empty());
}
#[test]
fn bare_slash_command_can_be_recalled_after_recording_pending_history() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    composer.set_text_content("/diff".to_string(), Vec::new(), Vec::new());
    let (result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    assert_eq!(result, InputResult::Command(SlashCommand::Diff));
    composer.record_pending_slash_command_history();

    let (result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
    assert_eq!(result, InputResult::None);
    assert_eq!(composer.current_text(), "/diff");
}
#[test]
fn popup_selected_slash_command_records_canonical_command_history() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    composer.set_text_content("/di".to_string(), Vec::new(), Vec::new());
    let (result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    assert_eq!(result, InputResult::Command(SlashCommand::Diff));
    composer.record_pending_slash_command_history();

    let (result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
    assert_eq!(result, InputResult::None);
    assert_eq!(composer.current_text(), "/diff");
}
#[test]
fn inline_slash_command_can_be_recalled_after_recording_pending_history() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );
    composer.set_collaboration_modes_enabled(/*enabled*/ true);

    composer.set_text_content("/plan investigate this".to_string(), Vec::new(), Vec::new());
    composer.popups.active = ActivePopup::None;
    let (result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    match result {
        InputResult::CommandWithArgs(cmd, args, text_elements) => {
            assert_eq!(cmd, SlashCommand::Plan);
            assert_eq!(args, "investigate this");
            assert!(text_elements.is_empty());
        }
        other => panic!("expected inline /plan command, got {other:?}"),
    }
    composer.record_pending_slash_command_history();

    let (result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
    assert_eq!(result, InputResult::None);
    assert_eq!(composer.current_text(), "/plan investigate this");
}
#[test]
fn apply_external_edit_rebuilds_text_and_attachments() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    let placeholder = local_image_label_text(/*label_number*/ 1);
    composer.draft.textarea.insert_element(&placeholder);
    composer.attachments.local_images.push(AttachedImage {
        placeholder: placeholder.clone(),
        path: PathBuf::from("img.png"),
    });
    composer
        .draft
        .pending_pastes
        .push(("[Pasted]".to_string(), "data".to_string()));

    composer.apply_external_edit(format!("Edited {placeholder} text"));

    assert_eq!(
        composer.current_text(),
        format!("Edited {placeholder} text")
    );
    assert!(composer.draft.pending_pastes.is_empty());
    assert_eq!(composer.attachments.local_images.len(), 1);
    assert_eq!(
        composer.attachments.local_images[0].placeholder,
        placeholder
    );
    assert_eq!(
        composer.draft.textarea.cursor(),
        composer.current_text().len()
    );
}
#[test]
fn apply_external_edit_absorbs_bash_prefix_without_duplication() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );
    composer.set_text_content("!git status".to_string(), Vec::new(), Vec::new());

    composer.apply_external_edit("!git status".to_string());

    assert!(composer.draft.is_bash_mode);
    assert_eq!(composer.draft.textarea.text(), "git status");
    assert_eq!(composer.current_text(), "!git status");
}
#[test]
fn apply_external_edit_can_leave_bash_mode() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );
    composer.set_text_content("!git status".to_string(), Vec::new(), Vec::new());

    composer.apply_external_edit("git status".to_string());

    assert!(!composer.draft.is_bash_mode);
    assert_eq!(composer.draft.textarea.text(), "git status");
    assert_eq!(composer.current_text(), "git status");
}
#[test]
fn apply_external_edit_can_enter_bash_mode() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );
    composer.set_text_content("git status".to_string(), Vec::new(), Vec::new());

    composer.apply_external_edit("!git status".to_string());

    assert!(composer.draft.is_bash_mode);
    assert_eq!(composer.draft.textarea.text(), "git status");
    assert_eq!(composer.current_text(), "!git status");
}
#[test]
fn apply_external_edit_drops_missing_attachments() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    let placeholder = local_image_label_text(/*label_number*/ 1);
    composer.draft.textarea.insert_element(&placeholder);
    composer.attachments.local_images.push(AttachedImage {
        placeholder: placeholder.clone(),
        path: PathBuf::from("img.png"),
    });

    composer.apply_external_edit("No images here".to_string());

    assert_eq!(composer.current_text(), "No images here".to_string());
    assert!(composer.attachments.local_images.is_empty());
}
#[test]
fn apply_external_edit_renumbers_image_placeholders() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    let first_path = PathBuf::from("img1.png");
    let second_path = PathBuf::from("img2.png");
    composer.attach_image(first_path);
    composer.attach_image(second_path.clone());

    let placeholder2 = local_image_label_text(/*label_number*/ 2);
    composer.apply_external_edit(format!("Keep {placeholder2}"));

    let placeholder1 = local_image_label_text(/*label_number*/ 1);
    assert_eq!(composer.current_text(), format!("Keep {placeholder1}"));
    assert_eq!(composer.attachments.local_images.len(), 1);
    assert_eq!(
        composer.attachments.local_images[0].placeholder,
        placeholder1
    );
    assert_eq!(composer.local_image_paths(), vec![second_path]);
    assert_eq!(
        composer.draft.textarea.element_payloads(),
        vec![placeholder1]
    );
}
#[test]
fn current_text_with_pending_expands_placeholders() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    let placeholder = "[Pasted ~5 chars]".to_string();
    composer.draft.textarea.insert_element(&placeholder);
    composer
        .draft
        .pending_pastes
        .push((placeholder.clone(), "hello".to_string()));

    assert_eq!(
        composer.current_text_with_pending(),
        "hello".to_string(),
        "placeholder should expand to actual text"
    );
}
#[test]
fn current_text_with_pending_expands_overlapping_placeholders() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    let first_paste = "a".repeat(LARGE_PASTE_CHAR_THRESHOLD + 4);
    let second_paste = "b".repeat(LARGE_PASTE_CHAR_THRESHOLD + 4);
    let base = format!("[Pasted ~{} chars]", first_paste.chars().count());
    let second = format!("{base} #2");

    composer.handle_paste(first_paste.clone());
    composer.handle_paste(second_paste.clone());

    assert_eq!(composer.current_text(), format!("{base}{second}"));
    assert_eq!(
        composer.current_text_with_pending(),
        format!("{first_paste}{second_paste}")
    );
}
#[test]
fn apply_external_edit_limits_duplicates_to_occurrences() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    let placeholder = local_image_label_text(/*label_number*/ 1);
    composer.draft.textarea.insert_element(&placeholder);
    composer.attachments.local_images.push(AttachedImage {
        placeholder: placeholder.clone(),
        path: PathBuf::from("img.png"),
    });

    composer.apply_external_edit(format!("{placeholder} extra {placeholder}"));

    assert_eq!(
        composer.current_text(),
        format!("{placeholder} extra {placeholder}")
    );
    assert_eq!(composer.attachments.local_images.len(), 1);
}
#[test]
fn remote_images_do_not_modify_textarea_text_or_elements() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    composer.set_remote_image_urls(vec![
        "https://example.com/one.png".to_string(),
        "https://example.com/two.png".to_string(),
    ]);

    assert_eq!(composer.current_text(), "");
    assert_eq!(composer.text_elements(), Vec::<TextElement>::new());
}
#[test]
fn input_disabled_ignores_keypresses_and_hides_cursor() {
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

    composer.set_text_content("hello".to_string(), Vec::new(), Vec::new());
    composer.set_input_enabled(
        /*enabled*/ false,
        Some("Input disabled for test.".to_string()),
    );

    let (result, needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE));

    assert_eq!(result, InputResult::None);
    assert!(!needs_redraw);
    assert_eq!(composer.current_text(), "hello");

    let area = Rect {
        x: 0,
        y: 0,
        width: 40,
        height: 5,
    };
    assert_eq!(composer.cursor_pos(area), None);
}
#[test]
fn shutdown_in_progress_disables_input_and_uses_hint_without_footer() {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    composer.set_text_content("hello".to_string(), Vec::new(), Vec::new());
    composer.show_shutdown_in_progress();

    assert!(!composer.input_enabled());
    assert_eq!(composer.current_text(), "hello");
    assert_eq!(composer.custom_footer_height(), Some(0));

    let area = Rect {
        x: 0,
        y: 0,
        width: 40,
        height: 5,
    };
    assert_eq!(composer.cursor_pos(area), None);

    let mut terminal = Terminal::new(TestBackend::new(40, 5)).expect("terminal");
    terminal
        .draw(|f| composer.render(f.area(), f.buffer_mut()))
        .unwrap();
    insta::assert_snapshot!("shutdown_in_progress", terminal.backend());
}