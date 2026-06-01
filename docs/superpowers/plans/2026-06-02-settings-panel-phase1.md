# 统一设置面板第一期 — 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在 TUI 中构建统一的 `/settings` 设置面板，包含通用/模型/权限/外观四个页签，新增动画/通知/原始输出/工具提示等开关。

**Architecture:** 新增一个 `BottomPaneView` 组件 `SettingsPanel`，内含 Tab 页签导航和每个页签的子渲染组件。各页签设置项通过 `AppEvent` → `event_dispatch.rs` → `config/batchWrite RPC` 持久化。模型/权限/外观页签在第一期显示引导提示，用户通过现有 `/` 命令操作。

**Tech Stack:** Rust, ratatui, codex_app_server_protocol (ConfigEdit)

---

### Task 1: Config 字段探查

**Files:** None（只读）

- [ ] **Step 1: 确认 Config 结构体中各设置项的字段路径**

在 `codex-rs/core/src/config/` 或 `codex-rs/config/src/` 中搜索以下字段的确切名称：

```bash
rg "tui\.animations|tui_animations|show_tooltips|raw_output_mode|tui\.notification" codex-rs/config/src/ codex-rs/core/src/config/ --type rust
```

记下每个字段的完整路径。例如可能是 `config.tui.animations`、`config.tui.show_tooltips`、`config.raw_output_mode` 等。

同时也确认 `ConfigEdit` 的 key_path（如 `"tui.animations"`、`"tui.show_tooltips"`、`"tui.raw_output_mode"`）和值的 JSON 格式。

- [ ] **Step 2: 确认 Result 后退出（无需提交）**

---

### Task 2: 添加 `AppEvent::UpdateGeneralSettings` 事件

**Files:**
- Modify: `codex-rs/tui/src/app_event.rs`
- Modify: `codex-rs/tui/src/config_update.rs`（新增 `build_general_settings_edits`）
- Modify: `codex-rs/tui/src/app/event_dispatch.rs`

- [ ] **Step 1: 在 `app_event.rs` 中添加新事件变体**

在 `UpdateMemorySettings`（约 line 778）附近添加：

```rust
    /// Update general TUI display/behavior settings.
    UpdateGeneralSettings {
        animations: bool,
        show_tooltips: bool,
        raw_output_mode: bool,
    },
```

- [ ] **Step 2: 在 `config_update.rs` 中添加构建函数**

```rust
pub(crate) fn build_general_settings_edits(
    animations: bool,
    show_tooltips: bool,
    raw_output_mode: bool,
) -> Vec<ConfigEdit> {
    vec![
        replace_config_value("tui.animations", serde_json::json!(animations)),
        replace_config_value("tui.show_tooltips", serde_json::json!(show_tooltips)),
        replace_config_value("tui.raw_output_mode", serde_json::json!(raw_output_mode)),
    ]
}
```

- [ ] **Step 3: 在 `event_dispatch.rs` 中处理新事件**

在 `AppEvent::UpdateMemorySettings` match 分支（约 line 1582）后面添加：

```rust
            AppEvent::UpdateGeneralSettings {
                animations,
                show_tooltips,
                raw_output_mode,
            } => {
                let edits = build_general_settings_edits(
                    animations,
                    show_tooltips,
                    raw_output_mode,
                );
                if let Err(err) = write_config_batch(
                    app_server.request_handle(),
                    edits,
                )
                .await
                {
                    self.chat_widget.add_error_message(
                        format!("写入配置失败: {err}")
                    );
                }
            }
```

注意：需要确保 `build_general_settings_edits` 已导入。在 `event_dispatch.rs` 顶部查找已有的 `use crate::config_update::*`，如果没有则添加。

- [ ] **Step 4: 编译检查**

Run: `cargo check -p codex-tui`
Expected: 编译通过

- [ ] **Step 5: 提交**

```bash
git add -A && git commit -m "feat(tui): 添加 UpdateGeneralSettings 事件和配置持久化"
```

---

### Task 3: 创建 `SettingsPanel` 主组件

