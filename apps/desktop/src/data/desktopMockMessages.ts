/**
 * Mock data for testing Agent Panel components
 * 包含线程数据、消息数据（各种类型）
 */
import type { Thread, Message } from '@/types';

// ========================================
// Mock Threads (5 threads)
// ========================================
export const mockThreads: Thread[] = [
  {
    id: 'thread-1',
    title: '专利检索会话',
    preview: '已检索到12件相关专利，涉及大语言模型...',
    timestamp: Date.now() - 3600000 * 2, // 2h ago
    status: 'active',
  },
  {
    id: 'thread-2',
    title: '技术对比分析',
    preview: '对比结果显示差异在权利要求1-3...',
    timestamp: Date.now() - 3600000 * 5, // 5h ago
    status: 'active',
  },
  {
    id: 'thread-3',
    title: '权利要求审查',
    preview: '审查报告已生成，建议修改从属权利要求...',
    timestamp: Date.now() - 86400000, // 1d ago
    status: 'active',
  },
  {
    id: 'thread-4',
    title: '专利申请起草',
    preview: '初稿已完成，包含说明书摘要和权利要求书...',
    timestamp: Date.now() - 86400000 * 3, // 3d ago
    status: 'archived',
  },
  {
    id: 'thread-5',
    title: '现有技术分析',
    preview: '分析了15件对比文件，准备创造性评估...',
    timestamp: Date.now() - 86400000 * 7, // 7d ago
    status: 'archived',
  },
];

