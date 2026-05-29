# Andrej Karpathy（卡帕西）知识库构建方法与理念 -- 深度调研报告

> 调研日期：2026-04-13
> 调研范围：个人知识管理、教学方法论、开源项目设计哲学、AI辅助知识管理

---

## 一、核心总览：卡帕西的知识管理哲学

Andrej Karpathy 的知识管理哲学可以概括为一句话：**最小化知识捕获的摩擦，用 AI 作为主动的编译器和组织者，而非被动的问答工具。**

他的知识管理栈从轻到重分为四个层级：

| 层级 | 工具/方法 | 用途 |
|------|----------|------|
| 快速捕获 | 单一 Apple Notes 文件（append-and-review） | 随手记录想法、TODO、引用 |
| 深度知识库 | LLM Wiki（Markdown + Obsidian + Claude Code） | 结构化、可持续积累的知识体系 |
| 自动化研究 | Autoresearch（自主实验循环） | AI 代理自动迭代研究和实验 |
| 多视角分析 | LLM Council（多模型咨询） | 向多个 LLM 同时提问并交叉验证 |

---

## 二、个人知识管理系统

### 2.1 "Append-and-Review" 笔记法

2025年3月19日，卡帕西在 Bear Blog 上发表了 [The append-and-review note](https://karpathy.bearblog.dev/the-append-and-review-note/) 一文，详细描述了他使用多年的笔记方法。

**数据结构：** 维护一个单一的文本笔记，放在 Apple Notes 中，就叫 "notes"。不做文件夹分类，不加标签体系，不做递归子结构。一个笔记意味着 CTRL+F 搜索简单直接。

**追加（Append）：** 任何时候有任何想法、TODO 或其他内容，直接追加到笔记的顶部，纯文本形式。无论是在电脑上工作还是在手机上出门在外。默认不加任何结构化元数据（日期、链接、概念、标签），唯一的例外是使用 `watch:`、`listen:`、`read:` 这样的标签前缀，方便在特定场景下 CTRL+F 搜索。

**回顾（Review）：** 随着新内容不断添加到顶部，旧内容自然下沉，仿佛受到重力作用。每隔一段时间，向下滚动浏览，如果发现值得保留的内容，复制粘贴到顶部进行"救援"。有时候会合并、处理、分组相关笔记。很少删除笔记。不值得关注的笔记会自然持续下沉——它们不会丢失，只是不再占据首要位置。

**卡帕西原话：**
> "When I note something down, I feel that I can immediately move on, wipe my working memory, and focus fully on something else at that time. I have confidence that I'll be able to revisit that idea later during review and process it when I have more time."

> "My note has grown quite giant over the last few years. It feels nice to scroll through some of the old things/thoughts that occupied me a long time ago."

**使用场景举例（来自原文）：**
- 突然冒出的随机想法，但在路上无法深思，先添加到笔记中
- 派对上有人推荐了一部电影
- 刷 X 时看到一篇好书推荐
- 早上坐下来写当天的 TODO 清单
- 需要一个临时写作面来思考某件事
- 想发一条推文但需要更多思考，先粘贴到笔记中
- 阅读论文时想记下一些有趣的数据
- 需要一个临时表面来 CTRL+C / CTRL+V 一些内容
- 运行超参数搜索时记录运行的命令和实验结果
- 感到焦虑有太多事情在脑海中，快速列个清单倾倒出来

### 2.2 LLM Wiki -- 编译器类比的知识库

2026年4月4日，卡帕西在 GitHub 上发布了一个 [LLM Wiki Gist](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f)（获得 5000+ Stars、3600+ Forks），描述了一种用 LLM 构建个人知识库的模式。

**核心思想：** 大多数人与 LLM 和文档的交互方式是 RAG——上传文件集合，LLM 在查询时检索相关片段并生成答案。但这存在问题：LLM 每次都在从头重新发现知识，没有积累。问一个需要综合五份文档的微妙问题，LLM 每次都要重新寻找和拼凑相关片段。

卡帕西提出的方案不同：不是在查询时从原始文档中检索，而是让 LLM **增量地构建和维护一个持久的 wiki**——一个结构化的、相互链接的 Markdown 文件集合，介于你和原始来源之间。添加新来源时，LLM 不仅仅是索引它以备后续检索，而是阅读它、提取关键信息、整合到现有 wiki 中——更新实体页面、修订主题摘要、标注新旧数据矛盾之处。

**卡帕西原话：**
> "The wiki is a persistent, compounding artifact. The cross-references are already there. The contradictions have already been flagged. The synthesis already reflects everything you've read. The wiki keeps getting richer with every source you add and every question you ask."

**三层架构：**

1. **Raw Sources（原始来源）** -- 策划收集的原始文档（文章、论文、图片、数据文件）。这些是不可变的——LLM 只读取，不修改。这是事实来源。

2. **The Wiki（知识库）** -- LLM 生成的 Markdown 文件目录。摘要、实体页面、概念页面、比较、概述、综合分析。LLM 完全拥有这一层——创建页面、在来源到达时更新、维护交叉引用、保持一致性。你阅读它；LLM 编写它。

3. **The Schema（模式文件）** -- 一个文档（如 Claude Code 的 CLAUDE.md 或 Codex 的 AGENTS.md），告诉 LLM wiki 如何组织、遵循什么约定、在摄取来源/回答问题/维护 wiki 时遵循什么工作流。这是关键配置文件——让 LLM 成为有纪律的 wiki 维护者而非通用聊天机器人。

**三个核心操作：**

- **Ingest（摄取）：** 将新来源放入 raw 目录，告诉 LLM 处理。LLM 阅读来源、与你讨论要点、在 wiki 中写摘要页面、更新索引、更新相关实体和概念页面、在日志中追加记录。单个来源可能触及 10-15 个 wiki 页面。

- **Query（查询）：** 对 wiki 提问。LLM 搜索相关页面、阅读并综合答案并附带引用。重要洞察：好的答案可以作为新页面归档回 wiki——你请求的比较、分析、发现的联系都是宝贵的，不应消失在聊天历史中。

- **Lint（健康检查）：** 定期让 LLM 检查 wiki 的健康状况——页面间的矛盾、被新来源取代的过时声明、没有入站链接的孤儿页面、提到但缺少独立页面的重要概念、缺失的交叉引用、可通过网络搜索填补的数据空白。

**两个特殊文件：**

- **index.md** -- 内容导向的目录，每个页面列出链接、一行摘要和可选元数据。LLM 每次摄取时更新。在中等规模（~100个来源，~数百页面）下效果出奇地好，避免了基于嵌入的 RAG 基础设施的需求。

- **log.md** -- 按时间顺序的追加式记录。每条记录使用一致的前缀格式（如 `## [2026-04-02] ingest | Article Title`），使得可以用简单的 unix 工具解析。

**工具链：**
- Obsidian 作为 IDE 浏览 wiki
- LLM Agent（Claude Code / OpenAI Codex）作为程序员
- Obsidian Web Clipper 浏览器扩展快速将网页文章转为 Markdown
- 整个 wiki 就是一个 git 仓库的 Markdown 文件——自动获得版本历史、分支和协作能力

**与 RAG 的本质区别（卡帕西原话）：**
> "Instead of just retrieving from raw documents at query time, the LLM incrementally builds and maintains a persistent wiki. The knowledge is compiled once and then kept current, not re-derived on every query."

**为什么这行得通（卡帕西原话）：**
> "The tedious part of maintaining a knowledge base is not the reading or the thinking — it's the bookkeeping. Updating cross-references, keeping summaries current, noting when new data contradicts old claims, maintaining consistency across dozens of pages. Humans abandon wikis because the maintenance burden grows faster than the value. LLMs don't get bored, don't forget to update a cross-reference, and can touch 15 files in one pass."

> "The human's job is to curate sources, direct the analysis, ask good questions, and think about what it all means. The LLM's job is everything else."

### 2.3 Autoresearch -- 自主 AI 研究循环

卡帕西开源了 [autoresearch](https://github.com/karpathy/autoresearch)（约31,000 Stars），这是一个让 AI 代理自主运行实验循环的项目。核心设计：给 AI 代理一个小型但真实的 LLM 训练环境，让它在夜间自主实验。

**循环流程：** 编辑代码 -> 运行实验（5分钟） -> 评估结果 -> 保留/丢弃 -> 重复

实践成果：一个 GPU 上 overnight 可以运行 50-700 个实验。《财富》杂志将其称为 **"The Karpathy Loop"**。

### 2.4 LLM Council -- 多模型咨询

卡帕西开发了 [llm-council](https://github.com/karpathy/llm-council)，一个简单的本地 Web 应用，将问题同时发送给多个 LLM（GPT、Claude、Gemini、Grok），让它们相互批判答案。这相当于一个个人顾问委员会。

---

## 三、教学材料组织方式

### 3.1 Stanford CS231n -- 深度学习课程的开创

卡帕西设计并教授了斯坦福大学第一门深度学习课程 [CS231n](https://cs231n.stanford.edu/2016/)（卷积神经网络用于视觉识别）。

**课程设计哲学：**

| 方面 | 细节 |
|------|------|
| **端到端聚焦** | 强调学习完整流水线——从原始数据到最终预测——而非孤立组件 |
| **实践优先** | 整合工程技巧、真实世界的实现细节，以及需要从零构建模型的作业 |
| **完全开放** | 所有讲座视频、幻灯片和详细课程笔记都在网上免费提供 |
| **直觉先行** | 优先建立直觉理解，再深入数学形式化 |

课程笔记（[cs231n.github.io](https://cs231n.github.io/)）被广泛认为是学习深度学习和计算机视觉最好的免费资源之一。

### 3.2 Neural Networks: Zero to Hero -- 从零到英雄系列

这是卡帕西在 YouTube 上的免费课程系列（[课程主页](https://karpathy.ai/zero-to-hero.html)），体现了他的核心教学理念。

**课程结构（螺旋上升式）：**

1. **micrograd** -- 从零构建一个 ~150 行的 Python 自动微分引擎
2. **语言建模入门** -- makemore 系列，构建字符级语言模型
3. **WaveNet** -- 构建更复杂的语言模型
4. **GPT 从零构建** -- 从空笔记本到训练好的 GPT 模型

**教学方法论关键词：**

- **"Spelled-out intro"（详细阐述入门）** -- 每个讲座标题都包含这个词，意味着一步一步，不偷懒
- **螺旋式课程** -- 早期引入的概念（如自动微分）在后续讲座中会以更大深度重现
- **直觉与严谨并重** -- 确保学习者不仅理解"怎么做"，还理解"为什么"
- **实时代码编写** -- 所有学习都通过现场编码会话完成
- **不使用高层框架** -- 一切都用 Python 从头编码

### 3.3 LLM101n -- Eureka Labs 的课程

卡帕西创立的 Eureka Labs（[eurekalabs.ai](https://eurekalabs.ai/)）推出了 LLM101n 课程，描述为"一个 AI 原生的学校"，愿景是 **Teacher + AI Symbiosis（教师与 AI 共生）** 模式。

---

## 四、开源项目与结构化方法

### 4.1 项目设计哲学总览

卡帕西的所有项目共享一套一致的设计原则：

| 原则 | 描述 |
|------|------|
| **极简主义** | 只保留最核心的代码，剥离所有不必要的东西 |
| **代码即文档** | 写得好、最少的代码本身就是最好的文档 |
| **可读性优先于性能** | 优先保证清晰度，不做前沿优化 |
| **教育导向** | 每一行都应该是可以被阅读和理解的 |
| **扁平目录结构** | 最少的嵌套层级 |
| **尽可能单文件实现** | 降低理解门槛 |
| **零依赖或极少依赖** | 避免框架臃肿 |

### 4.2 micrograd -- 极简自动微分引擎

- 约 150 行 Python 代码的标量反向传播（autograd）引擎
- 提供 PyTorch 风格的接口
- 只有约两个 Python 文件
- 目的：揭示神经网络在最底层是如何工作的
- 关键洞察：使用标量值反向传播来教授深度学习的核心机制

### 4.3 nanoGPT -- 最简 GPT 训练仓库

- [github.com/karpathy/nanogpt](https://github.com/karpathy/nanogpt)
- "The simplest, fastest repository for training/finetuning medium-sized GPTs"
- 核心文件只有 `model.py`、`train.py`、`config/`
- 卡帕西自己的评论（Hacker News）："aspire to spell everything out"
- 设计为极易 hack、从零训练、或微调预训练检查点
- 社区衍生项目包括 nanoMoE（混合专家）、modded-nanoGPT（速度优化版）

### 4.4 minGPT -- 最小 PyTorch GPT 实现

- [github.com/karpathy/minGPT](https://github.com/karpathy/minGPT)
- "A minimal PyTorch re-implementation of the GPT for both training and inference"
- 小巧、干净、可解释、教育性
- 代码本身即为文档

### 4.5 llm.c -- 纯 C/CUDA 实现 LLM 训练

- [github.com/karpathy/llm.c](https://github.com/karpathy/llm.c)
- ~5,000 行纯 C/CUDA，**无需 245MB 的 PyTorch 或 107MB 的 cPython**
- 只训练 GPT-2 -- 故意单一目的
- 24小时 ~$672 完成完整 GPT-2 预训练
- 被形容为"几乎叛逆的极简主义：保持最小、人类可读、可 hack"
- 不像那些淹没在抽象中的臃肿框架，llm.c 将一切还原到第一性原理

### 4.6 microgpt -- 200 行纯 Python GPT

- 2026年2月发布在 [karpathy.github.io](http://karpathy.github.io/2026/02/12/microgpt/)
- 单文件、~200 行纯 Python、零依赖
- 训练和推理一个 GPT 模型
- 复杂思想的极致蒸馏

### 4.7 minbpe -- 最简 BPE 分词器

- [github.com/karpathy/minbpe](https://github.com/karpathy/minbpe)
- "Minimal, clean code for the byte-level BPE algorithm"
- 配套讲座/教程讲解如何从零构建 GPT 分词器

### 4.8 rendergit -- 代码库扁平化阅读工具

- [github.com/karpathy/rendergit](https://github.com/karpathy/rendergit)
- 将任意 git 仓库渲染成单一静态 HTML 页面
- 提供语法高亮、Markdown 渲染和干净的侧边栏导航
- 为人类和 LLM 都设计——方便整体消费代码库
- 直接体现"代码即文档"理念

### 4.9 autoresearch -- 自主实验循环

- [github.com/karpathy/autoresearch](https://github.com/karpathy/autoresearch)
- ~630 行 Python，~31,000 Stars
- 单 GPU overnight 运行 50-100+ 实验
- 设计模式被推广为 "The Karpathy Loop"

### 4.10 llm-council -- 多模型咨询

- [github.com/karpathy/llm-council](https://github.com/karpathy/llm-council)
- 简单的本地 Web 应用
- 通过 OpenRouter 将查询发送给多个 LLM
- 让它们协作回答最难的问题

---

## 五、博客文章和写作中体现的知识管理哲学

### 5.1 写作平台选择

卡帕西使用两个博客平台：
- **karpathy.github.io** -- 长篇技术文章（如 "A Recipe for Training Neural Networks"）
- **karpathy.bearblog.dev** -- 短篇随笔（Bear Blog，一个极简博客平台）

这体现了他对极简主义的追求——Bear Blog 以简洁著称，没有多余功能。

### 5.2 "A Recipe for Training Neural Networks" -- 方法论的经典

这篇 2019 年的文章是理解卡帕西方法论的最佳窗口。

**两个核心观察：**
1. **神经网络训练是一个有漏洞的抽象（leaky abstraction）** -- 不像标准软件那样有干净的 API 和抽象
2. **神经网络训练会静默失败** -- 大部分时间会训练但悄悄地变差

**卡帕西原话：**
> "A 'fast and furious' approach to training neural networks does not work and only leads to suffering."

> "The qualities that in my experience correlate most strongly to success in deep learning are **patience and attention to detail**."

**六步方法论：**

1. **与数据合为一体（Become one with the data）** -- 花大量时间（以小时计）浏览数千个样本，理解分布，寻找模式。在触碰任何神经网络代码之前先做这步。

2. **搭建端到端训练/评估骨架 + 获取基线** -- 使用最简单的模型，固定随机种子，关闭数据增强，验证初始损失值，过拟合单个 batch。

3. **过拟合（Overfit）** -- 先找到一个足够大的模型使其能过拟合（聚焦训练损失），然后再正则化。"Don't be a hero" -- 直接复制相关论文中最简单的架构。

4. **正则化（Regularize）** -- 最好的正则化是更多真实数据。然后是数据增强、预训练、减少输入维度、减小模型、减小 batch size、dropout、权重衰减、早停。

5. **调参（Tune）** -- 随机搜索优于网格搜索。卡帕西开玩笑说"最先进的超参数优化方法是用一个实习生"。

6. **榨干最后一滴（Squeeze out the juice）** -- 模型集成几乎保证在任何事情上提升2%准确率。"Leave it training" -- 有一次他寒假期间忘记停止训练，回来后发现已经是 SOTA。

### 5.3 "Digital Hygiene" -- 数字卫生

2025年3月17日发表在 Bear Blog 上，从最基础到更小众的数字卫生建议。体现了他对工具选择和数字生活的系统性思考。

### 5.4 写作风格特点

- **极度坦诚** -- 他会主动标记自己过时的教程为"不应再使用"
- **实用主义** -- 每篇都提供可操作的具体建议
- **渐进式构建** -- 从简单到复杂的叙事结构
- **丰富的可视化** -- 强调"obsessed with visualizations of basically every possible thing"

---

## 六、学习方法论

### 6.1 成为专家的三步框架

卡帕西在 X（Twitter）上分享了成为任何领域专家的方法：

**原文：**
> "How to become expert at thing:
> 1. Iteratively take on concrete projects and accomplish them depth-wise, learning "on demand" (i.e., don't learn everything before starting — learn as you go).
> 2. Teach / explain what you learn (Feynman Technique)."

总结为三个步骤：

| 步骤 | 原则 | 描述 |
|------|------|------|
| **1** | **项目驱动学习** | 迭代式地承担具体项目并深入完成。不要在开始前试图学完一切——边做边学。 |
| **2** | **费曼技巧（教/解释）** | 教授你所学的东西。写博客、做视频、向他人解释概念，迫你巩固理解并暴露盲点。 |
| **3** | **迭代与分享** | 将作品发布到世界上，协作，获取反馈。构建-学习-解释的持续迭代复利成专业知识。 |

### 6.2 "Learning by Doing" -- 从零构建哲学

这是贯穿卡帕西所有工作的核心方法论：

- **从第一性原理出发** -- 将主题分解到基本组成部分，从底层构建理解
- **"深度学习"（Depthwise Learning）** -- 深入一个主题而非停留在表面
- **代码作为理解的载体** -- 如果你不能从零构建它，你就没有真正理解它
- **复制已知结果** -- 复现论文/模型（如 GPT-2）来构建深层直觉
- **动手编码优先** -- 所有著名课程都是实时代码编写会话

### 6.3 一万小时定律

卡帕西提倡通过做项目来学习，坚持"1万小时定律"——这与他持续多年的实践完全一致。从 CS231n（2015）到 Zero to Hero（2022）到 LLM Wiki（2026），他在 AI 教育领域持续深耕。

---

## 七、AI 辅助知识管理的看法

### 7.1 LLM 作为知识编译器

卡帕西对 AI 辅助知识管理最核心的贡献是 **"编译器类比"**：

| 类比 | 软件工程 | 知识管理 |
|------|---------|---------|
| 源代码 | 原始文件（论文、笔记、书签） | Raw Documents |
| 编译器 | GCC/LLM | LLM Agent |
| 可执行文件 | 编译后的二进制 | 结构化的 Wiki |

你不会直接运行源代码——你先编译它。同样地，你不应该直接查询原始文档——先用 LLM 将它们"编译"成结构良好的、相互链接的知识库，然后再查询编译后的输出。

### 7.2 与 Vannevar Bush 的 Memex 的精神联系

卡帕西自己在 LLM Wiki Gist 中写道：

> "The idea is related in spirit to Vannevar Bush's Memex (1945) — a personal, curated knowledge store with associative trails between documents. Bush's vision was closer to this than to what the web became: private, actively curated, with the connections between documents as valuable as the documents themselves. The part he couldn't solve was who does the maintenance. The LLM handles that."

### 7.3 对传统 RAG 的批评

卡帕西认为 RAG 的问题在于：
- 每次查询都从头重新发现知识
- 没有积累效应
- 问需要综合多个文档的问题时效率低下
- 没有什么是被"构建起来"的

### 7.4 Eureka Labs 的愿景

卡帕西选择教育而非其他可能更赚钱的 AI 领域，创立了 **Eureka Labs**（AI 原生学校）。核心愿景：
- **Teacher + AI Symbiosis** -- 教师与 AI 共生模式
- AI 可以大规模提供个性化、按需辅导
- 将高质量教育带给任何人

---

## 八、GitHub 组织方式与代码即文档理念

### 8.1 仓库组织特点

卡帕西的 GitHub 仓库（[github.com/karpathy](https://github.com/karpathy)，63 个公开仓库，163K+ Stars）展现了一致的组织模式：

- **扁平目录层级** -- 最少嵌套
- **单文件实现优先** -- 可能的话尽量使用单文件
- **广泛的行内注释** -- README 同时充当教程
- **不使用抽象层** -- 除非必要，优先可读性而非可扩展性
- **每个仓库有清晰的教育目的** -- 代码本身即为课程

### 8.2 "代码即文档" 的具体体现

| 仓库 | 体现方式 |
|------|---------|
| **nanoGPT** | 剥离到最简的 GPT 训练代码，自文档化 |
| **minGPT** | 最小、干净、可解释的 GPT 实现 |
| **llm.c** | 纯 C/CUDA，零框架臃肿——代码即为解释 |
| **micrograd** | ~150 行揭示反向传播本质 |
| **microgpt** | ~200 行揭示 GPT 本质 |
| **rendergit** | 将仓库扁平化为单页阅读的工具 |
| **nn-zero-to-hero** | 视频+代码教程，代码结构即为课程大纲 |

### 8.3 仓库命名哲学

- `nano*` -- 比最小还小，极致精简
- `min*` -- 最小化实现
- `micro*` -- 微观级别，揭示本质

---

## 九、总结：卡帕西知识管理方法的核心原则

### 9.1 十大核心原则

1. **极简捕获，最小摩擦** -- 单一笔记文件，不搞分类体系
2. **知识编译而非知识检索** -- 先编译成结构化 wiki，再查询
3. **AI 做苦力，人做思考** -- 人负责策划来源、引导分析、问好问题；LLM 做其他一切
4. **从零构建等于真正理解** -- micrograd、nanoGPT、llm.c 都是这一哲学的体现
5. **渐进式复杂化** -- 从简单开始，验证每一步，然后慢慢增加复杂度
6. **耐心和关注细节** -- 深度学习成功最相关的品质
7. **项目驱动学习** -- 边做边学，不要试图先学完一切
8. **教学即学习** -- 通过教授来巩固理解（费曼技巧）
9. **代码即文档** -- 好的代码本身就是最好的文档
10. **开放共享** -- 所有教学内容和项目全部开源

### 9.2 知识管理的三个层次

```
第一层：快速捕获（Append-and-Review）
    └── 单一 Apple Notes 文件，随手记录，定期回顾
    
第二层：结构化知识库（LLM Wiki）
    └── Raw -> Wiki -> Schema 三层架构
    └── LLM 自动编译、交叉引用、矛盾检测
    └── Obsidian 浏览 + Claude Code 维护
    
第三层：自动化研究（Autoresearch + LLM Council）
    └── AI 代理自主循环实验
    └── 多模型交叉验证
```

### 9.3 对个人知识库构建的启示

1. **不要过度设计笔记系统** -- 卡帕西本人只用一个文本文件做日常笔记
2. **让知识积累产生复利** -- Wiki 的交叉引用和综合分析会随时间增值
3. **利用 AI 解决知识维护的痛点** -- 人放弃 wiki 的原因通常是维护负担太大
4. **从零构建是最好的学习方式** -- 不要只依赖框架和抽象
5. **保持开放和分享** -- 教学是巩固理解的最佳方式

---

## 参考资源链接

### 原始来源
- [karpathy.ai](https://karpathy.ai/) -- 个人网站
- [karpathy.github.io](http://karpathy.github.io/) -- 技术博客
- [karpathy.bearblog.dev](https://karpathy.bearblog.dev/blog/) -- 随笔博客
- [github.com/karpathy](https://github.com/karpathy) -- GitHub 主页
- [x.com/karpathy](https://x.com/karpathy) -- X/Twitter

### 核心文章和项目
- [The Append-and-Review Note](https://karpathy.bearblog.dev/the-append-and-review-note/)
- [LLM Wiki Gist](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f)
- [A Recipe for Training Neural Networks](http://karpathy.github.io/2019/04/25/recipe/)
- [Digital Hygiene](https://karpathy.bearblog.dev/digital-hygiene/)
- [microgpt](http://karpathy.github.io/2026/02/12/microgpt/)
- [Neural Networks: Zero to Hero](https://karpathy.ai/zero-to-hero.html)

### 开源项目
- [nanoGPT](https://github.com/karpathy/nanogpt)
- [minGPT](https://github.com/karpathy/minGPT)
- [llm.c](https://github.com/karpathy/llm.c)
- [micrograd](https://github.com/karpathy/micrograd)
- [minbpe](https://github.com/karpathy/minbpe)
- [rendergit](https://github.com/karpathy/rendergit)
- [autoresearch](https://github.com/karpathy/autoresearch)
- [llm-council](https://github.com/karpathy/llm-council)
- [nn-zero-to-hero](https://github.com/karpathy/nn-zero-to-hero)

### 教育项目
- [Eureka Labs](https://eurekalabs.ai/)
- [CS231n (2016)](https://cs231n.stanford.edu/2016/)
- [CS231n Course Notes](https://cs231n.github.io/)

### 社区分析
- [Karpathy's LLM Wiki: Complete Guide (Antigravity Codes)](https://antigravity.codes/blog/karpathy-llm-wiki-idea-file)
- [Karpathy LLM Knowledge Base Guide (MindStudio)](https://www.mindstudio.ai/blog/andrej-karpathy-llm-wiki-knowledge-base-claude-code/)
- [Karpathy's Personal Knowledge Base: What's Actually New (Medium)](https://medium.com/jin-system-architect/karpathys-personal-knowledge-base-what-s-actually-new-here-22b2b3891060)
- [How to Build Karpathy's LLM Wiki (Starmorph)](https://blog.starmorph.com/blog/karpathy-llm-wiki-knowledge-base-guide)
- [The Karpathy Loop (Fortune)](https://fortune.com/2026/03/17/andrej-karpathy-loop-autonomous-ai-agents-future/)
- [Autoresearch Went Viral (Substack)](https://alexeyondata.substack.com/p/karpathys-autoresearch-went-viral)