**Files:**
- Create: `codex-rs/tui/src/bottom_pane/settings_panel.rs`
- Modify: `codex-rs/tui/src/bottom_pane/mod.rs`

- [ ] **Step 1: 创建 `settings_panel.rs`**

```rust
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

use crate::app_event_sender::AppEventSender;
use crate::bottom_pane::bottom_pane_view::BottomPaneView;
use crate::bottom_pane::CancellationEvent;
use crate::bottom_pane::selection_tabs;
use crate::bottom_pane::selection_tabs::SelectionTab;
use crate::render::Insets;
use crate::render::RectExt as _;
use crate::render::renderable::ColumnRenderable;
use crate::render::renderable::Renderable;

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
            .style(crate::style::user_message_style())
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
```

给 `SettingsTab` 添加辅助方法：

```rust
impl SettingsTab {
    // ... 已有 label() 和 all()

    fn command_name(self) -> &'static str {
        match self {
            SettingsTab::Model => "model",
            SettingsTab::Permissions => "permissions",
            SettingsTab::Appearance => "theme",
            _ => "",
        }
    }
}
```

注意：需要加上 `use crate::app_event::AppEvent;` — 检查是否遗漏。另外 `user_message_style()` 的导入路径可能是 `crate::style::user_message_style`。

- [ ] **Step 2: 在 `bottom_pane/mod.rs` 中注册模块**

在 `mod.rs` 的模块声明区域添加：

```rust
mod settings_panel;
pub(crate) use settings_panel::SettingsPanel;
```

- [ ] **Step 3: 编译检查**

Run: `cargo check -p codex-tui`
Expected: 编译通过。如有字段名/导入路径错误，根据编译器提示修正。

- [ ] **Step 4: 提交**

```bash
git add -A && git commit -m "feat(tui): 创建 SettingsPanel 主组件框架"
```

---

### Task 4: 创建 `SettingsGeneralView` 通用页签视图

**Files:**
- Create: `codex-rs/tui/src/bottom_pane/settings_general.rs`

- [ ] **Step 1: 实现通用页签视图（不依赖 AppEventSender）**

将保存逻辑与渲染逻辑分离，便于测试：

```rust
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

#[derive(Clone, Debug)]
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
    pub(crate) fn new(
        animations: bool,
        show_tooltips: bool,
        raw_output_mode: bool,
    ) -> Self {
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
            KeyEvent { code: KeyCode::Char(' '), modifiers: KeyModifiers::NONE, .. } => {
                if let Some(item) = self.items.get_mut(self.focus_idx) {
                    item.enabled = !item.enabled;
                }
                true
            }
            _ => false,
        }
    }

    pub(crate) fn current_values(&self) -> (bool, bool, bool) {
        let animations = self.items.iter()
            .find(|i| i.setting == GeneralSetting::Animations)
            .map(|i| i.enabled)
            .unwrap_or(true);
        let show_tooltips = self.items.iter()
            .find(|i| i.setting == GeneralSetting::ShowTooltips)
            .map(|i| i.enabled)
            .unwrap_or(true);
        let raw_output_mode = self.items.iter()
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
            let line = if idx == self.focus_idx {
                Line::from(text).cyan()
            } else {
                Line::from(text)
            };
            line.render(
                Rect { x: area.x, y, width: area.width, height: 1 },
                buf,
            );
        }
    }
}
```

- [ ] **Step 2: 编译检查**

Run: `cargo check -p codex-tui`
Expected: 编译通过

- [ ] **Step 3: 提交**

```bash
git add -A && git commit -m "feat(tui): 实现通用页签视图 SettingsGeneralView"
```

---

### Task 5: 注册 `/settings` 命令并路由到 SettingsPanel

**Files:**
- Modify: `codex-rs/tui/src/slash_command.rs`
- Modify: `codex-rs/tui/src/chatwidget/slash_dispatch.rs`
- Modify: `codex-rs/tui/src/chatwidget/settings_popups.rs`

