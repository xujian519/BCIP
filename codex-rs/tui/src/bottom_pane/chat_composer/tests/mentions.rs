#[test]
fn set_connector_mentions_refreshes_open_mention_popup() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );
    composer.set_connectors_enabled(/*enabled*/ true);
    composer.set_text_content("$".to_string(), Vec::new(), Vec::new());
    assert!(matches!(composer.popups.active, ActivePopup::None));

    let connectors = vec![AppInfo {
        id: "connector_1".to_string(),
        name: "Notion".to_string(),
        description: Some("Workspace docs".to_string()),
        logo_url: None,
        logo_url_dark: None,
        distribution_channel: None,
        branding: None,
        app_metadata: None,
        labels: None,
        install_url: Some("https://example.test/notion".to_string()),
        is_accessible: true,
        is_enabled: true,
        plugin_display_names: Vec::new(),
    }];
    composer.set_connector_mentions(Some(ConnectorsSnapshot { connectors }));

    let ActivePopup::Skill(popup) = &composer.popups.active else {
        panic!("expected mention popup to open after connectors update");
    };
    let mention = popup
        .selected_mention()
        .expect("expected connector mention to be selected");
    assert_eq!(mention.insert_text, "$notion".to_string());
    assert_eq!(mention.path, Some("app://connector_1".to_string()));
}
#[test]
fn set_connector_mentions_skips_disabled_connectors() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );
    composer.set_connectors_enabled(/*enabled*/ true);
    composer.set_text_content("$".to_string(), Vec::new(), Vec::new());
    assert!(matches!(composer.popups.active, ActivePopup::None));

    let connectors = vec![AppInfo {
        id: "connector_1".to_string(),
        name: "Notion".to_string(),
        description: Some("Workspace docs".to_string()),
        logo_url: None,
        logo_url_dark: None,
        distribution_channel: None,
        branding: None,
        app_metadata: None,
        labels: None,
        install_url: Some("https://example.test/notion".to_string()),
        is_accessible: true,
        is_enabled: false,
        plugin_display_names: Vec::new(),
    }];
    composer.set_connector_mentions(Some(ConnectorsSnapshot { connectors }));

    assert!(
        matches!(composer.popups.active, ActivePopup::None),
        "disabled connectors should not appear in the mention popup"
    );
}
#[test]
fn set_plugin_mentions_refreshes_open_mention_popup() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );
    composer.set_text_content("$".to_string(), Vec::new(), Vec::new());
    assert!(matches!(composer.popups.active, ActivePopup::None));

    composer.set_plugin_mentions(Some(vec![PluginCapabilitySummary {
        config_name: "sample@test".to_string(),
        display_name: "Sample Plugin".to_string(),
        description: None,
        has_skills: true,
        mcp_server_names: vec!["sample".to_string()],
        app_connector_ids: Vec::new(),
    }]));

    let ActivePopup::Skill(popup) = &composer.popups.active else {
        panic!("expected mention popup to open after plugin update");
    };
    let mention = popup
        .selected_mention()
        .expect("expected plugin mention to be selected");
    assert_eq!(mention.insert_text, "$sample".to_string());
    assert_eq!(mention.path, Some("plugin://sample@test".to_string()));
}
#[test]
fn set_skill_mentions_refreshes_open_mention_popup() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );
    composer.set_text_content("$".to_string(), Vec::new(), Vec::new());
    assert!(matches!(composer.popups.active, ActivePopup::None));

    let skill_path = test_path_buf("/tmp/skill/SKILL.md").abs();
    composer.set_skill_mentions(Some(vec![SkillMetadata {
        name: "codex".to_string(),
        description: "主要的个人云熙专利智能体仓库技能。".to_string(),
        short_description: None,
        interface: None,
        dependencies: None,
        policy: None,
        path_to_skills_md: skill_path.clone(),
        scope: crate::test_support::skill_scope_user(),
        plugin_id: None,
    }]));

    let ActivePopup::Skill(popup) = &composer.popups.active else {
        panic!("expected mention popup to open after skills update");
    };
    let mention = popup
        .selected_mention()
        .expect("expected skill mention to be selected");
    assert_eq!(mention.insert_text, "$codex".to_string());
    assert_eq!(mention.path, Some(skill_path.display().to_string()));
}
#[test]
fn mention_items_show_plugin_owned_skill_and_app_duplicates() {
    let skill_path = test_path_buf("/tmp/repo/google-calendar/SKILL.md").abs();
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );
    composer.set_connectors_enabled(/*enabled*/ true);
    composer.set_text_content("$goog".to_string(), Vec::new(), Vec::new());
    composer.set_skill_mentions(Some(vec![SkillMetadata {
        name: "google-calendar:availability".to_string(),
        description: "Find availability and plan event changes".to_string(),
        short_description: None,
        interface: Some(SkillInterface {
            display_name: Some("Google Calendar".to_string()),
            short_description: None,
            icon_small: None,
            icon_large: None,
            brand_color: None,
            default_prompt: None,
        }),
        dependencies: None,
        policy: None,
        path_to_skills_md: skill_path.clone(),
        scope: crate::test_support::skill_scope_repo(),
        plugin_id: None,
    }]));
    composer.set_plugin_mentions(Some(vec![PluginCapabilitySummary {
        config_name: "google-calendar@debug".to_string(),
        display_name: "Google Calendar".to_string(),
        description: Some(
            "Connect Google Calendar for scheduling, availability, and event management."
                .to_string(),
        ),
        has_skills: true,
        mcp_server_names: vec!["google-calendar".to_string()],
        app_connector_ids: vec![AppConnectorId("google_calendar".to_string())],
    }]));
    composer.set_connector_mentions(Some(ConnectorsSnapshot {
        connectors: vec![AppInfo {
            id: "google_calendar".to_string(),
            name: "Google Calendar".to_string(),
            description: Some("Look up events and availability".to_string()),
            logo_url: None,
            logo_url_dark: None,
            distribution_channel: None,
            branding: None,
            app_metadata: None,
            labels: None,
            install_url: Some("https://example.test/google-calendar".to_string()),
            is_accessible: true,
            is_enabled: true,
            plugin_display_names: vec!["Google Calendar".to_string()],
        }],
    }));

    let mentions = composer.mention_items();
    assert_eq!(mentions.len(), 3);
    assert_eq!(mentions[0].category_tag, Some("[Skill]".to_string()));
    assert_eq!(mentions[0].path, Some(skill_path.display().to_string()));
    assert_eq!(mentions[0].display_name, "Google Calendar".to_string());
    assert_eq!(mentions[1].category_tag, Some("[Plugin]".to_string()));
    assert_eq!(
        mentions[1].path,
        Some("plugin://google-calendar@debug".to_string())
    );
    assert_eq!(mentions[2].category_tag, Some("[App]".to_string()));
    assert_eq!(mentions[2].path, Some("app://google_calendar".to_string()));
}
#[test]
fn plugin_mention_popup_snapshot() {
    snapshot_composer_state(
        "plugin_mention_popup",
        /*enhanced_keys_supported*/ false,
        |composer| {
            composer.set_text_content("$sa".to_string(), Vec::new(), Vec::new());
            composer.set_plugin_mentions(Some(vec![PluginCapabilitySummary {
                config_name: "sample@test".to_string(),
                display_name: "Sample Plugin".to_string(),
                description: Some(
                    "Plugin that includes the Figma MCP server and Skills for common workflows"
                        .to_string(),
                ),
                has_skills: true,
                mcp_server_names: vec!["sample".to_string()],
                app_connector_ids: vec![AppConnectorId("calendar".to_string())],
            }]));
        },
    );
}
#[test]
fn mention_popup_type_prefixes_snapshot() {
    snapshot_composer_state_with_width(
        "mention_popup_type_prefixes",
        /*width*/ 72,
        /*enhanced_keys_supported*/ false,
        |composer| {
            composer.set_connectors_enabled(/*enabled*/ true);
            composer.set_text_content("$goog".to_string(), Vec::new(), Vec::new());
            composer.set_skill_mentions(Some(vec![SkillMetadata {
                name: "google-calendar-skill".to_string(),
                description: "Find availability and plan event changes".to_string(),
                short_description: None,
                interface: Some(SkillInterface {
                    display_name: Some("Google Calendar".to_string()),
                    short_description: None,
                    icon_small: None,
                    icon_large: None,
                    brand_color: None,
                    default_prompt: None,
                }),
                dependencies: None,
                policy: None,
                path_to_skills_md: test_path_buf("/tmp/repo/google-calendar/SKILL.md").abs(),
                scope: crate::test_support::skill_scope_repo(),
                plugin_id: None,
            }]));
            composer.set_plugin_mentions(Some(vec![PluginCapabilitySummary {
            config_name: "google-calendar@debug".to_string(),
            display_name: "Google Calendar".to_string(),
            description: Some(
                "Connect Google Calendar for scheduling, availability, and event management."
                    .to_string(),
            ),
            has_skills: false,
            mcp_server_names: vec!["google-calendar".to_string()],
            app_connector_ids: Vec::new(),
        }]));
            composer.set_connector_mentions(Some(ConnectorsSnapshot {
                connectors: vec![AppInfo {
                    id: "google_calendar".to_string(),
                    name: "Google Calendar".to_string(),
                    description: Some("Look up events and availability".to_string()),
                    logo_url: None,
                    logo_url_dark: None,
                    distribution_channel: None,
                    branding: None,
                    app_metadata: None,
                    labels: None,
                    install_url: Some("https://example.test/google-calendar".to_string()),
                    is_accessible: true,
                    is_enabled: true,
                    plugin_display_names: Vec::new(),
                }],
            }));
        },
    );
}
#[test]
fn set_connector_mentions_excludes_disabled_apps_from_mention_popup() {
    let (tx, _rx) = unbounded_channel::<AppEvent>();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        sender,
        /*enhanced_keys_supported*/ false,
        "向云熙专利智能体提出任何需求".to_string(),
        /*disable_paste_burst*/ false,
    );
    composer.set_connectors_enabled(/*enabled*/ true);
    composer.set_text_content("$".to_string(), Vec::new(), Vec::new());

    let connectors = vec![AppInfo {
        id: "connector_1".to_string(),
        name: "Notion".to_string(),
        description: Some("Workspace docs".to_string()),
        logo_url: None,
        logo_url_dark: None,
        distribution_channel: None,
        branding: None,
        app_metadata: None,
        labels: None,
        install_url: Some("https://example.test/notion".to_string()),
        is_accessible: true,
        is_enabled: false,
        plugin_display_names: Vec::new(),
    }];
    composer.set_connector_mentions(Some(ConnectorsSnapshot { connectors }));

    assert!(matches!(composer.popups.active, ActivePopup::None));
}