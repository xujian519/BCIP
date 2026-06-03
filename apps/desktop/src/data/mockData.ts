export interface PatentCase {
  id: string;
  name: string;
  number: string;
  status: 'draft' | 'published' | 'examination' | 'rejected';
  children: {
    id: string;
    name: string;
    type: 'claims' | 'description' | 'drawings' | 'search' | 'drafts';
  }[];
}

export interface SearchResult {
  id: string;
  title: string;
  number: string;
  applicant: string;
  date: string;
  status: 'draft' | 'published' | 'examination' | 'rejected';
  abstract: string;
  relevance: number;
}

export interface Session {
  id: string;
  title: string;
  timestamp: string;
}

export const patentCases: PatentCase[] = [
  {
    id: 'case-1',
    name: '智能电池管理系统',
    number: 'CN202410123456.X',
    status: 'examination',
    children: [
      { id: 'c1-1', name: '权利要求书', type: 'claims' },
      { id: 'c1-2', name: '说明书', type: 'description' },
      { id: 'c1-3', name: '附图', type: 'drawings' },
      { id: 'c1-4', name: '检索结果', type: 'search' },
      { id: 'c1-5', name: '草稿', type: 'drafts' },
    ],
  },
  {
    id: 'case-2',
    name: 'Adaptive Neural Network Controller',
    number: 'US17/123,456',
    status: 'published',
    children: [
      { id: 'c2-1', name: 'Claims', type: 'claims' },
      { id: 'c2-2', name: 'Description', type: 'description' },
      { id: 'c2-3', name: 'Drawings', type: 'drawings' },
      { id: 'c2-4', name: 'Search Results', type: 'search' },
    ],
  },
  {
    id: 'case-3',
    name: 'Blockchain Data Verification',
    number: 'PCT/CN2024/123456',
    status: 'draft',
    children: [
      { id: 'c3-1', name: '权利要求书', type: 'claims' },
      { id: 'c3-2', name: '说明书', type: 'description' },
      { id: 'c3-3', name: '检索结果', type: 'search' },
    ],
  },
];

export const sessions: Session[] = [
  { id: 's1', title: '智能电池管理系统分析', timestamp: '10:23' },
  { id: 's2', title: '权利要求书撰写辅助', timestamp: '昨天' },
  { id: 's3', title: '专利检索：神经网络', timestamp: '昨天' },
  { id: 's4', title: '对比文件分析', timestamp: '周一' },
  { id: 's5', title: '审查意见回复', timestamp: '周一' },
];

export const sampleClaims = `[0001]  1. 一种智能电池管理系统，其特征在于，包括：
[0002]      电池状态监测模块，用于实时采集电池组的电压、电流和温度数据；
[0003]      数据处理单元，耦合至所述电池状态监测模块，用于根据采集的
[0004]      数据计算电池的健康状态参数；
[0005]      以及控制模块，耦合至所述数据处理单元，用于根据所述健康状态
[0006]      参数调整电池的充放电策略。
[0007]  
[0008]  2. 根据权利要求1所述的系统，其特征在于，所述电池状态监测模块包括：
[0009]      电压传感器阵列，配置为以预定采样频率采集每个电池单元的电压值；
[0010]      电流传感器，配置为测量电池组的总电流；
[0011]      温度传感器网络，分布式布置于电池组的关键热节点。
[0012]  
[0013]  3. 根据权利要求1或2所述的系统，其特征在于，所述健康状态参数包括：
[0014]      荷电状态(SOC)、健康状态(SOH)、以及功率状态(SOP)。
[0015]  
[0016]  4. 根据权利要求1所述的系统，其特征在于，所述控制模块进一步包括：
[0017]      预测性维护单元，基于历史数据和机器学习算法预测电池退化趋势。
[0018]  
[0019]  5. 根据权利要求4所述的系统，其特征在于，所述预测性维护单元采用
[0020]      长短期记忆网络(LSTM)模型，输入为时间序列的电池运行数据，输出为
[0021]      未来预定时间段内的容量衰减预测值。`;

export interface DiffLine {
  type: 'add' | 'del' | 'unchanged';
  lineNum: number;
  content: string;
}