// ========================================
// Mock Messages for thread-1 (10+ messages)
// ========================================
export const mockMessagesThread1: Message[] = [
  // System greeting
  {
    id: 'msg-1',
    role: 'agent',
    content: '您好！我是云熙专利智能助手。我可以帮助您进行专利检索、对比分析、权利要求审查和专利申请文件起草。请告诉我您需要处理的技术领域或专利号。',
    timestamp: Date.now() - 3600000 * 2 - 600000,
    status: 'complete',
  },
  // User asks for patent search
  {
    id: 'msg-2',
    role: 'user',
    content: '帮我检索一下与"大语言模型在专利分析中的应用"相关的专利。',
    timestamp: Date.now() - 3600000 * 2,
    status: 'complete',
  },
  // Agent responds with search results (streaming simulation)
  {
    id: 'msg-3',
    role: 'agent',
    content: `好的，正在为您检索相关专利。我已经找到了 12 件高度相关的专利：

1. **CN202310123456** - "一种基于深度学习的专利文本分类方法"
2. **CN202210987654** - "智能专利分析系统与方法"
3. **CN202311045678** - "基于大语言模型的专利摘要生成方法"
4. **CN202310567890** - "专利权利要求语义分析方法及系统"

需要我为您生成详细的检索报告吗？`,
    timestamp: Date.now() - 3600000 * 2 + 120000,
    status: 'complete',
    reasoning: '用户请求检索与"大语言模型在专利分析中的应用"相关的专利。我需要：\n1. 分析用户的技术领域关键词\n2. 构建检索策略：LLM、patent analysis、NLP、text classification\n3. 在专利数据库中执行检索\n4. 筛选相关性最高的12件专利\n5. 按技术相关度排序返回结果',
  },
  // User asks for detailed report
  {
    id: 'msg-4',
    role: 'user',
    content: '生成详细报告，特别关注技术方案的创新性分析。',
    timestamp: Date.now() - 3600000 * 1.5,
    status: 'complete',
  },
  // Agent responds with detailed report (includes tool calls)
  {
    id: 'msg-5',
    role: 'agent',
    content: `正在为您生成详细检索报告，包含创新性分析...`,
    timestamp: Date.now() - 3600000,
    status: 'complete',
    toolCalls: [
      {
        id: 'tool-1',
        name: 'search_patents',
        status: 'success',
        output: JSON.stringify({
          query: '大语言模型 专利分析',
          results_count: 12,
          top_results: ['CN202310123456', 'CN202210987654'],
        }),
      },
      {
        id: 'tool-2',
        name: 'analyze_innovation',
        status: 'success',
        output: JSON.stringify({
          patent: 'CN202310123456',
          innovation_score: 0.82,
          key_features: ['深度学习模型', '文本分类', '专利领域'],
        }),
      },
      {
        id: 'tool-3',
        name: 'fetch_patent_details',
        status: 'running',
        output: JSON.stringify({ patent_id: 'CN202311045678' }),
      },
    ],
  },
  // User asks about specific patent
  {
    id: 'msg-6',
    role: 'user',
    content: '帮我分析一下 CN202310123456 这件专利的权利要求布局。',
    timestamp: Date.now() - 3000000,
    status: 'complete',
  },
  // Agent with reasoning block
  {
    id: 'msg-7',
    role: 'agent',
    content: `**CN202310123456** 的权利要求布局分析如下：

**独立权利要求（1项）：**
- 权利要求1：方法权利要求，保护一种基于深度学习的专利文本分类方法

**从属权利要求（5项）：**
- 权利要求2-3：特征提取步骤的细化
- 权利要求4-5：模型训练方法的限定
- 权利要求6：分类结果的应用场景

**布局评价：**
该专利采用了单独立权利要求 + 多从属的紧凑布局，保护范围适中。建议在权利要求1中增加具体的技术特征以提升授权前景。`,
    timestamp: Date.now() - 2400000,
    status: 'complete',
    reasoning: '用户请求分析 CN202310123456 的权利要求布局。我需要：\n1. 从数据库获取该专利的完整权利要求书\n2. 解析权利要求的层级结构\n3. 识别独立权利要求和从属权利要求\n4. 分析保护范围和布局策略\n5. 给出专业的布局评价和改进建议',
  },
  // User asks for code example
  {
    id: 'msg-8',
    role: 'user',
    content: '能给我一个专利文本预处理的代码示例吗？',
    timestamp: Date.now() - 1800000,
    status: 'complete',
  },
  // Agent with code block
  {
    id: 'msg-9',
    role: 'agent',
    content: `以下是一个专利文本预处理的 Python 代码示例：

\`\`\`python
import re
import jieba
from typing import List, Dict

class PatentTextPreprocessor:
    def __init__(self):
        self.stopwords = set(['所述', '其特征在于', '根据权利要求'])
    
     def clean_text(self, text: str) -> str:
        """清洗专利文本"""
        # 去除多余空格和换行
        text = re.sub(r'\s+', ' ', text)
        # 去除专利号引用
        text = re.sub(r'CN\d{13}', '[PATENT]', text)
        return text.strip()
    
    def segment(self, text: str) -> List[str]:
        """中文分词"""
        words = jieba.lcut(text)
        return [w for w in words if w not in self.stopwords and len(w) > 1]
    
    def extract_tech_terms(self, text: str) -> List[str]:
        """提取技术术语"""
        # 使用正则匹配专业术语
        pattern = r'[\u4e00-\u9fff]+(?:器|机|系统|方法|装置|模块)'
        return re.findall(pattern, text)

# 使用示例
preprocessor = PatentTextPreprocessor()
text = "本发明公开了一种基于深度学习的专利文本分类方法..."
cleaned = preprocessor.clean_text(text)
words = preprocessor.segment(cleaned)
print(f"分词结果: {words}")
\`\`\`

这个示例包含了文本清洗、中文分词和技术术语提取三个核心步骤。`,
    timestamp: Date.now() - 1200000,
    status: 'complete',
  },
  // User final message
  {
    id: 'msg-10',
    role: 'user',
    content: '谢谢，这些信息很有帮助！',
    timestamp: Date.now() - 600000,
    status: 'complete',
  },
  // Agent final response (streaming)
  {
    id: 'msg-11',
    role: 'agent',
    content: '不客气！如果您还有其他关于专利检索、分析或起草的问题，随时告诉我。祝您工作顺利！',
    timestamp: Date.now() - 300000,
    status: 'streaming',
  },
];

