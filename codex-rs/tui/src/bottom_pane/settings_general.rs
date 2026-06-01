use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::Widget;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GeneralSetting {
    Animations,
    ShowTooltips,
    RawOutputMode,
}

struct GeneralSettingItem {
    setting: GeneralSetting,
    name: &'static str,
    description: &'static str,
    enabled: bool,
}

pub(crate) struct SettingsGeneralView {
    items: Vec<GeneralSettingItem>,
    focus_idx: usize,
}

impl SettingsGeneralView {
    pub(crate) fn new(animations: bool, show_tooltips: bool, raw_output_mode: bool) -> Self {
        Self {
            items: vec![
                GeneralSettingItem {
                    setting: GeneralSetting::Animations,
                    name: "启用动画",
                    description: "启动动画、闪烁效果和加载动效",
                    enabled: animations,
                },
                GeneralSettingItem {
                    setting: GeneralSetting::ShowTooltips,
                    name: "显示启动提示",
                    description: "启动时显示工具提示",
                    enabled: show_tooltips,
                },
                GeneralSettingItem {
                    setting: GeneralSetting::RawOutputMode,
                    name: "默认原始输出模式",
                    description: "启动时使用原始回滚模式（便于终端选择复制）",
                    enabled: raw_output_mode,
                },
            ],
            focus_idx: 0,
        }
    }

    pub(crate) fn handle_key_event(&mut self, key_event: KeyEvent) -> bool {
        match key_event {
            _ if key_event.code == KeyCode::Up => {
                self.focus_idx = self.focus_idx.saturating_sub(1);
                true
            }
            _ if key_event.code == KeyCode::Down => {
                if self.focus_idx + 1 < self.items.len() {
                    self.focus_idx += 1;
                }
                true
            }
            KeyEvent {
                code: KeyCode::Char(' '),
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                if let Some(item) = self.items.get_mut(self.focus_idx) {
                    item.enabled = !item.enabled;
                }
                true
            }
            _ => false,
        }
    }

    pub(crate) fn current_values(&self) -> (bool, bool, bool) {
        let animations = self
            .items
            .iter()
            .find(|i| i.setting == GeneralSetting::Animations)
            .map(|i| i.enabled)
            .unwrap_or(true);
        let show_tooltips = self
            .items
            .iter()
            .find(|i| i.setting == GeneralSetting::ShowTooltips)
            .map(|i| i.enabled)
            .unwrap_or(true);
        let raw_output_mode = self
            .items
            .iter()
            .find(|i| i.setting == GeneralSetting::RawOutputMode)
            .map(|i| i.enabled)
            .unwrap_or(false);
        (animations, show_tooltips, raw_output_mode)
    }

    pub(crate) fn render_body(&self, area: Rect, buf: &mut Buffer) {
        for (idx, item) in self.items.iter().enumerate() {
            let y = area.y + idx as u16;
            if y >= area.y + area.height {
                break;
            }
            let prefix = if idx == self.focus_idx { "›" } else { " " };
            let marker = if item.enabled { "x" } else { " " };
            let text = format!("{} [{}] {}", prefix, marker, item.name);
            if idx == self.focus_idx {
                Line::from(text).cyan().render(
                    Rect {
                        x: area.x,
                        y,
                        width: area.width,
                        height: 1,
                    },
                    buf,
                );
            } else {
                Line::from(text).render(
                    Rect {
                        x: area.x,
                        y,
                        width: area.width,
                        height: 1,
                    },
                    buf,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_initial_values() {
        let view = SettingsGeneralView::new(
            /*animations*/ true, /*show_tooltips*/ false, /*raw_output_mode*/ true,
        );
        let (anim, tips, raw) = view.current_values();
        assert_eq!(anim, true);
        assert_eq!(tips, false);
        assert_eq!(raw, true);
    }

    #[test]
    fn test_toggle_animation_with_space() {
        let mut view = SettingsGeneralView::new(true, false, false);
        view.handle_key_event(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE));
        let (anim, ..) = view.current_values();
        assert_eq!(anim, false);

        view.handle_key_event(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE));
        let (anim, ..) = view.current_values();
        assert_eq!(anim, true);
    }

    #[test]
    fn test_focus_navigation() {
        let mut view = SettingsGeneralView::new(true, false, false);
        assert_eq!(view.focus_idx, 0);

        view.handle_key_event(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(view.focus_idx, 1);

        view.handle_key_event(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(view.focus_idx, 0);
    }

    #[test]
    fn test_focus_stays_in_bounds() {
        let mut view = SettingsGeneralView::new(true, false, false);
        view.focus_idx = 2;
        let before = view.focus_idx;
        view.handle_key_event(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(view.focus_idx, before);

        view.focus_idx = 0;
        view.handle_key_event(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(view.focus_idx, 0);
    }

    #[test]
    fn test_toggle_different_settings() {
        let mut view = SettingsGeneralView::new(true, true, false);
        view.focus_idx = 1;
        view.handle_key_event(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE));
        let (_, tips, _) = view.current_values();
        assert_eq!(tips, false);
    }
}
