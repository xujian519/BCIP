#[test]
fn slash_command_can_be_typed_and_dispatched_after_vim_normal_slash() {
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

    for ch in ['/', 'd', 'i', 'f', 'f'] {
        let _ = composer.handle_key_event(KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE));
    }
    assert_eq!(composer.draft.textarea.text(), "/diff");
    assert!(matches!(composer.popups.active, ActivePopup::Command(_)));

    let (result, needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    assert!(needs_redraw);
    assert!(composer.is_empty());
    assert_eq!(
        composer.vim_mode_indicator_span(),
        Some("Vim: Normal".magenta())
    );
    assert!(matches!(result, InputResult::Command(SlashCommand::Diff)));
}
#[test]
fn slash_popup_model_first_for_mo_ui() {
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

    // Type "/mo" humanlike so paste-burst doesn’t interfere.
    type_chars_humanlike(&mut composer, &['/', 'm', 'o']);

    let mut terminal = match Terminal::new(TestBackend::new(60, 5)) {
        Ok(t) => t,
        Err(e) => panic!("Failed to create terminal: {e}"),
    };
    terminal
        .draw(|f| composer.render(f.area(), f.buffer_mut()))
        .unwrap_or_else(|e| panic!("Failed to draw composer: {e}"));

    // Visual snapshot should show the slash popup with /model as the first entry.
    insta::assert_snapshot!("slash_popup_mo", terminal.backend());
}
#[test]
fn slash_popup_model_first_for_mo_logic() {
    use super::super::command_popup::CommandItem;
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );
    type_chars_humanlike(&mut composer, &['/', 'm', 'o']);

    match &composer.popups.active {
        ActivePopup::Command(popup) => match popup.selected_item() {
            Some(CommandItem::Builtin(cmd)) => {
                assert_eq!(cmd.command(), "model")
            }
            Some(CommandItem::ServiceTier(command)) => {
                panic!("expected model command, got service tier {command:?}")
            }
            None => panic!("no selected command for '/mo'"),
        },
        _ => panic!("slash popup not active after typing '/mo'"),
    }
}
#[test]
fn slash_popup_resume_for_res_ui() {
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

    // Type "/res" humanlike so paste-burst doesn’t interfere.
    type_chars_humanlike(&mut composer, &['/', 'r', 'e', 's']);

    let mut terminal = Terminal::new(TestBackend::new(60, 6)).expect("terminal");
    terminal
        .draw(|f| composer.render(f.area(), f.buffer_mut()))
        .expect("draw composer");

    // Snapshot should show /resume as the first entry for /res.
    insta::assert_snapshot!("slash_popup_res", terminal.backend());
}
#[test]
fn slash_popup_resume_for_res_logic() {
    use super::super::command_popup::CommandItem;
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );
    type_chars_humanlike(&mut composer, &['/', 'r', 'e', 's']);

    match &composer.popups.active {
        ActivePopup::Command(popup) => match popup.selected_item() {
            Some(CommandItem::Builtin(cmd)) => {
                assert_eq!(cmd.command(), "resume")
            }
            Some(CommandItem::ServiceTier(command)) => {
                panic!("expected resume command, got service tier {command:?}")
            }
            None => panic!("no selected command for '/res'"),
        },
        _ => panic!("slash popup not active after typing '/res'"),
    }
}
#[test]
fn slash_popup_pets_for_pet_ui() {
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

    type_chars_humanlike(&mut composer, &['/', 'p', 'e', 't']);

    let mut terminal = Terminal::new(TestBackend::new(60, 5)).expect("terminal");
    terminal
        .draw(|f| composer.render(f.area(), f.buffer_mut()))
        .expect("draw composer");

    insta::assert_snapshot!("slash_popup_pet", terminal.backend());
}
#[test]
fn slash_popup_pets_for_pet_logic() {
    use super::super::command_popup::CommandItem;
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );
    type_chars_humanlike(&mut composer, &['/', 'p', 'e', 't']);

    match &composer.popups.active {
        ActivePopup::Command(popup) => match popup.selected_item() {
            Some(CommandItem::Builtin(cmd)) => {
                assert_eq!(cmd.command(), "pets")
            }
            Some(CommandItem::ServiceTier(command)) => {
                panic!("expected pets command, got service tier {command:?}")
            }
            None => panic!("no selected command for '/pet'"),
        },
        _ => panic!("slash popup not active after typing '/pet'"),
    }
}
#[test]
fn slash_popup_btw_for_bt_ui() {
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

    type_chars_humanlike(&mut composer, &['/', 'b', 't']);

    let mut terminal = Terminal::new(TestBackend::new(60, 5)).expect("terminal");
    terminal
        .draw(|f| composer.render(f.area(), f.buffer_mut()))
        .expect("draw composer");

    insta::assert_snapshot!("slash_popup_bt", terminal.backend());
}
#[test]
fn slash_popup_btw_for_bt_logic() {
    use super::super::command_popup::CommandItem;
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );
    type_chars_humanlike(&mut composer, &['/', 'b', 't']);

    match &composer.popups.active {
        ActivePopup::Command(popup) => match popup.selected_item() {
            Some(CommandItem::Builtin(cmd)) => {
                assert_eq!(cmd.command(), "btw")
            }
            Some(CommandItem::ServiceTier(command)) => {
                panic!("expected btw command, got service tier {command:?}")
            }
            None => panic!("no selected command for '/bt'"),
        },
        _ => panic!("slash popup not active after typing '/bt'"),
    }
}
#[test]
fn slash_popup_side_for_si_ui() {
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

    type_chars_humanlike(&mut composer, &['/', 's', 'i']);

    let mut terminal = Terminal::new(TestBackend::new(60, 5)).expect("terminal");
    terminal
        .draw(|f| composer.render(f.area(), f.buffer_mut()))
        .expect("draw composer");

    insta::assert_snapshot!("slash_popup_si", terminal.backend());
}
#[test]
fn slash_popup_side_for_si_logic() {
    use super::super::command_popup::CommandItem;
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );
    type_chars_humanlike(&mut composer, &['/', 's', 'i']);

    match &composer.popups.active {
        ActivePopup::Command(popup) => match popup.selected_item() {
            Some(CommandItem::Builtin(cmd)) => {
                assert_eq!(cmd.command(), "side")
            }
            Some(CommandItem::ServiceTier(command)) => {
                panic!("expected side command, got service tier {command:?}")
            }
            None => panic!("no selected command for '/si'"),
        },
        _ => panic!("slash popup not active after typing '/si'"),
    }
}
#[test]
fn service_tier_slash_command_dispatches_from_catalog_name() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );
    composer.set_service_tier_commands_enabled(/*enabled*/ true);
    composer.set_service_tier_commands(vec![ServiceTierCommand {
        id: "priority".to_string(),
        name: "fast".to_string(),
        description: "Fastest inference with increased plan usage".to_string(),
    }]);
    type_chars_humanlike(&mut composer, &['/', 'f', 'a', 's', 't']);

    let (result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    assert_eq!(
        result,
        InputResult::ServiceTierCommand(ServiceTierCommand {
            id: "priority".to_string(),
            name: "fast".to_string(),
            description: "Fastest inference with increased plan usage".to_string(),
        })
    );
}
#[test]
fn slash_init_dispatches_command_and_does_not_submit_literal_text() {
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

    // Type the slash command.
    type_chars_humanlike(&mut composer, &['/', 'i', 'n', 'i', 't']);

    // Press Enter to dispatch the selected command.
    let (result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    // When a slash command is dispatched, the composer should return a
    // Command result (not submit literal text) and clear its textarea.
    match result {
        InputResult::Command(cmd) => {
            assert_eq!(cmd.command(), "init");
        }
        InputResult::CommandWithArgs(_, _, _) => {
            panic!("expected command dispatch without args for '/init'")
        }
        InputResult::ServiceTierCommand(command) => {
            panic!("expected init command, got service tier {command:?}")
        }
        InputResult::Submitted { text, .. } => {
            panic!("expected command dispatch, but composer submitted literal text: {text}")
        }
        InputResult::Queued { .. } => {
            panic!("expected command dispatch, but composer queued literal text")
        }
        InputResult::None => panic!("expected Command result for '/init'"),
    }
    assert!(
        composer.draft.textarea.is_empty(),
        "composer should be cleared"
    );
}
#[test]
fn slash_command_disabled_while_task_running_keeps_text() {
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
    composer.set_task_running(/*running*/ true);
    composer
        .draft
        .textarea
        .set_text_clearing_elements("/review these changes");

    let (result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    assert_eq!(InputResult::None, result);
    assert_eq!("/review these changes", composer.draft.textarea.text());

    let mut found_error = false;
    while let Ok(event) = rx.try_recv() {
        if let AppEvent::InsertHistoryCell(cell) = event {
            let message = cell
                .display_lines(/*width*/ 80)
                .into_iter()
                .map(|line| line.to_string())
                .collect::<Vec<_>>()
                .join("\n");
            assert!(message.contains("disabled while a task is in progress"));
            found_error = true;
            break;
        }
    }
    assert!(found_error, "expected error history cell to be sent");
}
#[test]
fn slash_tab_completion_moves_cursor_to_end() {
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

    type_chars_humanlike(&mut composer, &['/', 'c']);

    let (_result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));

    assert_eq!(composer.draft.textarea.text(), "/compact ");
    assert_eq!(
        composer.draft.textarea.cursor(),
        composer.draft.textarea.text().len()
    );
}
#[test]
fn slash_tab_completion_wins_over_queueing_while_task_running() {
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

    type_chars_humanlike(&mut composer, &['/', 'm', 'o']);

    let (result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));

    assert_eq!(result, InputResult::None);
    assert_eq!(composer.draft.textarea.text(), "/model ");
    assert_eq!(
        composer.draft.textarea.cursor(),
        composer.draft.textarea.text().len()
    );
}
#[test]
fn slash_key_completes_selected_slash_command_as_text() {
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

    type_chars_humanlike(&mut composer, &['/', 'm']);

    let (result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE));

    assert_eq!(result, InputResult::None);
    assert_eq!(composer.draft.textarea.text(), "/model ");
    assert_eq!(
        composer.draft.textarea.cursor(),
        composer.draft.textarea.text().len()
    );
}
#[test]
fn slash_tab_then_enter_dispatches_builtin_command() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    // Type a prefix and complete with Tab, which inserts a trailing space
    // and moves the cursor beyond the '/name' token (hides the popup).
    type_chars_humanlike(&mut composer, &['/', 'd', 'i']);
    let (_res, _redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
    assert_eq!(composer.draft.textarea.text(), "/diff ");

    // Press Enter: should dispatch the command, not submit literal text.
    let (result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    match result {
        InputResult::Command(cmd) => assert_eq!(cmd.command(), "diff"),
        InputResult::CommandWithArgs(_, _, _) => {
            panic!("expected command dispatch without args for '/diff'")
        }
        InputResult::ServiceTierCommand(command) => {
            panic!("expected diff command, got service tier {command:?}")
        }
        InputResult::Submitted { text, .. } => {
            panic!("expected command dispatch after Tab completion, got literal submit: {text}")
        }
        InputResult::Queued { .. } => {
            panic!("expected command dispatch after Tab completion, got literal queue")
        }
        InputResult::None => panic!("expected Command result for '/diff'"),
    }
    assert!(composer.draft.textarea.is_empty());
}
#[test]
fn slash_command_elementizes_on_space() {
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

    type_chars_humanlike(&mut composer, &['/', 'p', 'l', 'a', 'n', ' ']);

    let text = composer.draft.textarea.text().to_string();
    let elements = composer.draft.textarea.text_elements();
    assert_eq!(text, "/plan ");
    assert_eq!(elements.len(), 1);
    assert_eq!(elements[0].placeholder(&text), Some("/plan"));
}
#[test]
fn slash_command_elementizes_only_known_commands() {
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

    type_chars_humanlike(&mut composer, &['/', 'U', 's', 'e', 'r', 's', ' ']);

    let text = composer.draft.textarea.text().to_string();
    let elements = composer.draft.textarea.text_elements();
    assert_eq!(text, "/Users ");
    assert!(elements.is_empty());
}
#[test]
fn slash_command_element_removed_when_not_at_start() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    type_chars_humanlike(&mut composer, &['/', 'r', 'e', 'v', 'i', 'e', 'w', ' ']);

    let text = composer.draft.textarea.text().to_string();
    let elements = composer.draft.textarea.text_elements();
    assert_eq!(text, "/review ");
    assert_eq!(elements.len(), 1);

    composer.draft.textarea.set_cursor(/*pos*/ 0);
    type_chars_humanlike(&mut composer, &['x']);

    let text = composer.draft.textarea.text().to_string();
    let elements = composer.draft.textarea.text_elements();
    assert_eq!(text, "x/review ");
    assert!(elements.is_empty());
}
#[test]
fn slash_mention_dispatches_command_and_inserts_at() {
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

    type_chars_humanlike(&mut composer, &['/', 'm', 'e', 'n', 't', 'i', 'o', 'n']);

    let (result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    match result {
        InputResult::Command(cmd) => {
            assert_eq!(cmd.command(), "mention");
        }
        InputResult::CommandWithArgs(_, _, _) => {
            panic!("expected command dispatch without args for '/mention'")
        }
        InputResult::ServiceTierCommand(command) => {
            panic!("expected mention command, got service tier {command:?}")
        }
        InputResult::Submitted { text, .. } => {
            panic!("expected command dispatch, but composer submitted literal text: {text}")
        }
        InputResult::Queued { .. } => {
            panic!("expected command dispatch, but composer queued literal text")
        }
        InputResult::None => panic!("expected Command result for '/mention'"),
    }
    assert!(
        composer.draft.textarea.is_empty(),
        "composer should be cleared"
    );
    composer.insert_str("@");
    assert_eq!(composer.draft.textarea.text(), "@");
}
#[test]
fn slash_plan_args_preserve_text_elements() {
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
    composer.set_collaboration_modes_enabled(/*enabled*/ true);

    type_chars_humanlike(&mut composer, &['/', 'p', 'l', 'a', 'n', ' ']);
    let placeholder = local_image_label_text(/*label_number*/ 1);
    composer.attach_image(PathBuf::from("/tmp/plan.png"));

    let (result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    match result {
        InputResult::CommandWithArgs(cmd, args, text_elements) => {
            assert_eq!(cmd.command(), "plan");
            assert_eq!(args, placeholder);
            assert_eq!(text_elements.len(), 1);
            assert_eq!(
                text_elements[0].placeholder(&args),
                Some(placeholder.as_str())
            );
        }
        _ => panic!("expected CommandWithArgs for /plan with args"),
    }
}
#[test]
fn slash_path_input_submits_without_command_error() {
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

    composer
        .draft
        .textarea
        .set_text_clearing_elements("/Users/example/project/src/main.rs");

    let (result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    if let InputResult::Submitted { text, .. } = result {
        assert_eq!(text, "/Users/example/project/src/main.rs");
    } else {
        panic!("expected Submitted");
    }
    assert!(composer.draft.textarea.is_empty());
    match rx.try_recv() {
        Ok(event) => panic!("unexpected event: {event:?}"),
        Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {}
        Err(err) => panic!("unexpected channel state: {err:?}"),
    }
}
#[test]
fn slash_with_leading_space_submits_as_text() {
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

    composer
        .draft
        .textarea
        .set_text_clearing_elements(" /this-looks-like-a-command");

    let (result, _needs_redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    if let InputResult::Submitted { text, .. } = result {
        assert_eq!(text, "/this-looks-like-a-command");
    } else {
        panic!("expected Submitted");
    }
    assert!(composer.draft.textarea.is_empty());
    match rx.try_recv() {
        Ok(event) => panic!("unexpected event: {event:?}"),
        Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {}
        Err(err) => panic!("unexpected channel state: {err:?}"),
    }
}
#[test]
fn slash_popup_not_activated_for_slash_space_text_history_like_input() {
    use crossterm::event::KeyCode;
    use crossterm::event::KeyEvent;
    use crossterm::event::KeyModifiers;
    use tokio::sync::mpsc::unbounded_channel;

    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    // Simulate history-like content: "/ test"
    composer.set_text_content("/ test".to_string(), Vec::new(), Vec::new());

    // After set_text_content -> sync_popups is called; popup should NOT be Command.
    assert!(
        matches!(composer.popups.active, ActivePopup::None),
        "expected no slash popup for '/ test'"
    );

    // Up should be handled by history navigation path, not slash popup handler.
    let (result, _redraw) =
        composer.handle_key_event(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
    assert_eq!(result, InputResult::None);
}
#[test]
fn slash_popup_activated_for_bare_slash_and_valid_prefixes() {
    // use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use tokio::sync::mpsc::unbounded_channel;

    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );

    // Case 1: bare "/"
    composer.set_text_content("/".to_string(), Vec::new(), Vec::new());
    assert!(
        matches!(composer.popups.active, ActivePopup::Command(_)),
        "bare '/' should activate slash popup"
    );

    // Case 2: valid prefix "/re" (matches /review, /resume, etc.)
    composer.set_text_content("/re".to_string(), Vec::new(), Vec::new());
    assert!(
        matches!(composer.popups.active, ActivePopup::Command(_)),
        "'/re' should activate slash popup via prefix match"
    );

    // Case 3: fuzzy match "/ac" (subsequence of /compact and /feedback)
    composer.set_text_content("/ac".to_string(), Vec::new(), Vec::new());
    assert!(
        matches!(composer.popups.active, ActivePopup::Command(_)),
        "'/ac' should activate slash popup via fuzzy match"
    );

    // Case 4: invalid prefix "/zzz" – still allowed to open popup if it
    // matches no built-in command; our current logic will not open popup.
    // Verify that explicitly.
    composer.set_text_content("/zzz".to_string(), Vec::new(), Vec::new());
    assert!(
        matches!(composer.popups.active, ActivePopup::None),
        "'/zzz' should not activate slash popup because it is not a prefix of any built-in command"
    );
}