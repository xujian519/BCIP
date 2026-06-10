#[test]
fn empty_vim_insert_escape_enters_normal_without_esc_hint() {
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
        /*disable_paste_burst*/ false,
    );
    composer.set_vim_enabled(/*enabled*/ true);
    composer.handle_key_event(KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE));

    assert!(composer.is_empty());
    assert_eq!(
        composer.vim_mode_indicator_span(),
        Some("Vim: Insert".green())
    );

    let (result, needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));

    assert!(matches!(result, InputResult::None));
    assert!(needs_redraw);
    assert!(composer.is_empty());
    assert_eq!(
        composer.vim_mode_indicator_span(),
        Some("Vim: Normal".magenta())
    );
    assert_eq!(composer.footer.mode, FooterMode::ComposerEmpty);
    assert!(!composer.footer.esc_backtrack_hint);
}
#[test]
fn vim_mode_resets_to_normal_after_submission() {
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
        /*disable_paste_burst*/ false,
    );
    composer.set_steer_enabled(/*enabled*/ true);
    composer.set_vim_enabled(/*enabled*/ true);

    assert!(composer.draft.textarea.is_vim_enabled());
    assert_eq!(
        composer.vim_mode_indicator_span(),
        Some("Vim: Normal".magenta())
    );

    composer.handle_key_event(KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE));
    composer.set_text_content("h".to_string(), Vec::new(), Vec::new());
    let (result, _) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    assert!(composer.draft.textarea.is_vim_enabled());
    assert_eq!(
        composer.vim_mode_indicator_span(),
        Some("Vim: Normal".magenta())
    );
    assert!(composer.is_empty());
    match result {
        InputResult::Submitted { text, .. } => assert_eq!(text, "h"),
        _ => panic!("expected Submitted"),
    }
}
#[test]
fn vim_mode_resets_to_normal_after_queued_submission() {
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
        /*disable_paste_burst*/ false,
    );
    composer.set_steer_enabled(/*enabled*/ true);
    composer.set_task_running(/*running*/ true);
    composer.set_vim_enabled(/*enabled*/ true);

    composer.handle_key_event(KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE));
    composer.set_text_content("queued".to_string(), Vec::new(), Vec::new());
    let (result, _) = composer.handle_submission(/*should_queue*/ true);

    assert_eq!(
        composer.vim_mode_indicator_span(),
        Some("Vim: Normal".magenta())
    );
    assert!(composer.is_empty());
    match result {
        InputResult::Queued { text, .. } => assert_eq!(text, "queued"),
        _ => panic!("expected Queued"),
    }
}
#[test]
fn vim_mode_stays_insert_after_suppressed_submission() {
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
        /*disable_paste_burst*/ false,
    );
    composer.set_steer_enabled(/*enabled*/ true);
    composer.set_vim_enabled(/*enabled*/ true);

    composer.handle_key_event(KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE));
    composer.set_text_content("/not-a-command".to_string(), Vec::new(), Vec::new());
    let (result, _) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    assert!(matches!(result, InputResult::None));
    assert_eq!(composer.draft.textarea.text(), "/not-a-command");
    assert_eq!(
        composer.vim_mode_indicator_span(),
        Some("Vim: Insert".green())
    );
}
#[test]
fn esc_switches_vim_insert_to_normal() {
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
        /*disable_paste_burst*/ false,
    );
    composer.set_vim_enabled(/*enabled*/ true);

    composer.handle_key_event(KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE));
    composer.set_text_content("hey".to_string(), Vec::new(), Vec::new());
    composer
        .draft
        .textarea
        .set_cursor(composer.draft.textarea.text().len());
    assert_eq!(
        composer.vim_mode_indicator_span(),
        Some("Vim: Insert".green())
    );
    assert_eq!(composer.draft.textarea.cursor(), "hey".len());

    composer.handle_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    assert_eq!(
        composer.vim_mode_indicator_span(),
        Some("Vim: Normal".magenta())
    );
    assert_eq!(composer.draft.textarea.cursor(), "he".len());
}
#[test]
fn vim_insert_uses_bar_cursor_style() {
    use crate::render::renderable::Renderable;
    use crossterm::cursor::SetCursorStyle;
    use crossterm::event::KeyCode;
    use crossterm::event::KeyEvent;
    use crossterm::event::KeyModifiers;
    use crossterm::queue;

    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ true,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );
    let area = Rect::new(0, 0, 80, 10);
    let style_output = |style| {
        let mut output = Vec::new();
        queue!(output, style).expect("queue cursor style");
        output
    };
    let default = style_output(SetCursorStyle::DefaultUserShape);
    let steady_bar = style_output(SetCursorStyle::SteadyBar);

    assert_eq!(style_output(composer.cursor_style(area)), default,);

    composer.set_vim_enabled(/*enabled*/ true);
    assert_eq!(style_output(composer.cursor_style(area)), default,);

    composer.handle_key_event(KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE));
    composer.set_text_content("hey".to_string(), Vec::new(), Vec::new());
    assert_eq!(style_output(composer.cursor_style(area)), steady_bar);

    composer.handle_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    assert_eq!(style_output(composer.cursor_style(area)), default,);
}
#[test]
fn vim_normal_j_k_navigate_history_at_history_boundaries() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    type_chars_humanlike(&mut composer, &['f', 'i', 'r', 's', 't']);
    let (result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert!(matches!(result, InputResult::Submitted { .. }));

    type_chars_humanlike(&mut composer, &['s', 'e', 'c', 'o', 'n', 'd']);
    let (result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert!(matches!(result, InputResult::Submitted { .. }));

    composer.set_vim_enabled(/*enabled*/ true);

    let (_result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE));
    assert_eq!(composer.draft.textarea.text(), "second");
    assert_eq!(composer.draft.textarea.cursor(), "second".len() - 1);

    let (_result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE));
    assert_eq!(composer.draft.textarea.text(), "first");
    assert_eq!(composer.draft.textarea.cursor(), "first".len() - 1);

    let (_result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE));
    assert_eq!(composer.draft.textarea.text(), "second");
    assert_eq!(composer.draft.textarea.cursor(), "second".len() - 1);

    let (_result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE));
    assert!(composer.draft.textarea.is_empty());
    assert_eq!(
        composer.draft.textarea.cursor(),
        composer.draft.textarea.text().len()
    );
}
#[test]
fn vim_normal_j_k_fall_back_to_multiline_cursor_movement() {
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
        .set_text_clearing_elements("one\ntwo");
    composer.draft.textarea.set_cursor(/*pos*/ 0);
    composer.set_vim_enabled(/*enabled*/ true);

    let _ = composer.handle_key_event(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE));
    assert_eq!(composer.draft.textarea.cursor(), "one\n".len());

    let _ = composer.handle_key_event(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE));
    assert_eq!(composer.draft.textarea.cursor(), 0);
}
#[test]
fn vim_normal_operator_motion_does_not_navigate_history() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    type_chars_humanlike(&mut composer, &['f', 'i', 'r', 's', 't']);
    let (result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert!(matches!(result, InputResult::Submitted { .. }));

    type_chars_humanlike(&mut composer, &['s', 'e', 'c', 'o', 'n', 'd']);
    let (result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert!(matches!(result, InputResult::Submitted { .. }));

    composer.set_vim_enabled(/*enabled*/ true);

    let _ = composer.handle_key_event(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE));
    assert_eq!(composer.draft.textarea.text(), "second");

    let _ = composer.handle_key_event(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE));
    let _ = composer.handle_key_event(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE));
    assert!(composer.draft.textarea.is_empty());
    assert_eq!(composer.current_text(), "");
}
#[test]
fn vim_normal_operator_pending_consumes_submit_key() {
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
    composer.set_vim_enabled(/*enabled*/ true);

    let _ = composer.handle_key_event(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE));
    assert!(composer.draft.textarea.is_vim_operator_pending());

    let (result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    assert!(matches!(result, InputResult::None));
    assert_eq!(composer.draft.textarea.text(), "hello");
    assert_eq!(
        composer.vim_mode_indicator_span(),
        Some("Vim: Normal".magenta())
    );
    assert!(!composer.draft.textarea.is_vim_operator_pending());
}
#[test]
fn vim_normal_history_navigation_from_start_of_bang_command_recalls_older_entry() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    type_chars_humanlike(&mut composer, &['f', 'i', 'r', 's', 't']);
    let (result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert!(matches!(result, InputResult::Submitted { .. }));

    type_chars_humanlike(&mut composer, &['!', 'g', 'i', 't']);
    let (result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert!(matches!(result, InputResult::Submitted { .. }));

    composer.set_vim_enabled(/*enabled*/ true);

    let (_result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE));
    assert_eq!(composer.current_text(), "!git");
    assert_eq!(composer.draft.textarea.cursor(), "git".len() - 1);

    let (_result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE));
    assert_eq!(composer.current_text(), "first");
    assert_eq!(composer.draft.textarea.cursor(), "first".len() - 1);
}