- [ ] **Step 1: 更新 `SlashCommand::Settings` 的描述**

将 line 112 的 `SlashCommand::Settings` 描述改为：

```rust
            SlashCommand::Settings => "打开统一设置面板",
```

- [ ] **Step 2: 在 `settings_popups.rs` 中添加 open 方法**

在文件顶部添加导入：

```rust
use crate::bottom_pane::SettingsPanel;
```

添加方法（如有现有 `open_realtime_audio_popup` 方法，保留它。在文件末尾附近添加）：

```rust
    pub(crate) fn open_settings_panel(&mut self) {
        let view = SettingsPanel::new(
            self.config.tui_animations.unwrap_or(true),
            self.config.tui_show_tooltips.unwrap_or(true),
            self.config.raw_output_mode.unwrap_or(false),
            self.app_event_tx.clone(),
        );
        self.bottom_pane.show_view(Box::new(view));
    }
```

先编译一下确认 `self.config` 上这些字段的确切名称和类型，根据编译结果调整。

- [ ] **Step 3: 路由 `/settings` 命令**

在 `slash_dispatch.rs` 的 `dispatch_command` match 中，找到 `SlashCommand::Settings` 分支（line 216-221），替换为：

```rust
            SlashCommand::Settings => {
                self.open_settings_panel();
            }
```

- [ ] **Step 4: 编译检查**

Run: `cargo check -p codex-tui`
Expected: 编译通过

- [ ] **Step 5: 提交**

```bash
git add -A && git commit -m "feat(tui): 注册 /settings 命令并路由到 SettingsPanel"
```

---

### Task 6: 编写单元测试

**Files:**
- Modify: `codex-rs/tui/src/bottom_pane/settings_general.rs`（添加 `#[cfg(test)] mod tests`）

- [ ] **Step 1: 为 `SettingsGeneralView` 添加纯逻辑测试**

```rust
#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use super::*;

    #[test]
    fn test_initial_values() {
        let view = SettingsGeneralView::new(
            /*animations*/ true,
            /*show_tooltips*/ false,
            /*raw_output_mode*/ true,
        );
        let (anim, tips, raw) = view.current_values();
        assert_eq!(anim, true);
        assert_eq!(tips, false);
        assert_eq!(raw, true);
    }

    #[test]
    fn test_toggle_animation_with_space() {
        let mut view = SettingsGeneralView::new(true, false, false);
        // focus starts at index 0 (animations)
        view.handle_key_event(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE));
        let (anim, ..) = view.current_values();
        assert_eq!(anim, false);

        // toggle back
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
        // Select last item
        view.focus_idx = 2;
        let before = view.focus_idx;
        view.handle_key_event(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(view.focus_idx, before); // clamped

        // Up from first
        view.focus_idx = 0;
        view.handle_key_event(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(view.focus_idx, 0); // clamped
    }

    #[test]
    fn test_toggle_different_settings() {
        let mut view = SettingsGeneralView::new(true, true, false);
        // Toggle middle item
        view.focus_idx = 1;
        view.handle_key_event(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE));
        let (_, tips, _) = view.current_values();
        assert_eq!(tips, false);
    }
}
```

- [ ] **Step 2: 运行测试**

Run: `cargo test -p codex-tui -- settings_general`
Expected: 所有测试通过

- [ ] **Step 3: 提交**

```bash
git add -A && git commit -m "test(tui): 添加 SettingsGeneralView 单元测试"
```

---

### Task 7: 格式化 + 最终验证

- [ ] **Step 1: 运行 `just fmt`**

Run from `codex-rs/`: `just fmt`

- [ ] **Step 2: 编译检查**

Run: `cargo check -p codex-tui`
Expected: 编译通过

- [ ] **Step 3: 运行现有测试确保没破坏**

Run: `cargo test -p codex-tui`
Expected: 全部通过（如有个别快照测试需要更新，按项目惯例用 `cargo insta accept` 处理）

- [ ] **Step 4: 提交**

```bash
git add -A && git commit -m "style: rustfmt 格式化并最终验证"
```
