#[test]
fn history_navigation_restores_remote_and_local_image_attachments() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );
    let remote_image_url = "https://example.com/remote.png".to_string();
    composer.set_remote_image_urls(vec![remote_image_url.clone()]);
    let path = PathBuf::from("/tmp/image1.png");
    composer.attach_image(path.clone());

    let (result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert!(matches!(result, InputResult::Submitted { .. }));

    let _ = composer.take_remote_image_urls();
    composer.set_text_content(String::new(), Vec::new(), Vec::new());

    let _ = composer.handle_key_event(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));

    let text = composer.current_text();
    assert_eq!(text, "[Image #2]");
    let text_elements = composer.text_elements();
    assert_eq!(text_elements.len(), 1);
    assert_eq!(text_elements[0].placeholder(&text), Some("[Image #2]"));
    assert_eq!(composer.local_image_paths(), vec![path]);
    assert_eq!(composer.remote_image_urls(), vec![remote_image_url]);
}
#[test]
fn history_navigation_restores_remote_only_submissions() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );
    let remote_image_urls = vec![
        "https://example.com/one.png".to_string(),
        "https://example.com/two.png".to_string(),
    ];
    composer.set_remote_image_urls(remote_image_urls.clone());

    let (submitted_text, submitted_elements) = composer
        .prepare_submission_text(/*record_history*/ true)
        .expect("remote-only submission should be prepared");
    assert_eq!(submitted_text, "");
    assert!(submitted_elements.is_empty());

    let _ = composer.take_remote_image_urls();
    composer.set_text_content(String::new(), Vec::new(), Vec::new());

    let _ = composer.handle_key_event(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
    assert_eq!(composer.current_text(), "");
    assert!(composer.text_elements().is_empty());
    assert_eq!(composer.remote_image_urls(), remote_image_urls);
}
#[test]
fn history_navigation_leaves_cursor_at_end_of_line() {
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

    let (_result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
    assert_eq!(composer.draft.textarea.text(), "second");
    assert_eq!(
        composer.draft.textarea.cursor(),
        composer.draft.textarea.text().len()
    );

    let (_result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
    assert_eq!(composer.draft.textarea.text(), "first");
    assert_eq!(
        composer.draft.textarea.cursor(),
        composer.draft.textarea.text().len()
    );

    let (_result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
    assert_eq!(composer.draft.textarea.text(), "second");
    assert_eq!(
        composer.draft.textarea.cursor(),
        composer.draft.textarea.text().len()
    );

    let (_result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
    assert!(composer.draft.textarea.is_empty());
    assert_eq!(
        composer.draft.textarea.cursor(),
        composer.draft.textarea.text().len()
    );
}
#[test]
fn remapped_vim_normal_history_navigation_does_not_fall_back_to_j_k() {
    use crate::key_hint;
    use crate::keymap::RuntimeKeymap;

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

    let mut keymap = RuntimeKeymap::defaults();
    keymap.vim_normal.move_up = vec![key_hint::plain(KeyCode::F(2))];
    keymap.vim_normal.move_down = vec![key_hint::plain(KeyCode::F(3))];
    composer.set_keymap_bindings(&keymap);
    composer.set_vim_enabled(/*enabled*/ true);

    let _ = composer.handle_key_event(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE));
    assert!(composer.draft.textarea.is_empty());

    let _ = composer.handle_key_event(KeyEvent::new(KeyCode::F(2), KeyModifiers::NONE));
    assert_eq!(composer.draft.textarea.text(), "first");

    let _ = composer.handle_key_event(KeyEvent::new(KeyCode::F(3), KeyModifiers::NONE));
    assert!(composer.draft.textarea.is_empty());
}
#[test]
fn remapped_editor_history_navigation_does_not_fall_back_to_up() {
    use crate::key_hint;
    use crate::keymap::RuntimeKeymap;

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

    let mut keymap = RuntimeKeymap::defaults();
    keymap.editor.move_up = vec![key_hint::plain(KeyCode::F(2))];
    composer.set_keymap_bindings(&keymap);

    let _ = composer.handle_key_event(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
    assert!(composer.draft.textarea.is_empty());

    let _ = composer.handle_key_event(KeyEvent::new(KeyCode::F(2), KeyModifiers::NONE));
    assert_eq!(composer.draft.textarea.text(), "first");
}
#[test]
fn history_navigation_from_start_of_bang_command_recalls_older_entry() {
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

    type_chars_humanlike(&mut composer, &['f', 'i', 'r', 's', 't']);
    let (result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert!(matches!(result, InputResult::Submitted { .. }));

    type_chars_humanlike(&mut composer, &['!', 'g', 'i', 't']);
    let (result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert!(matches!(result, InputResult::Submitted { .. }));

    let (_result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
    assert_eq!(composer.current_text(), "!git");

    composer.draft.textarea.set_cursor(/*pos*/ 0);
    let (_result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
    assert_eq!(composer.current_text(), "first");
}