// ========================================
// Mock Messages for thread-2 (5 messages)
// ========================================
export const mockMessagesThread2: Message[] = [
  {
    id: 'msg-t2-1',
    role: 'agent',
    content: '技术对比分析会话已启动。请上传您需要对比的两件专利文件。',
    timestamp: Date.now() - 3600000 * 5 - 300000,
    status: 'complete',
  },
  {
    id: 'msg-t2-2',
    role: 'user',
    content: '对比 CN202310123456 和 CN202311045678 这两件专利的技术方案。',
    timestamp: Date.now() - 3600000 * 5,
    status: 'complete',
  },
  {
    id: 'msg-t2-3',
    role: 'agent',
    content: '正在加载两件专利的详细信息进行对比分析...',
    timestamp: Date.now() - 3600000 * 4,
    status: 'complete',
    toolCalls: [
      {
        id: 'tool-t2-1',
        name: 'fetch_patent',
        status: 'success',
        output: JSON.stringify({ patent_id: 'CN202310123456', title: '基于深度学习的专利文本分类方法' }),
      },
      {
        id: 'tool-t2-2',
        name: 'fetch_patent',
        status: 'success',
        output: JSON.stringify({ patent_id: 'CN202311045678', title: '基于大语言模型的专利摘要生成方法' }),
      },
      {
        id: 'tool-t2-3',
        name: 'compare_patents',
        status: 'success',
        output: JSON.stringify({
          similarity: 0.45,
          common_tech: ['深度学习', 'NLP', '专利文本'],
          differences: ['分类 vs 生成', '模型架构不同'],
        }),
      },
    ],
  },
  {
    id: 'msg-t2-4',
    role: 'user',
    content: '详细说明一下技术差异点。',
    timestamp: Date.now() - 3600000 * 3,
    status: 'complete',
  },
  {
    id: 'msg-t2-5',
    role: 'agent',
    content: `两件专利的核心技术差异分析：

**1. 技术目标差异**
- CN202310123456：聚焦于"分类"——将专利文本归类到特定技术领域
- CN202311045678：聚焦于"生成"——自动生成专利摘要文本

**2. 模型架构差异**
- CN202310123456：使用 BERT + 全连接层的分类架构
- CN202311045678：使用 GPT 架构的生成式模型

**3. 输入输出差异**
- 前者输入原始专利文本，输出类别标签
- 后者输入专利全文，输出结构化摘要

**4. 技术效果差异**
- 分类准确率：前者达到 92.3%
- 摘要 BLEU 分数：后者达到 0.41

两件专利虽然都应用于专利文本处理，但解决的技术问题完全不同，不构成抵触申请。`,
    timestamp: Date.now() - 3600000 * 2,
    status: 'complete',
    reasoning: '需要详细对比两件专利的技术差异。从4个维度分析：技术目标、模型架构、输入输出、技术效果。最终结论：两件专利不构成抵触申请。',
  },
];

// ========================================
// Combined messages map
// ========================================
export const mockMessagesMap: Record<string, Message[]> = {
  'thread-1': mockMessagesThread1,
  'thread-2': mockMessagesThread2,
  'thread-3': [
    {
      id: 'msg-t3-1',
      role: 'agent',
      content: '权利要求审查会话已启动。请上传您需要审查的专利申请文件。',
      timestamp: Date.now() - 86400000,
      status: 'complete',
    },
    {
      id: 'msg-t3-2',
      role: 'user',
      content: '审查这件专利的权利要求是否满足创造性要求。',
      timestamp: Date.now() - 80000000,
      status: 'complete',
    },
    {
      id: 'msg-t3-3',
      role: 'agent',
      content: '审查报告已生成。主要发现：独立权利要求1缺乏创造性，建议补充技术特征。从属权利要求3-5的限定具有创造性。',
      timestamp: Date.now() - 75000000,
      status: 'complete',
      toolCalls: [
        {
          id: 'tool-t3-1',
          name: 'review_creativity',
          status: 'success',
          output: JSON.stringify({ creativity_score: 0.62, issues: ['特征不够具体'] }),
        },
      ],
    },
  ],
  'thread-4': [
    {
      id: 'msg-t4-1',
      role: 'agent',
      content: '专利申请起草会话已启动。请描述您的发明技术方案。',
      timestamp: Date.now() - 86400000 * 3,
      status: 'complete',
    },
    {
      id: 'msg-t4-2',
      role: 'user',
      content: '起草一份关于"基于区块链的专利存证系统"的专利申请。',
      timestamp: Date.now() - 86400000 * 2,
      status: 'complete',
    },
  ],
  'thread-5': [
    {
      id: 'msg-t5-1',
      role: 'agent',
      content: '现有技术分析会话已启动。请提供您的技术方案描述。',
      timestamp: Date.now() - 86400000 * 7,
      status: 'complete',
    },
  ],
};
