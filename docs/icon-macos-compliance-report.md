# BCIP 桌面端 Logo/Icon macOS 合规性验证报告

**检查日期**: 2026-06-03  
**检查范围**: `apps/desktop/src-tauri/icons/` 全部图标资源  
**参照标准**: Apple Human Interface Guidelines — App Icons (macOS)

---

## 一、macOS 图标规范要求摘要

| 要求项 | Apple 规范 |
|--------|-----------|
| 格式 | `.icns` (含全部尺寸) |
| 必须尺寸 | 16×16, 32×32, 128×128, 256×256, 512×512, 1024×1024 (@2x) |
| @2x 支持 | 每个 1x 尺寸必须有对应的 @2x 版本 |
| 形状 | macOS Big Sur+ 圆角矩形 (superellipse), 系统自动裁剪 |
| 透明通道 | 不可有全透明区域，图标应填满画布 |
| 色彩空间 | sRGB 或 Display P3 |
| DPI | 72 DPI (1x), 144 DPI (@2x) |
| 图标一致性 | icns、iconset、外层 PNG 必须全部源自同一 1024×1024 源图 |
| Info.plist | CFBundleIconFile 指向正确 .icns 文件 |
| 暗色/亮色 | 可选提供暗色调变体 |

---

## 二、文件清单与尺寸验证

### 2.1 外层 icons/ 目录（Tauri 实际使用）

| 文件 | 像素尺寸 | 格式 | 文件大小 | DPI | Alpha | 状态 |
|------|---------|------|---------|-----|-------|------|
| `32x32.png` | 32×32 | PNG | 1,123 B | 72 | yes | ✅ Tauri 引用 |
| `128x128.png` | 128×128 | PNG | 7,118 B | 72 | yes | ✅ Tauri 引用 |
| `128x128@2x.png` | 256×256 | PNG | 19,374 B | 72 | yes | ✅ Tauri 引用 |
| `icon.icns` | 1024×1024 | ICNS | 386,834 B | — | — | ✅ Tauri 引用 |
| `icon.ico` | 256×256 | ICO | 33,424 B | — | — | ✅ Tauri 引用 |
| `icon_16x16.png` | 16×16 | PNG | 1,979 B | 144 | yes | ⚠️ 未引用 |
| `icon_16x16@2x.png` | 32×32 | PNG | 4,110 B | 144 | yes | ⚠️ 未引用 |
| `icon_32x32.png` | 32×32 | PNG | 4,110 B | 144 | yes | ⚠️ 未引用 |
| `icon_32x32@2x.png` | 64×64 | PNG | 8,995 B | 144 | yes | ⚠️ 未引用 |
| `icon_128x128.png` | 128×128 | PNG | 6,512 B | 144 | yes | ⚠️ 未引用 |
| `icon_128x128@2x.png` | 256×256 | PNG | 17,375 B | 144 | yes | ⚠️ 未引用 |
| `icon_256x256.png` | 256×256 | PNG | **106,054 B** | 144 | yes | ⚠️ 异常偏大 |
| `icon_256x256@2x.png` | 512×512 | PNG | 370,379 B | 144 | yes | ⚠️ 未引用 |
| `icon_512x512.png` | 512×512 | PNG | 370,379 B | 144 | yes | ⚠️ 未引用 |
| `icon_512x512@2x.png` | 1024×1024 | PNG | **1,273,826 B** | 144 | yes | ⚠️ 异常偏大 |
| `icon.png` | 1024×1024 | PNG | 156,030 B | 72 | yes | 源文件 |
| `logo.png` | 512×512 | PNG | 370,379 B | 144 | yes | 内部使用 |

### 2.2 icon.iconset/ 目录（macOS 原生）

| 文件 | 像素尺寸 | 文件大小 | DPI | 状态 |
|------|---------|---------|-----|------|
| `icon_16x16.png` | 16×16 | 487 B | 72 | ✅ |
| `icon_16x16@2x.png` | 32×32 | 1,123 B | 72 | ✅ |
| `icon_32x32.png` | 32×32 | 1,123 B | 72 | ✅ |
| `icon_32x32@2x.png` | 64×64 | 2,799 B | 72 | ✅ |
| `icon_128x128.png` | 128×128 | 7,118 B | 72 | ✅ |
| `icon_128x128@2x.png` | 256×256 | 19,374 B | 72 | ✅ |
| `icon_256x256.png` | 256×256 | 19,374 B | 72 | ✅ |
| `icon_256x256@2x.png` | 512×512 | 55,971 B | 72 | ✅ |
| `icon_512x512.png` | 512×512 | 55,971 B | 72 | ✅ |
| `icon_512x512@2x.png` | 1024×1024 | 156,030 B | 72 | ✅ |

