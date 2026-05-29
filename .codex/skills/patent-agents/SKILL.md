---
name: patent-agents
description: 专利领域专业代理角色 — 9 个专利专家角色用于检索、分析、撰写、审查、质量评估
---

# 专利领域专业代理角色

本项目包含 9 个专利领域专业代理角色，可通过 `@角色名` 或任务分发调用。

## 角色列表

| 角色 | ID | 职责 | 核心工具 |
|------|-----|------|---------|
| 检索专家 | retriever | 多源专利检索、检索式构建、对比文件筛选 | PatentSearch, GooglePatentsFetch, IterativeSearch, KnowledgeSearch |
| 分析专家 | analyzer | 权利要求解析、特征提取、四层对比、本质识别 | ClaimParse, ClaimCompare, KnowledgeSearch |
| 撰写专家 | writer | 专利文件撰写（说明书/权利要求/摘要） | ClaimGenerator, SpecificationDrafter, AbstractDrafter |
| 新颖性评估专家 | novelty_checker | 三步法新颖性判断、逐特征对比 | ClaimCompare, NoveltyAnalysis, KnowledgeSearch |
| 创造性评估专家 | creativity_checker | 问题-解决方案法创造力分析 | InventivenessAnalysis, ClaimCompare, KnowledgeSearch |
| 侵权分析专家 | infringement_checker | 全面覆盖+等同原则侵权分析 | ClaimParse, InfringementAnalysis, KnowledgeSearch |
| 无效分析专家 | invalidity_checker | 无效理由和证据分析 | ClaimCompare, KnowledgeSearch |
| 文件审查专家 | reviewer | 格式规范和内容质量审查 | FormalCheck, ClaimParse, QualityAssess |
| 质量评估专家 | quality_checker | 多维度专利质量评估 | QualityAssess, FormalCheck, KnowledgeSearch |

## 使用方法

调用专利代理角色时，先加载 `patent-agents` 技能，然后通过子代理调用指定角色。

## 工具依赖

所有角色共同依赖的基础工具：read_file, glob_search, grep_search, TodoWrite, SendUserMessage