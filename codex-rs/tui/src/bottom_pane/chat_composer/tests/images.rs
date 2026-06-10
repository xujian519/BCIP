#[test]
fn image_placeholder_snapshots() {
    snapshot_composer_state(
        "image_placeholder_single",
        /*enhanced_keys_supported*/ false,
        |composer| {
            composer.attach_image(PathBuf::from("/tmp/image1.png"));
        },
    );

    snapshot_composer_state(
        "image_placeholder_multiple",
        /*enhanced_keys_supported*/ false,
        |composer| {
            composer.attach_image(PathBuf::from("/tmp/image1.png"));
            composer.attach_image(PathBuf::from("/tmp/image2.png"));
        },
    );
}
#[test]
fn remote_image_rows_snapshots() {
    use crossterm::event::KeyCode;
    use crossterm::event::KeyEvent;
    use crossterm::event::KeyModifiers;

    snapshot_composer_state(
        "remote_image_rows",
        /*enhanced_keys_supported*/ false,
        |composer| {
            composer.set_remote_image_urls(vec![
                "https://example.com/one.png".to_string(),
                "https://example.com/two.png".to_string(),
            ]);
            composer.set_text_content("describe these".to_string(), Vec::new(), Vec::new());
        },
    );

    snapshot_composer_state(
        "remote_image_rows_selected",
        /*enhanced_keys_supported*/ false,
        |composer| {
            composer.set_remote_image_urls(vec![
                "https://example.com/one.png".to_string(),
                "https://example.com/two.png".to_string(),
            ]);
            composer.set_text_content("describe these".to_string(), Vec::new(), Vec::new());
            composer.draft.textarea.set_cursor(/*pos*/ 0);
            let _ = composer.handle_key_event(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        },
    );

    snapshot_composer_state(
        "remote_image_rows_after_delete_first",
        /*enhanced_keys_supported*/ false,
        |composer| {
            composer.set_remote_image_urls(vec![
                "https://example.com/one.png".to_string(),
                "https://example.com/two.png".to_string(),
            ]);
            composer.set_text_content("describe these".to_string(), Vec::new(), Vec::new());
            composer.draft.textarea.set_cursor(/*pos*/ 0);
            let _ = composer.handle_key_event(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
            let _ = composer.handle_key_event(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
            let _ =
                composer.handle_key_event(KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE));
        },
    );
}
#[test]
fn attach_image_and_submit_includes_local_image_paths() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );
    let path = PathBuf::from("/tmp/image1.png");
    composer.attach_image(path.clone());
    composer.handle_paste(" hi".into());
    let (result, _) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    match result {
        InputResult::Submitted {
            text,
            text_elements,
        } => {
            assert_eq!(text, "[Image #1] hi");
            assert_eq!(text_elements.len(), 1);
            assert_eq!(text_elements[0].placeholder(&text), Some("[Image #1]"));
            assert_eq!(
                text_elements[0].byte_range,
                ByteRange {
                    start: 0,
                    end: "[Image #1]".len()
                }
            );
        }
        _ => panic!("expected Submitted"),
    }
    let imgs = composer.take_recent_submission_images();
    assert_eq!(vec![path], imgs);
}
#[test]
fn attach_image_without_text_submits_empty_text_and_images() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );
    let path = PathBuf::from("/tmp/image2.png");
    composer.attach_image(path.clone());
    let (result, _) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    match result {
        InputResult::Submitted {
            text,
            text_elements,
        } => {
            assert_eq!(text, "[Image #1]");
            assert_eq!(text_elements.len(), 1);
            assert_eq!(text_elements[0].placeholder(&text), Some("[Image #1]"));
            assert_eq!(
                text_elements[0].byte_range,
                ByteRange {
                    start: 0,
                    end: "[Image #1]".len()
                }
            );
        }
        _ => panic!("expected Submitted"),
    }
    let imgs = composer.take_recent_submission_images();
    assert_eq!(imgs.len(), 1);
    assert_eq!(imgs[0], path);
    assert!(composer.attachments.local_images.is_empty());
}
#[test]
fn duplicate_image_placeholders_get_suffix() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );
    let path = PathBuf::from("/tmp/image_dup.png");
    composer.attach_image(path.clone());
    composer.handle_paste(" ".into());
    composer.attach_image(path);

    let text = composer.draft.textarea.text().to_string();
    assert!(text.contains("[Image #1]"));
    assert!(text.contains("[Image #2]"));
    assert_eq!(
        composer.attachments.local_images[0].placeholder,
        "[Image #1]"
    );
    assert_eq!(
        composer.attachments.local_images[1].placeholder,
        "[Image #2]"
    );
}
#[test]
fn image_placeholder_backspace_behaves_like_text_placeholder() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );
    let path = PathBuf::from("/tmp/image3.png");
    composer.attach_image(path.clone());
    let placeholder = composer.attachments.local_images[0].placeholder.clone();

    // Case 1: backspace at end
    composer
        .draft
        .textarea
        .move_cursor_to_end_of_line(/*move_down_at_eol*/ false);
    composer.handle_key_event(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
    assert!(!composer.draft.textarea.text().contains(&placeholder));
    assert!(composer.attachments.local_images.is_empty());

    // Re-add and ensure backspace at element start does not delete the placeholder.
    composer.attach_image(path);
    let placeholder2 = composer.attachments.local_images[0].placeholder.clone();
    // Move cursor to roughly middle of placeholder
    if let Some(start_pos) = composer.draft.textarea.text().find(&placeholder2) {
        let mid_pos = start_pos + (placeholder2.len() / 2);
        composer.draft.textarea.set_cursor(mid_pos);
        composer.handle_key_event(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
        assert!(composer.draft.textarea.text().contains(&placeholder2));
        assert_eq!(composer.attachments.local_images.len(), 1);
    } else {
        panic!("Placeholder not found in textarea");
    }
}
#[test]
fn backspace_with_multibyte_text_before_placeholder_does_not_panic() {
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

    // Insert an image placeholder at the start
    let path = PathBuf::from("/tmp/image_multibyte.png");
    composer.attach_image(path);
    // Add multibyte text after the placeholder
    composer.draft.textarea.insert_str("日本語");

    // Cursor is at end; pressing backspace should delete the last character
    // without panicking and leave the placeholder intact.
    composer.handle_key_event(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));

    assert_eq!(composer.attachments.local_images.len(), 1);
    assert!(composer.draft.textarea.text().starts_with("[Image #1]"));
}
#[test]
fn deleting_one_of_duplicate_image_placeholders_removes_one_entry() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    let path1 = PathBuf::from("/tmp/image_dup1.png");
    let path2 = PathBuf::from("/tmp/image_dup2.png");

    composer.attach_image(path1);
    // separate placeholders with a space for clarity
    composer.handle_paste(" ".into());
    composer.attach_image(path2.clone());

    let placeholder1 = composer.attachments.local_images[0].placeholder.clone();
    let placeholder2 = composer.attachments.local_images[1].placeholder.clone();
    let text = composer.draft.textarea.text().to_string();
    let start1 = text.find(&placeholder1).expect("first placeholder present");
    let end1 = start1 + placeholder1.len();
    composer.draft.textarea.set_cursor(end1);

    // Backspace should delete the first placeholder and its mapping.
    composer.handle_key_event(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));

    let new_text = composer.draft.textarea.text().to_string();
    assert_eq!(
        1,
        new_text.matches(&placeholder1).count(),
        "one placeholder remains after deletion"
    );
    assert_eq!(
        0,
        new_text.matches(&placeholder2).count(),
        "second placeholder was relabeled"
    );
    assert_eq!(
        1,
        new_text.matches("[Image #1]").count(),
        "remaining placeholder relabeled to #1"
    );
    assert_eq!(
        vec![AttachedImage {
            path: path2,
            placeholder: "[Image #1]".to_string()
        }],
        composer.attachments.local_images,
        "one image mapping remains"
    );
}
#[test]
fn deleting_reordered_image_one_renumbers_text_in_place() {
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

    let path1 = PathBuf::from("/tmp/image_first.png");
    let path2 = PathBuf::from("/tmp/image_second.png");
    let placeholder1 = local_image_label_text(/*label_number*/ 1);
    let placeholder2 = local_image_label_text(/*label_number*/ 2);

    // Placeholders can be reordered in the text buffer; deleting image #1 should renumber
    // image #2 wherever it appears, not just after the cursor.
    let text = format!("Test {placeholder2} test {placeholder1}");
    let start2 = text.find(&placeholder2).expect("placeholder2 present");
    let start1 = text.find(&placeholder1).expect("placeholder1 present");
    let text_elements = vec![
        TextElement::new(
            ByteRange {
                start: start2,
                end: start2 + placeholder2.len(),
            },
            Some(placeholder2),
        ),
        TextElement::new(
            ByteRange {
                start: start1,
                end: start1 + placeholder1.len(),
            },
            Some(placeholder1.clone()),
        ),
    ];
    composer.set_text_content(text, text_elements, vec![path1, path2.clone()]);

    let end1 = start1 + placeholder1.len();
    composer.draft.textarea.set_cursor(end1);

    composer.handle_key_event(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));

    assert_eq!(
        composer.draft.textarea.text(),
        format!("Test {placeholder1} test ")
    );
    assert_eq!(
        vec![AttachedImage {
            path: path2,
            placeholder: placeholder1
        }],
        composer.attachments.local_images,
        "attachment renumbered after deletion"
    );
}
#[test]
fn deleting_first_text_element_renumbers_following_text_element() {
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

    let path1 = PathBuf::from("/tmp/image_first.png");
    let path2 = PathBuf::from("/tmp/image_second.png");

    // Insert two adjacent atomic elements.
    composer.attach_image(path1);
    composer.attach_image(path2.clone());
    assert_eq!(composer.draft.textarea.text(), "[Image #1][Image #2]");
    assert_eq!(composer.attachments.local_images.len(), 2);

    // Delete the first element using normal textarea editing (forward Delete at cursor start).
    composer.draft.textarea.set_cursor(/*pos*/ 0);
    composer.handle_key_event(KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE));

    // Remaining image should be renumbered and the textarea element updated.
    assert_eq!(composer.attachments.local_images.len(), 1);
    assert_eq!(composer.attachments.local_images[0].path, path2);
    assert_eq!(
        composer.attachments.local_images[0].placeholder,
        "[Image #1]"
    );
    assert_eq!(composer.draft.textarea.text(), "[Image #1]");
}
#[test]
fn pasting_filepath_attaches_image() {
    let tmp = tempdir().expect("create TempDir");
    let tmp_path: PathBuf = tmp.path().join("codex_tui_test_paste_image.png");
    let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_fn(3, 2, |_x, _y| Rgba([1, 2, 3, 255]));
    img.save(&tmp_path).expect("failed to write temp png");

    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    let needs_redraw = composer.handle_paste(tmp_path.to_string_lossy().to_string());
    assert!(needs_redraw);
    assert!(composer.draft.textarea.text().starts_with("[Image #1] "));

    let imgs = composer.take_recent_submission_images();
    assert_eq!(imgs, vec![tmp_path]);
}
#[test]
fn attach_image_after_remote_prefix_uses_offset_label() {
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
    composer.attach_image(PathBuf::from("/tmp/local.png"));

    assert_eq!(
        composer.attachments.local_images[0].placeholder,
        "[Image #3]"
    );
    assert_eq!(composer.current_text(), "[Image #3]");
}
#[test]
fn prepare_submission_keeps_remote_offset_local_placeholder_numbering() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    composer.set_remote_image_urls(vec!["https://example.com/one.png".to_string()]);
    let base_text = "[Image #2] hello".to_string();
    let base_elements = vec![TextElement::new(
        (0.."[Image #2]".len()).into(),
        Some("[Image #2]".to_string()),
    )];
    composer.set_text_content(
        base_text,
        base_elements,
        vec![PathBuf::from("/tmp/local.png")],
    );

    let (submitted_text, submitted_elements) = composer
        .prepare_submission_text(/*record_history*/ true)
        .expect("remote+local submission should be generated");
    assert_eq!(submitted_text, "[Image #2] hello");
    assert_eq!(
        submitted_elements,
        vec![TextElement::new(
            (0.."[Image #2]".len()).into(),
            Some("[Image #2]".to_string())
        )]
    );
}
#[test]
fn prepare_submission_with_only_remote_images_returns_empty_text() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    composer.set_remote_image_urls(vec!["https://example.com/one.png".to_string()]);
    let (submitted_text, submitted_elements) = composer
        .prepare_submission_text(/*record_history*/ true)
        .expect("remote-only submission should be generated");
    assert_eq!(submitted_text, "");
    assert!(submitted_elements.is_empty());
}
#[test]
fn delete_selected_remote_image_relabels_local_placeholders() {
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
    composer.attach_image(PathBuf::from("/tmp/local.png"));
    composer.draft.textarea.set_cursor(/*pos*/ 0);

    let _ = composer.handle_key_event(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
    let _ = composer.handle_key_event(KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE));
    assert_eq!(
        composer.remote_image_urls(),
        vec!["https://example.com/one.png".to_string()]
    );
    assert_eq!(composer.current_text(), "[Image #2]");
    assert_eq!(
        composer.attachments.local_images[0].placeholder,
        "[Image #2]"
    );

    let _ = composer.handle_key_event(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
    let _ = composer.handle_key_event(KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE));
    assert_eq!(composer.remote_image_urls(), Vec::<String>::new());
    assert_eq!(composer.current_text(), "[Image #1]");
    assert_eq!(
        composer.attachments.local_images[0].placeholder,
        "[Image #1]"
    );
}