---

## 三、发现的问题

### 🔴 P0 — 严重问题

#### 1. 三套图标完全不一致（三个来源 = 三张不同的图）

项目的图标资源存在 **三套完全不同的图标**：

| 来源 | 特征 | 文件大小趋势 |
|------|------|-------------|
| **icon.iconset/** | 文件更小、更精简、72 DPI | 128×128 = 7KB, 512×512 = 56KB |
| **外层 icon_\*.png** | 文件极大、144 DPI | 128×128 = 6.5KB, **512×512 = 370KB**, **1024×1024 = 1.27MB** |
| **icns（构建产物）** | 第三套图标，与上面两者都不同 | 128×128 = 8.3KB, 512×512 = 67KB |

**全部 10 对文件哈希均不匹配**：iconset ≠ 外层 ≠ icns

**影响**：
- Dock 显示的图标（来自 icns）与 Finder 中不同尺寸预览可能显示不同的图
- 构建时 Tauri 可能使用不同来源，导致不同场景看到不同图标
- 用户对品牌认知产生混乱

#### 2. 外层 `icon_256x256.png` 和 `icon_512x512@2x.png` 文件异常巨大

| 文件 | 像素 | 文件大小 | 对比同类 |
|------|------|---------|---------|
| `icon_256x256.png` | 256×256 | **106 KB** | iconset 同尺寸 = 19 KB (5.5x) |
| `icon_512x512@2x.png` | 1024×1024 | **1.27 MB** | iconset 同尺寸 = 156 KB (8x) |

**原因推测**：外层文件可能使用了未优化的 PNG（如截图直接保存），或者包含了与 iconset 不同的、更复杂的图像内容。

#### 3. `icon.png` (1024×1024) 与 `icon_512x512@2x.png` (1024×1024) 不一致

两者同为 1024×1024 但哈希不同（`f14ecb0...` vs `4cb86ad2...`），说明并非从同一源图生成。

---

### 🟡 P1 — 中等问题

#### 4. DPI 混乱：外层 icon_\*.png 使用 144 DPI，iconset 使用 72 DPI

| 来源 | DPI | macOS 期望 |
|------|-----|-----------|
| icon.iconset/ | 72 DPI | ✅ 标准 |
| 32x32.png / 128x128.png / 128x128@2x.png | 72 DPI | ✅ 标准 |
| icon_\*.png (外层) | **144 DPI** | ⚠️ 非标准 |
| icon.png | 72 DPI | ✅ 标准 |

macOS 的 iconset 规范要求使用 72 DPI 作为基准，@2x 文件在逻辑上是 144 DPI 但通常也标记为 72 DPI。144 DPI 标记可能导致某些工具渲染时出现尺寸偏差。

#### 5. Tauri 配置引用不完整

```json
"icon": [
  "icons/32x32.png",
  "icons/128x128.png", 
  "icons/128x128@2x.png",
  "icons/icon.icns",
  "icons/icon.ico"
]
```

**缺失引用**：
- `icons/16x16.png` — macOS Finder 列表视图需要
- `icons/icon_256x256.png` — macOS Finder 大图标需要
- `icons/icon_512x512.png` — macOS Retina Finder 需要
- `icons/icon.png` — Tauri 通用源文件

虽然 `icon.icns` 理论上包含所有尺寸，但如果系统直接从 bundle 引用 PNG（如通知中心、分享菜单），缺失的文件可能导致回退到默认图标。

#### 6. 无 sRGB / Display P3 色彩配置文件

所有图标的 `hasProfile` 为 `<nil>`（无嵌入 ICC 配置文件）。

Apple 推荐为 macOS 图标嵌入 sRGB 或 Display P3 配置文件以确保色彩一致性。缺少配置文件可能导致图标在宽色域显示器上颜色偏移。

---

### 🟢 P2 — 改进建议

#### 7. 缺少暗色调变体（Dark Mode Variant）

macOS 支持 `AppIcon - Dark` 和 `AppIcon - Tinted` 变体。当前未提供，在深色桌面背景下如果图标有浅色底可能显得不协调。

**注意**：这取决于当前图标设计。如果图标本身深色调为主，则暗色变体不是必须的。

#### 8. ico 文件仅 256×256

`icon.ico` 仅包含 256×256 尺寸。Windows 11 推荐的 ICO 应包含 16/24/32/48/64/128/256 多个尺寸。如果只关注 macOS 则不是问题。

#### 9. Windows Store 图标未被 Tauri 配置引用

`Square*.png` 和 `StoreLogo.png` 文件存在但未被 Tauri bundle 配置引用，属于冗余文件。

#### 10. public/logo.png 和 public/mascot.png

两个文件大小完全相同（160,806 bytes），都是 1024×1024，但文件名不同。需要确认是否确实需要两份。

---

## 四、合规性总结

| 检查项 | macOS 要求 | 当前状态 | 评级 |
|--------|-----------|---------|------|
| ICNS 格式 | ✅ 必须 | ✅ 存在 icon.icns | 🟢 通过 |
| ICNS 包含所有尺寸 | 10 个尺寸 | ✅ 10 个尺寸 | 🟢 通过 |
| iconset 完整性 | 10 个文件 | ✅ 10 个文件 | 🟢 通过 |
| Info.plist CFBundleIconFile | 指向 icon.icns | ✅ 已配置 | 🟢 通过 |
| NSHighResolutionCapable | true | ✅ 已配置 | 🟢 通过 |
| Alpha 通道 | 存在 | ✅ 所有 PNG 均有 | 🟢 通过 |
| **三源一致性** | 必须统一 | ❌ 三套不同图标 | 🔴 **不通过** |
| **文件大小合理性** | 正常范围 | ❌ 部分文件异常大 | 🔴 **不通过** |
| **DPI 一致性** | 72 DPI 基准 | ⚠️ 外层混用 144 DPI | 🟡 需修正 |
| 色彩配置文件 | 推荐 sRGB/P3 | ⚠️ 无 ICC 配置 | 🟡 建议添加 |
| 暗色变体 | 可选 | ❌ 未提供 | 🟢 可选 |
| Tauri 配置完整性 | 全尺寸覆盖 | ⚠️ 引用不完整 | 🟡 需修正 |

---

## 五、修复建议（优先级排序）

### 🔴 紧急 — 统一图标源

1. **确定唯一源图**：选择一张 1024×1024 的 PNG 作为唯一源文件
2. **统一生成所有尺寸**：用脚本从源图一次性生成全部尺寸（1x + @2x）
3. **重新生成 iconset 和 icns**：使用 `iconutil -c icns` 从统一 iconset 生成 icns
4. **替换外层所有 icon_\*.png**：确保与 iconset 完全一致

### 🟡 重要 — 修正配置

5. **统一 DPI 为 72**：所有 1x 文件使用 72 DPI
6. **修正 Tauri icon 配置**：补全所有引用路径
7. **嵌入 sRGB ICC 配置文件**：确保色彩一致性

### 🟢 可选 — 增强

8. 设计并添加暗色调图标变体
9. 优化 Windows ICO（多尺寸嵌入）
10. 清理冗余的 Square*.png 文件

---

## 六、一键修复脚本（推荐）

```bash
# 前提：确定源图为 apps/desktop/src-tauri/icons/icon.png (1024×1024)
SOURCE="apps/desktop/src-tauri/icons/icon.png"

# Step 1: 从源图生成全部 PNG 尺寸
for size in 16 32 128 256 512; do
  sips -z $size $size "$SOURCE" --out "icons/icon_${size}x${size}.png"
  sips -z $((size*2)) $((size*2)) "$SOURCE" --out "icons/icon_${size}x${size}@2x.png"
done

# Step 2: 同步到 iconset
cp icons/icon_*.png icons/icon.iconset/

# Step 3: 重新生成 icns
iconutil -c icns icons/icon.iconset -o icons/icon.icns

# Step 4: 修正 DPI 为 72
for f in icons/icon_*.png; do
  sips -s dpiWidth 72 -s dpiHeight 72 "$f"
done
```

---

**报告结论**：项目的图标资源存在 **三套不一致的图标** 这一严重问题，需要立即统一。macOS 的基本格式要求（ICNS、iconset、Info.plist）已满足，但细节品质需要通过统一源图和优化生成流程来提升。
