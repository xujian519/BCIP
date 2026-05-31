# 开发进度记录

## 2026-06-01: 专利撰写技能统一合并

### 背景
本机存在4个专利撰写相关技能，功能高度重叠：
- `patent-drafting-v2`（shared-skills，12阶段实战版）
- `patent-drafting-workflow`（shared-skills，7步Python编排版）
- `patent-drafting`（shared-skills，Athena TUI流程版）
- `patent-drafting`（.agents/skills，重复副本）

### 执行
1. **分析对比**：三个技能核心流程一致（交底书→分析→检索→撰写→质检），差异在流程颗粒度和工具绑定方式
2. **合并策略**：以 `patent-drafting-v2` 为主体，吸收其他两个精华
   - 从 `patent-drafting`（Athena版）吸收：宪法原则P1-P6、TUI工具绑定表、权利要求分层模板、权利要求类型模板、说明书章节规范、7维度质量评分+迭代规则、模型推荐配置、工作流快检清单
   - 从 `patent-drafting-workflow` 吸收：Python编排脚本（workflow.py/patent-draft.py/zhiwei_loader.py），去除fast/smart/deep自动模式
3. **清理**：
   - 删除 `shared-skills/patent-legal/patent-drafting/`
   - 删除 `shared-skills/patent-legal/patent-drafting-workflow/`（含所有脚本）
   - 删除 `.agents/skills/patent-drafting/`
4. **引用更新**：更新 SKILL-INDEX.md 和 profiles（xiaonuo/yunxi/yunpat/claude-code）
5. **结果**：本机仅保留一个 `patent-drafting-v2`（v3.0），12阶段纯HITL模式

### 技能路径
`/Users/xujian/shared-skills/patent-legal/patent-drafting-v2/SKILL.md`

### 包含脚本
- workflow.py（纯HITL编排器）
- patent-draft.py（CLI入口）
- zhiwei_loader.py（芷薇能力加载器）
