/**
 * Mock 数据 —— 开发阶段使用
 * 包含项目树、待办事项、阶段信息等测试数据
 */

import type { StageInfo, TodoItem } from '@/types';

// ========================================
// 项目树数据 —— 3个专利案件
// ========================================
export interface ProjectFile {
  id: string;
  name: string;
  type: 'file';
  ext: string;
}

export interface ProjectNode {
  id: string;
  name: string;
  type: 'project';
  children: ProjectFile[];
}

export const projectTree: ProjectNode[] = [
  {
    id: 'case-a',
    name: '案件A',
    type: 'project',
    children: [
      { id: 'case-a-1', name: 'claims.md', type: 'file', ext: '.md' },
      { id: 'case-a-2', name: 'OA.pdf', type: 'file', ext: '.pdf' },
      { id: 'case-a-3', name: '对比分析.md', type: 'file', ext: '.md' },
      { id: 'case-a-4', name: '检索报告.md', type: 'file', ext: '.md' },
      { id: 'case-a-5', name: '附图说明.txt', type: 'file', ext: '.txt' },
    ],
  },
  {
    id: 'case-b',
    name: '案件B',
    type: 'project',
    children: [
      { id: 'case-b-1', name: 'claims.md', type: 'file', ext: '.md' },
      { id: 'case-b-2', name: '审查意见.pdf', type: 'file', ext: '.pdf' },
      { id: 'case-b-3', name: '答复草案.md', type: 'file', ext: '.md' },
    ],
  },
  {
    id: 'case-c',
    name: '案件C',
    type: 'project',
    children: [
      { id: 'case-c-1', name: '权利要求书.md', type: 'file', ext: '.md' },
      { id: 'case-c-2', name: '说明书.pdf', type: 'file', ext: '.pdf' },
      { id: 'case-c-3', name: '对比表格.xlsx', type: 'file', ext: '.xlsx' },
      { id: 'case-c-4', name: '引用文献清单.md', type: 'file', ext: '.md' },
    ],
  },
];

// ========================================
// 文件源数据 —— FilesTab
// ========================================
export interface SourceNode {
  id: string;
  name: string;
  type: 'source';
}

export const fileSources: SourceNode[] = [
  { id: 'src-1', name: 'PatentLens 专利库', type: 'source' },
  { id: 'src-2', name: 'Google Patents', type: 'source' },
  { id: 'src-3', name: 'CNIPA 中国专利', type: 'source' },
  { id: 'src-4', name: '本地文档库', type: 'source' },
];

// ========================================
// 待办事项 —— 5条
// ========================================
export const initialTodos: TodoItem[] = [
  {
    id: 'todo-1',
    text: '分析案件A的权利要求1-3的新颖性',
    completed: false,
    createdAt: Date.now() - 86400000,
  },
  {
    id: 'todo-2',
    text: '对比D1与案件B的技术特征差异',
    completed: true,
    createdAt: Date.now() - 172800000,
  },
  {
    id: 'todo-3',
    text: '起草案件C的审查意见答复书',
    completed: false,
    createdAt: Date.now() - 43200000,
  },
  {
    id: 'todo-4',
    text: '检索相关现有技术文献（至少10件）',
    completed: false,
    createdAt: Date.now() - 21600000,
  },
  {
    id: 'todo-5',
    text: '核对案件A的附图说明与权利要求一致性',
    completed: false,
    createdAt: Date.now() - 3600000,
  },
];

// ========================================
// 阶段信息 —— 4个阶段
// ========================================
export const initialStages: StageInfo[] = [
  { id: 'search', label: '检索', status: 'completed' },
  { id: 'compare', label: '对比', status: 'active' },
  { id: 'review', label: '审查', status: 'pending' },
  { id: 'draft', label: '起草', status: 'pending' },
];

// ========================================
// Markdown 预览用的 Mock 文件内容
// ========================================
export const mockFileContents: Record<string, string> = {
  'claims.md': `# 权利要求书

## 独立权利要求

**1. 一种基于大语言模型的专利智能分析方法，其特征在于，包括以下步骤：**

- S1. 接收用户输入的技术描述文本；
- S2. 通过大语言模型对所述技术描述进行语义理解；
- S3. 基于理解结果检索相关现有技术；
- S4. 生成专利性分析报告。

## 从属权利要求

**2. 根据权利要求1所述的方法，其特征在于**，步骤S2中所述语义理解包括：技术领域识别、关键技术特征提取、创新点定位。

**3. 根据权利要求1所述的方法，其特征在于**，步骤S3中所述检索包括：向量化语义检索、关键词布尔检索、分类号层级检索的组合策略。

---

*文件编号：CN20241001*
*最后更新：2024年12月*
`,
  '对比分析.md': `# 对比分析报告

## 技术方案对比

| 技术特征 | 本申请 | D1 (CN2023xxxx) | D2 (US2023xxxx) |
|----------|--------|-----------------|-----------------|
| 语义理解模型 | LLM + 专利领域微调 | 通用BERT | 关键词匹配 |
| 检索策略 | 组合检索 | 单一关键词 | 分类号检索 |
| 报告生成 | 自动结构化 | 人工整理 | 模板填充 |

## 新颖性结论

> 本申请的技术方案相对于D1和D2的组合具有**新颖性和创造性**。

关键区别在于：
1. 专利领域微调的语义理解
2. 多策略组合检索
3. 自动结构化报告生成
`,
  '检索报告.md': `# 检索报告

## 检索范围
- 数据库：PatentLens, Google Patents, CNIPA
- 时间范围：2019-2024
- 关键词：LLM, patent analysis, semantic search

## 检索结果
共检索到 **12件** 高度相关专利：

1. CN2023xxxx - 基于BERT的专利分类方法
2. US2023xxxx - 智能专利检索系统
3. EP2022xxxx - 语义专利分析平台
...

## 结论
现有技术中未公开与本申请完全相同的技术方案。
`,
};

// ========================================
// 文件扩展名颜色映射
// ========================================
export const fileExtColors: Record<string, string> = {
  '.md': '#5B8DEF',
  '.pdf': '#E8575A',
  '.docx': '#2B6CB0',
  '.doc': '#2B6CB0',
  '.txt': '#9CA3AF',
  '.xlsx': '#38A169',
  '.csv': '#38A169',
  '.png': '#9F7AEA',
  '.jpg': '#9F7AEA',
  '.jpeg': '#9F7AEA',
  default: '#9CA3AF',
};

/**
 * 根据扩展名获取文件图标颜色
 */
export function getFileExtColor(ext: string): string {
  return fileExtColors[ext] || fileExtColors.default;
}
