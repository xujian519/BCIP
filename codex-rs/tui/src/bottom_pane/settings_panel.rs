use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use ratatui::buffer::Buffer;
use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::Block;
use ratatui::widgets::Widget;

use crate::app_event::AppEvent;
use crate::app_event_sender::AppEventSender;
use crate::bottom_pane::bottom_pane_view::BottomPaneView;
use crate::bottom_pane::CancellationEvent;
use crate::bottom_pane::selection_tabs;
use crate::bottom_pane::selection_tabs::SelectionTab;
use crate::render::Insets;
use crate::render::RectExt as _;
use crate::render::renderable::ColumnRenderable;
use crate::render::renderable::Renderable;
use crate::style::user_message_style;

use super::settings_general::SettingsGeneralView;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SettingsTab {
    General,
    Model,
    Permissions,
    Appearance,
}

impl SettingsTab {
    fn label(self) -> &'static str {
        match self {
            SettingsTab::General => "通用",
            SettingsTab::Model => "模型",
            SettingsTab::Permissions => "权限",
            SettingsTab::Appearance => "外观",
        }
    }

    fn all() -> [SettingsTab; 4] {
        [SettingsTab::General, SettingsTab::Model, SettingsTab::Permissions, SettingsTab::Appearance]
    }

    fn command_name(self) -> &'static str {
        match self {
            SettingsTab::Model => "model",
            SettingsTab::Permissions => "permissions",
            SettingsTab::Appearance => "theme",
            _ => "",
        }
    }
}

pub(crate) struct SettingsPanel {
    active_tab: SettingsTab,
    general_view: SettingsGeneralView,
    complete: bool,
    app_event_tx: AppEventSender,
    footer_hint: Line<'static>,
}

impl SettingsPanel {
    pub(crate) fn new(
        animations: bool,
        show_tooltips: bool,
        raw_output_mode: bool,
        app_event_tx: AppEventSender,
    ) -> Self {
        Self {
            active_tab: SettingsTab::General,
            general_view: SettingsGeneralView::new(
                animations,
                show_tooltips,
                raw_output_mode,
            ),
            complete: false,
            app_event_tx,
            footer_hint: vec![
                "Tab 切换页签 ".dim(),
                "↑↓ 导航 ".dim(),
                "Space 切换 ".dim(),
                "Esc 关闭".dim(),
            ].into(),
        }
    }

    fn next_tab(&mut self) {
        let tabs = SettingsTab::all();
        let idx = tabs.iter().position(|t| *t == self.active_tab).unwrap_or(0);
        self.active_tab = tabs[(idx + 1) % tabs.len()];
    }

    fn prev_tab(&mut self) {
        let tabs = SettingsTab::all();
        let idx = tabs.iter().position(|t| *t == self.active_tab).unwrap_or(0);
        self.active_tab = tabs[(idx + tabs.len() - 1) % tabs.len()];
    }

    fn tab_bar_items(&self) -> Vec<SelectionTab> {
        SettingsTab::all().iter().map(|tab| SelectionTab {
            id: tab.label().to_string(),
            label: tab.label().to_string(),
            header: Box::new(ColumnRenderable::new()),
            items: Vec::new(),
        }).collect()
    }

    fn active_tab_idx(&self) -> usize {
        SettingsTab::all().iter().position(|t| *t == self.active_tab).unwrap_or(0)
    }

    fn save_and_close(&mut self) {
        if self.active_tab == SettingsTab::General {
            let (animations, show_tooltips, raw_output_mode) = self.general_view.current_values();
            self.app_event_tx.send(AppEvent::UpdateGeneralSettings {
                animations,
                show_tooltips,
                raw_output_mode,
            });
        }
        self.complete = true;
    }
}

impl BottomPaneView for SettingsPanel {
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event {
            KeyEvent { code: KeyCode::Tab, modifiers: KeyModifiers::NONE, .. } => {
                self.next_tab();
            }
            KeyEvent { code: KeyCode::BackTab, .. } => {
                self.prev_tab();
            }
            _ if self.active_tab == SettingsTab::General
                && self.general_view.handle_key_event(key_event) => {}
            KeyEvent { code: KeyCode::Esc, .. } => {
                self.save_and_close();
            }
            _ => {}
        }
    }

    fn is_complete(&self) -> bool {
        self.complete
    }

    fn on_ctrl_c(&mut self) -> CancellationEvent {
        self.save_and_close();
        CancellationEvent::Handled
    }
}

impl Renderable for SettingsPanel {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let [content_area, footer_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

        Block::default()
            .style(user_message_style())
            .render(content_area, buf);

        let tabs = self.tab_bar_items();
        let active_idx = self.active_tab_idx();
        let tab_bar_h = selection_tabs::tab_bar_height(&tabs, active_idx, content_area.width.saturating_sub(4));
        let inset = content_area.inset(Insets::vh(1, 2));
        let [tab_bar_area, _, body_area] = Layout::vertical([
            Constraint::Length(tab_bar_h),
            Constraint::Length(1),
            Constraint::Fill(1),
        ]).areas(inset);

        selection_tabs::render_tab_bar(&tabs, active_idx, tab_bar_area, buf);

        match self.active_tab {
            SettingsTab::General => {
                self.general_view.render_body(body_area, buf);
            }
            other => {
                let hint = match other {
                    SettingsTab::Model => "模型设置",
                    SettingsTab::Permissions => "权限设置",
                    SettingsTab::Appearance => "外观设置",
                    _ => unreachable!(),
                };
                let lines = vec![
                    Line::from(format!("  {hint}")).bold(),
                    Line::from(""),
                    Line::from(format!("  请使用 /{} 命令打开独立设置", other.command_name())).dim(),
                    Line::from("  后续版本将集成到统一面板").dim(),
                ];
                for (i, line) in lines.iter().enumerate() {
                    let y = body_area.y + i as u16;
                    if y >= body_area.y + body_area.height { break; }
                    line.clone().render(
                        Rect { x: body_area.x, y, width: body_area.width, height: 1 },
                        buf,
                    );
                }
            }
        }

        let hint_area = Rect {
            x: footer_area.x + 2,
            y: footer_area.y,
            width: footer_area.width.saturating_sub(2),
            height: footer_area.height,
        };
        self.footer_hint.clone().dim().render(hint_area, buf);
    }

    fn desired_height(&self, _width: u16) -> u16 {
        14
    }
}