export const diffComparison: { original: DiffLine[]; modified: DiffLine[] } = {
  original: [
    { type: 'unchanged', lineNum: 1, content: '1. 一种电池管理系统，包括：' },
    { type: 'del', lineNum: 2, content: '    监测模块，用于采集电池数据；' },
    { type: 'del', lineNum: 3, content: '    控制模块，用于控制电池充放电。' },
    { type: 'unchanged', lineNum: 4, content: '' },
    { type: 'unchanged', lineNum: 5, content: '2. 根据权利要求1所述的系统，其特征在于：' },
    { type: 'del', lineNum: 6, content: '    所述监测模块包括电压传感器和温度传感器。' },
  ],
  modified: [
    { type: 'unchanged', lineNum: 1, content: '1. 一种电池管理系统，包括：' },
    { type: 'add', lineNum: 2, content: '    电池状态监测模块，用于实时采集电池组的电压、电流和温度数据；' },
    { type: 'add', lineNum: 3, content: '    数据处理单元，耦合至所述监测模块，用于计算电池健康状态参数；' },
    { type: 'add', lineNum: 4, content: '    以及控制模块，耦合至所述数据处理单元，用于调整充放电策略。' },
    { type: 'unchanged', lineNum: 5, content: '' },
    { type: 'unchanged', lineNum: 6, content: '2. 根据权利要求1所述的系统，其特征在于：' },
    { type: 'add', lineNum: 7, content: '    所述电池状态监测模块包括电压传感器阵列、电流传感器和温度传感器网络。' },
  ],
};

export const searchResults: SearchResult[] = [
  {
    id: 'p1',
    title: '智能电池管理系统及方法',
    number: 'CN115629104A',
    applicant: '宁德时代新能源科技',
    date: '2023-01-18',
    status: 'published',
    abstract: '本发明公开了一种智能电池管理系统，包括电池状态监测模块、数据处理单元以及控制模块，能够实现对电池组的实时监控和智能管理。',
    relevance: 0.96,
  },
  {
    id: 'p2',
    title: 'Battery Management System with Predictive Analytics',
    number: 'US2023/0087654A1',
    applicant: 'Tesla, Inc.',
    date: '2023-03-22',
    status: 'examination',
    abstract: 'A battery management system employing machine learning algorithms to predict battery degradation and optimize charging strategies.',
    relevance: 0.88,
  },
  {
    id: 'p3',
    title: '基于深度学习的电池健康状态估计方法',
    number: 'CN116234567A',
    applicant: '比亚迪股份有限公司',
    date: '2023-06-15',
    status: 'published',
    abstract: '本发明涉及一种基于长短期记忆网络的电池健康状态估计方法，通过采集电池运行数据训练神经网络模型，实现对SOH的准确预测。',
    relevance: 0.82,
  },
  {
    id: 'p4',
    title: '分布式电池组温度监控装置',
    number: 'CN202320987654.X',
    applicant: '中航锂电科技有限公司',
    date: '2023-08-30',
    status: 'examination',
    abstract: '一种分布式温度传感器网络布置方案，用于大型电池组的精准温度监测和过热预警。',
    relevance: 0.71,
  },
  {
    id: 'p5',
    title: 'Battery Thermal Management and Safety Control',
    number: 'EP3987654A1',
    applicant: 'Samsung SDI Co., Ltd.',
    date: '2023-05-10',
    status: 'published',
    abstract: 'A thermal management system for lithium-ion batteries featuring distributed sensor networks and predictive safety controls.',
    relevance: 0.65,
  },
];

export const reviewData = {
  objections: [
    {
      id: 'obj-1',
      type: 'novelty' as const,
      claim: '权利要求1',
      citation: 'CN115629104A',
      content: '对比文件1公开了权利要求1的全部技术特征，权利要求1相对于对比文件1不具备新颖性。',
    },
    {
      id: 'obj-2',
      type: 'inventive' as const,
      claim: '权利要求2',
      citation: 'CN115629104A + US2023/0087654A1',
      content: '权利要求2的附加特征已被对比文件2公开，本领域技术人员容易想到将其与对比文件1结合，因此权利要求2不具备创造性。',
    },
    {
      id: 'obj-3',
      type: 'support' as const,
      claim: '权利要求4',
      citation: '',
      content: '权利要求4中限定的"预测性维护单元"在说明书中没有充分的技术效果支持，不符合专利法第26条第4款的规定。',
    },
  ],
  responses: [
    {
      id: 'resp-1',
      objectionId: 'obj-1',
      content: '申请人不同意审查意见。权利要求1中的"数据处理单元"采用特定的健康状态参数计算方法，该具体算法流程在对比文件1中并未公开。',
    },
    {
      id: 'resp-2',
      objectionId: 'obj-2',
      content: '申请人认为权利要求2的传感器布置方案产生了预料不到的技术效果，请求在答复时补充实验数据予以证明。',
    },
  ],
};

export type WorkflowStage = 'search' | 'compare' | 'review' | 'draft';

export const stageLabels: Record<WorkflowStage, string> = {
  search: '检索',
  compare: '对比',
  review: '审查',
  draft: '起草',
};
