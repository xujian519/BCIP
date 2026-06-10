#[test]
fn enter_submits_when_file_popup_has_no_selection() {
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

    let input = "npx -y @kaeawc/auto-mobile@latest";
    composer.draft.textarea.insert_str(input);
    composer.draft.textarea.set_cursor(input.len());
    composer.sync_popups();

    assert!(matches!(composer.popups.active, ActivePopup::File(_)));

    let (result, consumed) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert!(consumed);
    match result {
        InputResult::Submitted { text, .. } => assert_eq!(text, input),
        _ => panic!("expected Submitted"),
    }
}
#[test]
fn empty_enter_returns_none() {
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

    // Ensure composer is empty and press Enter.
    assert!(composer.draft.textarea.text().is_empty());
    let (result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    match result {
        InputResult::None => {}
        other => panic!("expected None for empty enter, got: {other:?}"),
    }
}
#[test]
fn submit_at_character_limit_succeeds() {
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
    let input = "x".repeat(MAX_USER_INPUT_TEXT_CHARS);
    composer.draft.textarea.set_text_clearing_elements(&input);

    let (result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    assert!(matches!(
        result,
        InputResult::Submitted { text, .. } if text == input
    ));
}
#[test]
fn oversized_submit_reports_error_and_restores_draft() {
    use crossterm::event::KeyCode;
    use crossterm::event::KeyEvent;
    use crossterm::event::KeyModifiers;

    let (tx, mut rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );
    composer.set_steer_enabled(true);
    let input = "x".repeat(MAX_USER_INPUT_TEXT_CHARS + 1);
    composer.draft.textarea.set_text_clearing_elements(&input);

    let (result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    assert_eq!(InputResult::None, result);
    assert_eq!(composer.draft.textarea.text(), input);

    let mut found_error = false;
    while let Ok(event) = rx.try_recv() {
        if let AppEvent::InsertHistoryCell(cell) = event {
            let message = cell
                .display_lines(/*width*/ 80)
                .into_iter()
                .map(|line| line.to_string())
                .collect::<Vec<_>>()
                .join("\n");
            assert!(message.contains(&user_input_too_large_message(input.chars().count())));
            found_error = true;
            break;
        }
    }
    assert!(found_error, "expected oversized-input error history cell");
}
#[test]
fn oversized_queued_submission_reports_error_and_restores_draft() {
    use crossterm::event::KeyCode;
    use crossterm::event::KeyEvent;
    use crossterm::event::KeyModifiers;

    let (tx, mut rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );
    composer.set_steer_enabled(false);
    let input = "x".repeat(MAX_USER_INPUT_TEXT_CHARS + 1);
    composer.draft.textarea.set_text_clearing_elements(&input);

    let (result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    assert_eq!(InputResult::None, result);
    assert_eq!(composer.draft.textarea.text(), input);

    let mut found_error = false;
    while let Ok(event) = rx.try_recv() {
        if let AppEvent::InsertHistoryCell(cell) = event {
            let message = cell
                .display_lines(/*width*/ 80)
                .into_iter()
                .map(|line| line.to_string())
                .collect::<Vec<_>>()
                .join("\n");
            assert!(message.contains(&user_input_too_large_message(input.chars().count())));
            found_error = true;
            break;
        }
    }
    assert!(found_error, "expected oversized-input error history cell");
}
#[test]
fn edit_clears_pending_paste() {
    use crossterm::event::KeyCode;
    use crossterm::event::KeyEvent;
    use crossterm::event::KeyModifiers;

    let large = "y".repeat(LARGE_PASTE_CHAR_THRESHOLD + 1);
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    composer.handle_paste(large);
    assert_eq!(composer.draft.pending_pastes.len(), 1);

    // Any edit that removes the placeholder should clear pending_paste
    composer.handle_key_event(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
    assert!(composer.draft.pending_pastes.is_empty());
}
#[test]
fn kill_buffer_persists_after_submit() {
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
    composer.draft.textarea.insert_str("restore me");
    composer.draft.textarea.set_cursor(/*pos*/ 0);

    let (_result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::CONTROL));
    assert!(composer.draft.textarea.is_empty());

    composer.draft.textarea.insert_str("hello");
    let (result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert!(matches!(result, InputResult::Submitted { .. }));
    assert!(composer.draft.textarea.is_empty());

    let (_result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Char('y'), KeyModifiers::CONTROL));
    assert_eq!(composer.draft.textarea.text(), "restore me");
}
#[test]
fn kill_buffer_persists_after_slash_command_dispatch() {
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
    composer.draft.textarea.insert_str("restore me");
    composer.draft.textarea.set_cursor(/*pos*/ 0);

    let (_result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::CONTROL));
    assert!(composer.draft.textarea.is_empty());

    composer.draft.textarea.insert_str("/diff");
    let (result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    match result {
        InputResult::Command(cmd) => {
            assert_eq!(cmd.command(), "diff");
        }
        _ => panic!("expected Command result for '/diff'"),
    }
    assert!(composer.draft.textarea.is_empty());

    let (_result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Char('y'), KeyModifiers::CONTROL));
    assert_eq!(composer.draft.textarea.text(), "restore me");
}
#[test]
fn enter_queues_when_queue_submissions_is_enabled() {
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
    composer.set_queue_submissions(/*queue_submissions*/ true);
    composer
        .draft
        .textarea
        .set_text_clearing_elements("queued before session");

    let (result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    assert_eq!(
        result,
        InputResult::Queued {
            text: "queued before session".to_string(),
            text_elements: Vec::new(),
            action: QueuedInputAction::Plain,
        }
    );
}
#[test]
fn tab_queues_slash_led_prompts_while_task_running_without_validation() {
    use crossterm::event::KeyCode;
    use crossterm::event::KeyEvent;
    use crossterm::event::KeyModifiers;

    fn assert_queued_slash(input: &str) {
        let (tx, mut rx) = unbounded_channel::<AppEvent>();
        let sender = AppEventSender::new(tx);
        let mut composer = ChatComposer::new(
            /*has_input_focus*/ true,
            sender,
            /*enhanced_keys_supported*/ false,
            "向云熙专利智能体提出任何需求".to_string(),
            /*disable_paste_burst*/ false,
        );
        composer.set_task_running(/*running*/ true);
        composer.draft.textarea.set_text_clearing_elements(input);

        let (result, _needs_redraw) =
            composer.handle_key_event(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));

        match result {
            InputResult::Queued {
                text,
                text_elements,
                action,
            } => {
                assert_eq!(text, input);
                assert!(text_elements.is_empty());
                assert_eq!(action, QueuedInputAction::ParseSlash);
            }
            other => panic!("expected slash-led input to queue, got {other:?}"),
        }
        assert!(composer.draft.textarea.is_empty());
        assert!(
            rx.try_recv().is_err(),
            "queueing should not report slash errors"
        );
    }

    assert_queued_slash("/compact");
    assert_queued_slash("/review check regressions");
    assert_queued_slash("/fast");
    assert_queued_slash("/does-not-exist");
}
#[test]
fn remapped_submit_does_not_fall_back_to_enter() {
    use crate::key_hint;
    use crate::keymap::RuntimeKeymap;
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
        .textarea
        .set_text_clearing_elements("explain the change");
    composer
        .draft
        .textarea
        .set_cursor(composer.draft.textarea.text().len());
    let mut keymap = RuntimeKeymap::defaults();
    keymap.composer.submit = vec![key_hint::ctrl(KeyCode::Char('j'))];
    composer.set_keymap_bindings(&keymap);

    let (result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    assert_eq!(InputResult::None, result);
    assert_eq!("explain the change\n", composer.draft.textarea.text());
}
#[test]
fn remapped_queue_does_not_fall_back_to_tab() {
    use crate::key_hint;
    use crate::keymap::RuntimeKeymap;
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
    composer.set_task_running(/*running*/ true);
    composer
        .draft
        .textarea
        .set_text_clearing_elements("queue me");
    let mut keymap = RuntimeKeymap::defaults();
    keymap.composer.queue = vec![key_hint::ctrl(KeyCode::Char('q'))];
    composer.set_keymap_bindings(&keymap);

    let (result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));

    assert_eq!(InputResult::None, result);
    assert_eq!("queue me", composer.draft.textarea.text());
}
#[test]
fn remapped_history_search_does_not_fall_back_to_ctrl_r() {
    use crate::key_hint;
    use crate::keymap::RuntimeKeymap;
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
    let mut keymap = RuntimeKeymap::defaults();
    keymap.composer.history_search_previous = vec![key_hint::plain(KeyCode::F(2))];
    composer.set_keymap_bindings(&keymap);

    let _ = composer.handle_key_event(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL));
    assert!(!composer.history_search_active());

    let _ = composer.handle_key_event(KeyEvent::new(KeyCode::F(2), KeyModifiers::NONE));
    assert!(composer.history_search_active());
}
#[test]
fn tab_queues_leading_space_slash_as_plain_text_while_task_running() {
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
    composer.set_task_running(/*running*/ true);
    composer
        .draft
        .textarea
        .set_text_clearing_elements(" /does-not-exist");

    let (result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));

    match result {
        InputResult::Queued { text, action, .. } => {
            assert_eq!(text, "/does-not-exist");
            assert_eq!(action, QueuedInputAction::Plain);
        }
        other => panic!("expected leading-space slash input to queue, got {other:?}"),
    }
}
#[test]
fn tab_queues_bang_shell_prompts_while_task_running_without_execution() {
    use crossterm::event::KeyCode;
    use crossterm::event::KeyEvent;
    use crossterm::event::KeyModifiers;

    fn assert_queued_shell(input: &str, expected_text: &str) {
        let (tx, mut rx) = unbounded_channel::<AppEvent>();
        let sender = AppEventSender::new(tx);
        let mut composer = ChatComposer::new(
            /*has_input_focus*/ true,
            sender,
            /*enhanced_keys_supported*/ false,
            "向云熙专利智能体提出任何需求".to_string(),
            /*disable_paste_burst*/ false,
        );
        composer.set_task_running(/*running*/ true);
        composer.draft.textarea.set_text_clearing_elements(input);

        let (result, _needs_redraw) =
            composer.handle_key_event(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));

        match result {
            InputResult::Queued {
                text,
                text_elements,
                action,
            } => {
                assert_eq!(text, expected_text);
                assert!(text_elements.is_empty());
                assert_eq!(action, QueuedInputAction::RunShell);
            }
            other => panic!("expected bang shell input to queue, got {other:?}"),
        }
        assert!(composer.draft.textarea.is_empty());
        assert!(
            rx.try_recv().is_err(),
            "queueing should not show shell help immediately"
        );
    }

    assert_queued_shell("!echo hi", "!echo hi");
    assert_queued_shell("!", "!");
    assert_queued_shell(" !echo hi", "!echo hi");
}
#[test]
fn tab_submits_when_no_task_running() {
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

    type_chars_humanlike(&mut composer, &['h', 'i']);

    let (result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));

    assert!(matches!(
        result,
        InputResult::Submitted { ref text, .. } if text == "hi"
    ));
    assert!(composer.draft.textarea.is_empty());
}
#[test]
fn tab_does_not_submit_for_bang_shell_command() {
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
    composer.set_task_running(/*running*/ false);

    type_chars_humanlike(&mut composer, &['!', 'l', 's']);

    let (result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));

    assert!(matches!(result, InputResult::None));
    assert!(
        composer.current_text().starts_with("!ls"),
        "expected Tab not to submit or clear a `!` command"
    );
}
#[test]
fn bang_prefixed_slash_text_submits_literal_shell_command() {
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

    type_chars_humanlike(&mut composer, &['!', '/', 'd', 'i', 'f', 'f']);

    let (result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    assert!(matches!(
        result,
        InputResult::Submitted { ref text, .. } if text == "!/diff"
    ));
}
#[test]
fn submit_captures_recent_mention_bindings_before_clearing_textarea() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    let mention_bindings = vec![MentionBinding {
        mention: "figma".to_string(),
        path: "/tmp/user/figma/SKILL.md".to_string(),
    }];
    composer.set_text_content_with_mention_bindings(
        "$figma please".to_string(),
        Vec::new(),
        Vec::new(),
        mention_bindings.clone(),
    );

    let (result, _) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert!(matches!(result, InputResult::Submitted { .. }));
    assert_eq!(
        composer.take_recent_submission_mention_bindings(),
        mention_bindings
    );
    assert!(composer.take_mention_bindings().is_empty());
}