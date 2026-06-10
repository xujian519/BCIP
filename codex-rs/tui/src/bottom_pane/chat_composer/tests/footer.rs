#[test]
fn footer_hint_row_is_separated_from_composer() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    let area = Rect::new(0, 0, 40, 6);
    let mut buf = Buffer::empty(area);
    composer.render(area, &mut buf);

    let row_to_string = |y: u16| {
        let mut row = String::new();
        for x in 0..area.width {
            row.push(buf[(x, y)].symbol().chars().next().unwrap_or(' '));
        }
        row
    };

    let mut hint_row: Option<(u16, String)> = None;
    for y in 0..area.height {
        let row = row_to_string(y);
        if row.contains("? for shortcuts") {
            hint_row = Some((y, row));
            break;
        }
    }

    let (hint_row_idx, hint_row_contents) =
        hint_row.expect("expected footer hint row to be rendered");
    assert_eq!(
        hint_row_idx,
        area.height - 1,
        "hint row should occupy the bottom line: {hint_row_contents:?}",
    );

    assert!(
        hint_row_idx > 0,
        "expected a spacing row above the footer hints",
    );

    let spacing_row = row_to_string(hint_row_idx - 1);
    assert_eq!(
        spacing_row.trim(),
        "",
        "expected blank spacing row above hints but saw: {spacing_row:?}",
    );
}
#[test]
fn footer_flash_overrides_footer_hint_override() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );
    composer.set_footer_hint_override(Some(vec![("K".to_string(), "label".to_string())]));
    composer.show_footer_flash(Line::from("FLASH"), Duration::from_secs(10));

    let area = Rect::new(0, 0, 60, 6);
    let mut buf = Buffer::empty(area);
    composer.render(area, &mut buf);

    let mut bottom_row = String::new();
    for x in 0..area.width {
        bottom_row.push(
            buf[(x, area.height - 1)]
                .symbol()
                .chars()
                .next()
                .unwrap_or(' '),
        );
    }
    assert!(
        bottom_row.contains("FLASH"),
        "expected flash content to render in footer row, saw: {bottom_row:?}",
    );
    assert!(
        !bottom_row.contains("K label"),
        "expected flash to override hint override, saw: {bottom_row:?}",
    );
}
#[cfg(not(target_os = "linux"))]
#[test]
fn remove_recording_meter_placeholder_clears_placeholder_text() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    let id = composer.insert_recording_meter_placeholder("⠤⠤⠤⠤");
    composer.remove_recording_meter_placeholder(&id);

    assert_eq!(composer.draft.textarea.text(), "");
    assert!(composer.draft.textarea.named_element_range(&id).is_none());
}
#[test]
fn footer_flash_expires_and_falls_back_to_hint_override() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );
    composer.set_footer_hint_override(Some(vec![("K".to_string(), "label".to_string())]));
    composer.show_footer_flash(Line::from("FLASH"), Duration::from_secs(10));
    composer.footer.flash.as_mut().unwrap().expires_at =
        Instant::now() - Duration::from_secs(1);

    let area = Rect::new(0, 0, 60, 6);
    let mut buf = Buffer::empty(area);
    composer.render(area, &mut buf);

    let mut bottom_row = String::new();
    for x in 0..area.width {
        bottom_row.push(
            buf[(x, area.height - 1)]
                .symbol()
                .chars()
                .next()
                .unwrap_or(' '),
        );
    }
    assert!(
        bottom_row.contains("K label"),
        "expected hint override to render after flash expired, saw: {bottom_row:?}",
    );
    assert!(
        !bottom_row.contains("FLASH"),
        "expected expired flash to be hidden, saw: {bottom_row:?}",
    );
}
fn snapshot_composer_state_with_width<F>(
    name: &str,
    width: u16,
    enhanced_keys_supported: bool,
    setup: F,
) where
    F: FnOnce(&mut ChatComposer),
{
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        enhanced_keys_supported,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );
    setup(&mut composer);
    let footer_props = composer.footer_props();
    let footer_lines = footer_height(&footer_props);
    let footer_spacing = ChatComposer::footer_spacing(footer_lines);
    let height = footer_lines + footer_spacing + 8;
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal
        .draw(|f| composer.render(f.area(), f.buffer_mut()))
        .unwrap();
    insta::assert_snapshot!(name, terminal.backend());
}
fn snapshot_composer_state<F>(name: &str, enhanced_keys_supported: bool, setup: F)
where
    F: FnOnce(&mut ChatComposer),
{
    snapshot_composer_state_with_width(
        name,
        /*width*/ 100,
        enhanced_keys_supported,
        setup,
    );
}
#[test]
fn shell_command_cursor_uses_absorbed_prefix() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ true,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );
    let area = Rect::new(0, 0, 40, 5);

    composer.set_text_content("!git".to_string(), Vec::new(), Vec::new());
    composer.move_cursor_to_end();
    assert_eq!(composer.cursor_pos(area), Some((5, 1)));

    composer.set_text_content("! git".to_string(), Vec::new(), Vec::new());
    composer.move_cursor_to_end();
    assert_eq!(composer.cursor_pos(area), Some((6, 1)));
}
#[test]
fn shell_command_uses_shell_accent_style() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ true,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );
    composer.set_status_line_enabled(/*enabled*/ true);
    composer.set_status_line(Some(Line::from(
        "gpt-5.4 high fast · ~/code/codex-1 · Context 0% used",
    )));
    composer.set_text_content("!git status".to_string(), Vec::new(), Vec::new());

    let area = Rect::new(0, 0, 100, 9);
    let mut buf = Buffer::empty(area);
    composer.render(area, &mut buf);

    let prompt_cell = &buf[(0, 1)];
    assert_eq!(prompt_cell.symbol(), "!");
    assert_eq!(prompt_cell.style().fg, Some(Color::LightRed));

    let footer_y = area.height - 1;
    let footer_text = (0..area.width)
        .map(|x| buf[(x, footer_y)].symbol().chars().next().unwrap_or(' '))
        .collect::<String>();
    let shell_label_x = footer_text
        .find("Shell mode")
        .expect("expected shell mode footer label");
    assert_eq!(
        buf[(shell_label_x as u16, footer_y)].style().fg,
        Some(Color::LightRed)
    );
}
#[test]
fn status_line_hyperlink_marks_pr_number_cells() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ true,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );
    let url = "https://github.com/openai/codex/pull/20252";
    composer.set_status_line_enabled(/*enabled*/ true);
    composer.set_status_line(Some(Line::from(Span::styled(
        "PR #20252",
        Style::default().cyan().underlined(),
    ))));
    composer.set_status_line_hyperlink(Some(url.to_string()));

    let area = Rect::new(0, 0, 40, 6);
    let mut buf = Buffer::empty(area);
    composer.render(area, &mut buf);

    let marked_cells = (area.top()..area.bottom())
        .flat_map(|y| (area.left()..area.right()).map(move |x| (x, y)))
        .filter(|&(x, y)| buf[(x, y)].symbol().contains(url))
        .count();
    assert_eq!(
        marked_cells,
        "PR #20252".chars().filter(|ch| !ch.is_whitespace()).count()
    );
}
#[test]
fn esc_exits_empty_shell_mode() {
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

    type_chars_humanlike(&mut composer, &['!']);
    assert!(composer.draft.is_bash_mode);
    assert_eq!(composer.current_text(), "!");

    let (result, needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));

    assert!(matches!(result, InputResult::None));
    assert!(needs_redraw);
    assert!(!composer.draft.is_bash_mode);
    assert_eq!(composer.current_text(), "");
}
#[test]
fn esc_keeps_shell_mode_when_paste_burst_flushes_pending_text() {
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

    type_chars_humanlike(&mut composer, &['!']);
    let _ = composer.handle_key_event(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE));
    assert!(composer.is_in_paste_burst());
    assert_eq!(composer.current_text(), "!");

    let (result, needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));

    assert!(matches!(result, InputResult::None));
    assert!(needs_redraw);
    assert!(composer.draft.is_bash_mode);
    assert_eq!(composer.current_text(), "!g");
}
#[test]
fn esc_hint_stays_hidden_with_draft_content() {
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

    type_chars_humanlike(&mut composer, &['d']);

    assert!(!composer.is_empty());
    assert_eq!(composer.current_text(), "d");
    assert_eq!(composer.footer.mode, FooterMode::ComposerEmpty);
    assert!(matches!(composer.popups.active, ActivePopup::None));

    let _ = composer.handle_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));

    assert_eq!(composer.footer.mode, FooterMode::ComposerEmpty);
    assert!(!composer.footer.esc_backtrack_hint);
}
#[test]
fn base_footer_mode_tracks_empty_state_after_quit_hint_expires() {
    use crossterm::event::KeyCode;

    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    type_chars_humanlike(&mut composer, &['d']);
    composer
        .show_quit_shortcut_hint(key_hint::ctrl(KeyCode::Char('c')), /*has_focus*/ true);
    composer.footer.quit_shortcut_expires_at =
        Some(Instant::now() - std::time::Duration::from_secs(1));

    assert_eq!(composer.footer_mode(), FooterMode::ComposerHasDraft);

    composer.set_text_content(String::new(), Vec::new(), Vec::new());
    assert_eq!(composer.footer_mode(), FooterMode::ComposerEmpty);
}