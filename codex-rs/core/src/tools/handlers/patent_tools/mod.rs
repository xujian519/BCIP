use crate::function_tool::FunctionCallError;
use crate::tools::context::FunctionToolOutput;
use crate::tools::context::ToolInvocation;
use crate::tools::context::ToolOutput;
use crate::tools::context::ToolPayload;
use crate::tools::registry::CoreToolRuntime;
use crate::tools::registry::ToolExecutor;
use codex_patent_tools::ToolHandler;
use codex_patent_tools::register_all_tools;
use codex_tools::ResponsesApiTool;
use codex_tools::ToolName;
use codex_tools::ToolSpec;
use serde_json::json;
use std::sync::Arc;

pub struct PatentToolHandler {
    name: String,
    spec: ToolSpec,
    handler: ToolHandler,
}

impl PatentToolHandler {
    fn new(name: String, handler: ToolHandler) -> Self {
        let spec = ToolSpec::Function(ResponsesApiTool {
            name: name.clone(),
            description: tool_description(&name),
            strict: false,
            defer_loading: None,
            parameters: serde_json::from_value(tool_parameters(&name)).unwrap_or_default(),
            output_schema: None,
        });
        Self {
            name,
            spec,
            handler,
        }
    }

    pub fn create_all_adapters() -> Vec<Arc<dyn CoreToolRuntime>> {
        let tools = register_all_tools();
        tools
            .into_iter()
            .map(|(name, handler)| Arc::new(Self::new(name, handler)) as Arc<dyn CoreToolRuntime>)
            .collect()
    }
}

#[async_trait::async_trait]
impl ToolExecutor<ToolInvocation> for PatentToolHandler {
    fn tool_name(&self) -> ToolName {
        ToolName::plain(&self.name)
    }

    fn spec(&self) -> ToolSpec {
        self.spec.clone()
    }

    async fn handle(
        &self,
        invocation: ToolInvocation,
    ) -> Result<Box<dyn ToolOutput>, FunctionCallError> {
        let args_str = match &invocation.payload {
            ToolPayload::Function { arguments } => arguments.clone(),
            _ => {
                return Err(FunctionCallError::RespondToModel(format!(
                    "unsupported payload for patent tool: {}",
                    self.name
                )));
            }
        };
        let args: serde_json::Value = serde_json::from_str(&args_str)
            .map_err(|e| FunctionCallError::RespondToModel(format!("invalid JSON args: {e}")))?;
        match (self.handler)(args).await {
            Ok(value) => Ok(Box::new(FunctionToolOutput::from_text(
                serde_json::to_string(&value).unwrap_or_else(|e| format!("{e}")),
                None,
            ))),
            Err(e) => Err(FunctionCallError::RespondToModel(e)),
        }
    }
}

impl CoreToolRuntime for PatentToolHandler {
    fn matches_kind(&self, _payload: &ToolPayload) -> bool {
        true
    }
}

fn tool_description(name: &str) -> String {
    match name {
        "PatentSearch" => "统一专利检索 — 基于现有技术定义（法第22条第2款），支持同义词扩展和跨源搜索".into(),
        "GooglePatentsFetch" => "Google Patents 专利检索 — 国际现有技术检索".into(),
        "SearchQueryBuilder" => "检索式构建 — 审查指南第七章：体现技术方案基本构思的检索要素".into(),
        "IterativeSearch" => "迭代检索 — 审查指南第七章：覆盖全部相关现有技术".into(),
        "PatentDownload" => "专利 PDF 下载 — 获取对比文件用于新颖性/创造性（法第22条）分析".into(),
        "ClaimGenerator" => "权利要求生成 — 法第26条第4款（清楚简要）、细则第19条（必要特征）、细则第22条（引用规范）".into(),
        "SpecificationDrafter" => "说明书撰写 — 法第26条第3款（充分公开）、细则第17条（五部分顺序）".into(),
        "AbstractDrafter" => "摘要撰写 — 细则第23条（≤300字）".into(),
        "OaParser" => "OA解析 — 审查指南第二部分第八章：类型检测+引证提取+驳回理由".into(),
        "OaStrategist" => "OA策略 — 基于法第22条第2/3款、第26条第4款、第33条的答复策略推荐".into(),
        "PatentResponder" => "答复生成 — 法第33条（修改不超范围）、法第22条（新颖性/创造性）争辩".into(),
        "StrategyArgumentGenerator" => "争辩生成 — 法第22条第2/3款、审查指南第二部分第三/四章".into(),
        "UnifiedQuality" => "综合质检 — 法第26条第3/4款、细则第17-23条、审查指南第二部分第二章".into(),
        "QualityChecker" => "规则质检 — 审查指南第二部分第二章2.2.6（禁止用语/模糊表述）".into(),
        "SubjectMatterChecker" => "保护客体检查(结构化) — 法第2条（三要素）、法第5条（违法排除）、法第25条（排除客体）。输入为发明名称+权利要求列表".into(),
        "UnityChecker" => "单一性检查(结构化) — 法第31条：属于一个总的发明构思。输入为权利要求列表+专利类型".into(),
        "SpecFormalityChecker" => "说明书形式检查 — 细则第17-19条（章节完整性）".into(),
        "LegalLanguageChecker" => "法言法语检查 — 禁止商业宣传/不确定用语/引权利要求语".into(),
        "FormatRules" => "CNIPA格式校验 — 细则第17-23条格式规范".into(),
        "ClaimParse" => "权利要求解析 — 法第26条第4款（清楚简要）、细则第19/22条（独立/从属结构）".into(),
        "ClaimCompare" => "权利要求对比 — 法第22条第2款（新颖性单独对比）".into(),
        "NoveltyAnalysis" => "新颖性分析 — 法第22条第2款：单独对比、四相同、上下位概念、惯用手段置换".into(),
        "InventivenessAnalysis" => "创造性分析 — 法第22条第3款：三步法（审查指南第二部分第四章）".into(),
        "InfringementAnalysis" => "侵权分析 — 法第59条（保护范围）、法第11条（侵权行为）、全面覆盖+等同+禁止反悔".into(),
        "InnovationEvaluator" => "创新性评估 — 法第22条第3款：六种发明类型差异化分析".into(),
        "SemanticCompare" => "语义对比 — 语义+结构+顺序相似度综合对比".into(),
        "TechTripleExtractor" => "问题-特征-效果三元组 — 法第2条技术方案三要素分析".into(),
        "FeatureExtractor" => "技术特征提取 — 最小技术单元法（侵权判定指南(2017)第8条）".into(),
        "LegalQA" => "专利法问答 — 涵盖法第2/5/9/11/22/24/25/26/29/31/33/41/45/59条".into(),
        "LegalKnowledgeSearch" => "法律检索 — 法律法规+审查指南+司法解释+案例".into(),
        "LegalBasisRefs" => "法条查询 — 专利法及实施细则条款原文".into(),
        "KnowledgeSearch" => "知识库搜索 — 法律法规/审查指南/案例/知识图谱跨库检索".into(),
        "FormatConverter" => "格式转换 — CNIPA标准格式（发明/实用新型/外观设计）".into(),
        "DocxTools" => "DOCX处理 — CNIPA专利申请文件格式".into(),
        "PdfTools" => "PDF处理 — 专利文档文本提取与解析".into(),
        "OcrBridge" => "OCR识别 — 专利附图/扫描件文字识别".into(),
        "MarkdownParser" => "Markdown解析 — CJK文本统计与结构分析".into(),
        "TemplateLibrary" => "模板库 — 审查意见答复/无效宣告/复审请求/专利申请标准模板".into(),
        "ClaimsStructure" => "权利要求结构 — 法第26条第4款（独立/从属层次分析）".into(),
        "WriterTool" => "通用写作 — 专利文书辅助撰写".into(),
        "ResponseTemplate" => "答复模板 — 审查指南第二部分第八章：答复意见书格式".into(),
        "ClaimOutputProcessor" => "权利要求格式化 — CNIPA标准权利要求书格式".into(),
        "SpecOutputProcessor" => "说明书格式化 — CNIPA标准说明书格式".into(),
        "PatentCompareTool" => "专利对比 — 法第22条第2款（逐特征对比分析）".into(),
        "TechUnit" => "最小技术单元 — 侵权判定指南(2017)第8条：特征分解".into(),
        "InventionUnderstanding" => "发明理解 — 技术交底书解析（法第26条第3款：充分公开的前置分析）".into(),
        "Researcher" => "技术调研 — 现有技术检索与分析".into(),
        "PatentInfringement" => "侵权风险 — 法第59条+第11条（全面覆盖+等同判断）".into(),
        "PatentManager" => "专利管理 — 全生命周期管理（申请→审查→授权→维持）".into(),
        "TemplateManager" => "模板管理 — 专利文书模板检索与选取".into(),
        "ProcessChart" => "流程图 — 专利申请/审查/无效流程图生成".into(),
        "TrademarkAnalysis" => "商标分析 — 商标显著性/近似性/类别分析".into(),
        "FormalCheck" => "形式审查 — 权利要求编号/引用有效性/说明书章节完整性检查".into(),
        "QualityAssess" => "质量评估 — 权利要求和说明书综合质量评分".into(),
        "SubjectMatterCheck" => "保护客体审查(纯文本) — 法第25条排除客体快速检查。输入为单条权利要求文本".into(),
        "UnityCheck" => "单一性审查(纯文本) — 法第31条：基于词汇重叠度快速判断。输入为权利要求文本列表".into(),
        "OaStrategy" => "OA策略推荐 — 基于定性规则引擎的审查意见答复策略".into(),
        "OaResponseTemplate" => "OA答复模板 — 新颖性/创造性/修改方案等答复模板生成".into(),
        "ActionReview" => "动作复核 — 比对工具输出与预期结果".into(),
        "LlmReflection" => "LLM自省 — 基于准则的输出质量检查".into(),
        "FaithfulnessEval" => "忠实度评估 — 检测输出是否忠实于源文本".into(),
        "SelfConsistencyEval" => "自洽性评估 — 多次输出的一致性检查".into(),
        "GEval" => "G-Eval — 基于量规的多维度评分".into(),
        "SynergyAnalysis" => "协同分析 — 技术特征之间的协同效应评估".into(),
        "HighCitationSearch" => "高被引检索 — 查找高被引后续专利".into(),
        "SuccessPredictor" => "成功预测 — 基于OA类型的答复成功概率预测".into(),
        "GraphQuery" => "图谱查询 — 知识图谱路径查询与遍历".into(),
        "GraphNeighbors" => "图谱邻居 — 查询节点的关联实体".into(),
        "LinkGraph" => "链接图谱 — 知识库文档间交叉引用关系分析".into(),
        "RefreshKnowledge" => "知识刷新 — 触发知识库增量更新".into(),
        "SearchEval" => "搜索评估 — 检索质量评估（精确率/召回率）".into(),

        // ── Council 域（2 个工具）──
        "CouncilDeliberate" => "多模型审议 — 三阶段 LLM Council：多模型并行独立分析→匿名化互评排名→Chairman终裁综合输出。用于需要多视角交叉验证的专利任务（创造性判断/审查意见策略/无效分析等）。触发方式：提供 task 任务描述，可选 models/chairman/criteria/api_base 参数。默认使用硅基流动模型 (DeepSeek-V3/Qwen2.5-72B/GLM-4)。".into(),
        "CouncilQualityGate" => "质量门控 — 多模型独立评审文档质量，计算通过率，低于阈值返回修改建议。支持权利要求书/说明书/审查意见答复三种文档类型。触发方式：提供 document(文档全文)和document_type(类型)，可选 threshold/api_base 参数。".into(),

        // ── Simulator 域（6 个工具）──
        "ExaminerSimulate" => "审查员模拟 — 模拟专利审查员对权利要求逐条提出驳回意见。输入审查意见文本+权利要求列表，输出模拟驳回结论和审查策略。".into(),
        "ExaminerRespond" => "审查员反驳模拟 — 模拟审查员对申请人答复的反驳。输入申请人论点和轮次，输出模拟反驳和弱点分析。".into(),
        "ResponseEvaluate" => "答复质量评估 — 评估OA答复文件的完整性/说服力/技术深度/逻辑一致性（0-100分），输出改进建议和预测结果。".into(),
        "RuleAnalysis" => "规则分析 — 基于定性规则引擎的新颖性/创造性/OA策略分析。输入分析类型和案情，输出规则匹配结果和置信度。".into(),
        "OaFeedbackRecord" => "OA反馈记录 — 记录审查意见答复结果（成功/失败/部分成功），收集反馈数据供策略学习。输入 oa_id/patent_id/feedback_type/outcome/quality_score。".into(),
        "OaPatternExtract" => "OA模式提取 — 从历史答复轨迹中提取可复用的工作流模式，基于成功率和最小支持度筛选。输出可复用模式列表。".into(),
        "ScenarioDispatch" => "场景编排调度 — 根据任务类型返回预设的DAG编排计划（含并行分组、依赖关系、HITL节点）。支持 oa_strategy/novelty_analysis/inventiveness_rejection/infringement_analysis/quality_review 五种场景。".into(),

        _ => format!("{name} - 专利工具"),
    }
}

fn tool_parameters(name: &str) -> serde_json::Value {
    let fallback =
        || json!({"type": "object", "properties": {}, "description": format!("{} 工具参数", name)});
    match name {
        "PatentSearch" => {
            json!({"type":"object","properties":{"query":{"type":"string"},"limit":{"type":"integer","default":10},"use_synonyms":{"type":"boolean","default":true}},"required":["query"]})
        }
        "GooglePatentsFetch" => {
            json!({"type":"object","properties":{"query":{"type":"string"},"limit":{"type":"integer","default":10},"patent_number":{"type":"string"}},"required":["query"]})
        }
        "SearchQueryBuilder" => {
            json!({"type":"object","properties":{"concept":{"type":"string"},"field":{"type":"string"}},"required":["concept"]})
        }
        "IterativeSearch" => {
            json!({"type":"object","properties":{"query":{"type":"string"},"rounds":{"type":"integer","default":3},"limit":{"type":"integer","default":10}},"required":["query"]})
        }
        "PatentDownload" => {
            json!({"type":"object","properties":{"patent_number":{"type":"string"}},"required":["patent_number"]})
        }
        "ClaimGenerator" => {
            json!({"type":"object","properties":{"invention_name":{"type":"string"},"essential_features":{"type":"array","items":{"type":"string"}},"optional_features":{"type":"array","items":{"type":"array","items":{"type":"string"}}}},"required":["invention_name","essential_features"]})
        }
        "SpecificationDrafter" => {
            json!({"type":"object","properties":{"title":{"type":"string"},"technical_field":{"type":"string"},"background":{"type":"string"},"invention_content":{"type":"string"},"embodiments":{"type":"string"}},"required":["title"]})
        }
        "AbstractDrafter" => {
            json!({"type":"object","properties":{"title":{"type":"string"},"technical_problem":{"type":"string"},"technical_solution":{"type":"string"},"technical_effect":{"type":"string"}},"required":["title"]})
        }
        "OaParser" => {
            json!({"type":"object","properties":{"content":{"type":"string"},"application_number":{"type":"string"},"patent_title":{"type":"string"}},"required":["content"]})
        }
        "OaStrategist" => {
            json!({"type":"object","properties":{"oa_type":{"type":"string"},"examiner_arguments":{"type":"string"},"affected_claims":{"type":"array","items":{"type":"integer"}}},"required":["oa_type","examiner_arguments"]})
        }
        "PatentResponder" => {
            json!({"type":"object","properties":{"oa_content":{"type":"string"},"strategy":{"type":"string"}},"required":["oa_content"]})
        }
        "UnifiedQuality" => {
            json!({"type":"object","properties":{"claims":{"type":"array","items":{"type":"object","properties":{"claim_type":{"type":"string"},"preamble":{"type":"string"},"elements":{"type":"array","items":{"type":"string"}}}}}},"required":["claims"]})
        }
        "SubjectMatterChecker" => {
            json!({"type":"object","properties":{"invention_title":{"type":"string"},"claims":{"type":"array","items":{"type":"string"}},"patent_type":{"type":"string","default":"invention"}},"required":["invention_title","claims"]})
        }
        "NoveltyAnalysis" => {
            json!({"type":"object","properties":{"invention_description":{"type":"string"},"prior_art_descriptions":{"type":"array","items":{"type":"string"}},"differences":{"type":"array","items":{"type":"string"}}},"required":["invention_description"]})
        }
        "InventivenessAnalysis" => {
            json!({"type":"object","properties":{"invention_description":{"type":"string"},"technical_effect":{"type":"string"},"performance_improvement":{"type":"number"},"obviousness":{"type":"boolean"}},"required":["invention_description"]})
        }
        "InfringementAnalysis" => {
            json!({"type":"object","properties":{"claim_text":{"type":"string"},"accused_product_description":{"type":"string"}},"required":["claim_text","accused_product_description"]})
        }
        "LegalQA" => {
            json!({"type":"object","properties":{"question":{"type":"string"},"domain":{"type":"string","default":"patent"}},"required":["question"]})
        }
        "LegalKnowledgeSearch" => {
            json!({"type":"object","properties":{"query":{"type":"string"},"limit":{"type":"integer","default":5},"category":{"type":"string"}},"required":["query"]})
        }
        "FormatConverter" => {
            json!({"type":"object","properties":{"content":{},"input_format":{"type":"string"},"output_format":{"type":"string"},"patent_office_format":{"type":"string","default":"CNIPA"}},"required":["content","input_format","output_format"]})
        }
        "DocxTools" => {
            json!({"type":"object","properties":{"markdown":{"type":"string"},"output_path":{"type":"string"}},"required":["markdown"]})
        }
        "TechTripleExtractor" => {
            json!({"type":"object","properties":{"text":{"type":"string"}},"required":["text"]})
        }
        "FeatureExtractor" => {
            json!({"type":"object","properties":{"text":{"type":"string"}},"required":["text"]})
        }
        "FormalCheck" => {
            json!({"type":"object","properties":{"claims":{"type":"array","items":{"type":"string"}},"specification_sections":{"type":"array","items":{"type":"string"}}},"required":["claims"]})
        }
        "QualityAssess" => {
            json!({"type":"object","properties":{"claims":{"type":"array","items":{"type":"string"}},"specification_word_count":{"type":"integer"}},"required":["claims","specification_word_count"]})
        }
        "SubjectMatterCheck" => {
            json!({"type":"object","properties":{"claim_text":{"type":"string"}},"required":["claim_text"]})
        }
        "UnityCheck" => {
            json!({"type":"object","properties":{"claims":{"type":"array","items":{"type":"string"}}},"required":["claims"]})
        }
        "OaStrategy" => {
            json!({"type":"object","properties":{"rejection_type":{"type":"string"},"differences":{"type":"array","items":{"type":"string"}},"technical_effects":{"type":"array","items":{"type":"string"}},"prior_art_different_field":{"type":"boolean"}},"required":["rejection_type"]})
        }
        "OaResponseTemplate" => {
            json!({"type":"object","properties":{"template_type":{"type":"string"},"arguments":{"type":"array","items":{"type":"string"}}},"required":["template_type"]})
        }
        "ActionReview" => {
            json!({"type":"object","properties":{"action":{"type":"string"},"expected":{"type":"string"},"actual":{"type":"string"}},"required":["action","expected","actual"]})
        }
        "LlmReflection" => {
            json!({"type":"object","properties":{"output":{"type":"string"},"criteria":{"type":"array","items":{"type":"string"}}},"required":["output"]})
        }
        "FaithfulnessEval" => {
            json!({"type":"object","properties":{"source":{"type":"string"},"output":{"type":"string"}},"required":["source","output"]})
        }
        "SelfConsistencyEval" => {
            json!({"type":"object","properties":{"results":{"type":"array","items":{"type":"string"}}},"required":["results"]})
        }
        "GEval" => {
            json!({"type":"object","properties":{"output":{"type":"string"},"rubric":{"type":"array","items":{"type":"object","properties":{"name":{"type":"string"},"weight":{"type":"number"}}}}},"required":["output","rubric"]})
        }
        "SynergyAnalysis" => {
            json!({"type":"object","properties":{"features":{"type":"array","items":{"type":"string"}},"description":{"type":"string"}},"required":["features","description"]})
        }
        "HighCitationSearch" => {
            json!({"type":"object","properties":{"patent_number":{"type":"string"},"limit":{"type":"integer"}},"required":["patent_number"]})
        }
        "SuccessPredictor" => {
            json!({"type":"object","properties":{"rejection_type":{"type":"string"},"has_differences":{"type":"boolean"},"has_technical_effect":{"type":"boolean"},"argument_count":{"type":"integer"}},"required":["rejection_type"]})
        }
        "TrademarkAnalysis" => {
            json!({"type":"object","properties":{"mark":{"type":"string"}},"required":["mark"]})
        }
        "GraphQuery" => {
            json!({"type":"object","properties":{"start_id":{"type":"string"},"max_depth":{"type":"integer","default":2},"relation_filter":{"type":"array","items":{"type":"string"}}},"required":["start_id"]})
        }
        "GraphNeighbors" => {
            json!({"type":"object","properties":{"node_id":{"type":"string"}},"required":["node_id"]})
        }
        "LinkGraph" => {
            json!({"type":"object","properties":{"keyword":{"type":"string"},"kb_root":{"type":"string"}},"required":[]})
        }
        "RefreshKnowledge" => {
            json!({"type":"object","properties":{},"required":[]})
        }
        "SearchEval" => {
            json!({"type":"object","properties":{"semantic":{"type":"boolean","default":false}},"required":[]})
        }
        "ClaimCompare" => {
            json!({"type":"object","properties":{"claim_a":{"type":"string","description":"待比较的权利要求A"},"claim_b":{"type":"string","description":"待比较的权利要求B"}},"required":["claim_a","claim_b"]})
        }
        "ClaimParse" => {
            json!({"type":"object","properties":{"claim_text":{"type":"string","description":"权利要求全文"},"claim_number":{"type":"integer","description":"权利要求编号"}},"required":["claim_text","claim_number"]})
        }
        "ClaimsStructure" => {
            json!({"type":"object","properties":{"claims_text":{"type":"string","description":"全部权利要求文本"}},"required":["claims_text"]})
        }
        "FormatRules" => {
            json!({"type":"object","properties":{"content":{"type":"string","description":"待检查的文档内容"},"doc_type":{"type":"string","description":"文档类型","default":"generic"}},"required":["content"]})
        }
        "InnovationEvaluator" => {
            json!({"type":"object","properties":{"invention_description":{"type":"string","description":"发明描述"},"technical_effect":{"type":"string","description":"技术效果"},"performance_improvement":{"type":"number","description":"性能提升比例"},"obviousness":{"type":"boolean","description":"是否显而易见"}},"required":["invention_description"]})
        }
        "InventionUnderstanding" => {
            json!({"type":"object","properties":{"invention_title":{"type":"string","description":"发明名称"},"technical_field":{"type":"string","description":"技术领域"},"technical_disclosure":{"type":"string","description":"技术交底书全文"}},"required":["invention_title","technical_field","technical_disclosure"]})
        }
        "KnowledgeSearch" => {
            json!({"type":"object","properties":{"query":{"type":"string","description":"搜索查询"},"limit":{"type":"integer","description":"返回数量上限","default":10},"semantic":{"type":"boolean","description":"是否启用语义搜索","default":false}},"required":["query"]})
        }
        "LegalBasisRefs" => {
            json!({"type":"object","properties":{"legal_issue":{"type":"string","description":"法律问题或关键词"},"patent_type":{"type":"string","description":"专利类型"}},"required":["legal_issue"]})
        }
        "LegalLanguageChecker" => {
            json!({"type":"object","properties":{"claims":{"type":"array","items":{"type":"string"},"description":"权利要求列表"},"check_level":{"type":"integer","description":"检查级别(1-3)","default":1}},"required":["claims"]})
        }
        "MarkdownParser" => {
            json!({"type":"object","properties":{"text":{"type":"string","description":"Markdown文本"},"format_options":{"type":"array","items":{"type":"string"},"description":"格式化选项"}},"required":["text"]})
        }
        "OcrBridge" => {
            json!({"type":"object","properties":{"image_path":{"type":"string","description":"图片文件路径"},"language":{"type":"string","description":"OCR语言(如zh/en)"},"operation":{"type":"string","description":"操作类型"}},"required":["image_path"]})
        }
        "PatentCompareTool" => {
            json!({"type":"object","properties":{"target":{"type":"string","description":"目标专利文本"},"prior_art":{"type":"string","description":"对比文件文本"}},"required":["target","prior_art"]})
        }
        "PatentInfringement" => {
            json!({"type":"object","properties":{"claim_text":{"type":"string","description":"权利要求文本"},"accused_product_description":{"type":"string","description":"被控侵权产品描述"}},"required":["claim_text","accused_product_description"]})
        }
        "PatentManager" => {
            json!({"type":"object","properties":{"action":{"type":"string","description":"操作类型(list/detail/create/update)","default":"list"}},"required":[]})
        }
        "PdfTools" => {
            json!({"type":"object","properties":{"operation":{"type":"string","description":"操作类型(extract/merge/split)"},"content":{"type":"string","description":"PDF内容或Base64"},"file_path":{"type":"string","description":"PDF文件路径"}},"required":["operation"]})
        }
        "ProcessChart" => {
            json!({"type":"object","properties":{"process_type":{"type":"string","description":"流程类型(application/examination/invalidation)","default":"application"}},"required":[]})
        }
        "QualityChecker" => {
            json!({"type":"object","properties":{"claims":{"type":"array","items":{"type":"object","properties":{"claim_type":{"type":"string"},"preamble":{"type":"string"},"elements":{"type":"array","items":{"type":"string"}}}}},"patent_type":{"type":"string","default":"invention"}},"required":["claims"]})
        }
        "Researcher" => {
            json!({"type":"object","properties":{"query":{"type":"string","description":"调研查询"},"depth":{"type":"integer","description":"调研深度(1-5)","default":2}},"required":["query"]})
        }
        "ResponseTemplate" => {
            json!({"type":"object","properties":{"oa_type":{"type":"string","description":"审查意见类型"},"format":{"type":"string","description":"输出格式"}},"required":["oa_type"]})
        }
        "SemanticCompare" => {
            json!({"type":"object","properties":{"text_a":{"type":"string","description":"文本A"},"text_b":{"type":"string","description":"文本B"},"mode":{"type":"string","description":"对比模式(lexical/structural/hybrid)","default":"hybrid"}},"required":["text_a","text_b"]})
        }
        "SpecFormalityChecker" => {
            json!({"type":"object","properties":{"specification":{"type":"object","properties":{"technical_field":{"type":"string"},"background_art":{"type":"string"},"invention_content":{"type":"string"},"embodiment":{"type":"string"},"drawings_description":{"type":"string"}}},"claims":{"type":"array","items":{"type":"string"}},"patent_type":{"type":"string","default":"invention"}},"required":["specification","claims"]})
        }
        "SpecOutputProcessor" => {
            json!({"type":"object","properties":{},"required":[]})
        }
        "ClaimOutputProcessor" => {
            json!({"type":"object","properties":{},"required":[]})
        }
        "StrategyArgumentGenerator" => {
            json!({"type":"object","properties":{"oa_type":{"type":"string","description":"审查意见类型"},"differences":{"type":"array","items":{"type":"string"},"description":"区别特征"},"technical_effects":{"type":"array","items":{"type":"string"},"description":"技术效果"},"legal_basis":{"type":"string","description":"法律依据"}},"required":["oa_type","differences","technical_effects"]})
        }
        "TechUnit" => {
            json!({"type":"object","properties":{"claim_text":{"type":"string","description":"权利要求文本"}},"required":["claim_text"]})
        }
        "TemplateLibrary" => {
            json!({"type":"object","properties":{"template_id":{"type":"string","description":"模板ID"},"variables":{"type":"object","additionalProperties":{"type":"string"},"description":"模板变量"}},"required":["template_id"]})
        }
        "TemplateManager" => {
            json!({"type":"object","properties":{"template_type":{"type":"string","description":"模板类型(patent_application/oa_response/invalidation)","default":"patent_application"}},"required":[]})
        }
        "UnityChecker" => {
            json!({"type":"object","properties":{"claims":{"type":"array","items":{"type":"string"},"description":"权利要求列表"},"patent_type":{"type":"string"},"invention_title":{"type":"string"}},"required":["claims"]})
        }
        "WriterTool" => {
            json!({"type":"object","properties":{"topic":{"type":"string","description":"撰写主题"}},"required":["topic"]})
        }

        // ── Council 域 ──
        "CouncilDeliberate" => {
            json!({"type":"object","properties":{
                "task":{"type":"string","description":"审议任务描述（如：判断权利要求1相对于D1的创造性）"},
                "models":{"type":"string","description":"Council 成员模型（逗号分隔，默认：deepseek-ai/DeepSeek-V3,Qwen/Qwen2.5-72B-Instruct,deepseek-ai/DeepSeek-R1）","default":"deepseek-ai/DeepSeek-V3,Qwen/Qwen2.5-72B-Instruct,deepseek-ai/DeepSeek-R1"},
                "chairman":{"type":"string","description":"Chairman 模型（默认：Qwen/Qwen2.5-72B-Instruct）","default":"Qwen/Qwen2.5-72B-Instruct"},
                "criteria":{"type":"string","description":"评审维度（逗号分隔，默认：准确性,法律依据,论证深度,完整性）","default":"准确性,法律依据,论证深度,完整性"},
                "api_base":{"type":"string","description":"API 地址（如 https://api.siliconflow.cn/v1，不填则读 OPENAI_BASE_URL 环境变量）"}
            },"required":["task"]})
        }
        "CouncilQualityGate" => {
            json!({"type":"object","properties":{
                "document":{"type":"string","description":"待评审文档全文"},
                "document_type":{"type":"string","description":"文档类型","enum":["权利要求书","说明书","审查意见答复"]},
                "threshold":{"type":"number","description":"通过阈值 (0.0-1.0，默认0.7)","default":0.7},
                "models":{"type":"string","description":"Council 成员模型（逗号分隔）","default":"deepseek-ai/DeepSeek-V3,Qwen/Qwen2.5-72B-Instruct,deepseek-ai/DeepSeek-R1"},
                "chairman":{"type":"string","description":"Chairman 模型","default":"Qwen/Qwen2.5-72B-Instruct"},
                "api_base":{"type":"string","description":"API 地址（默认 https://api.siliconflow.cn/v1，不填则读 OPENAI_BASE_URL 环境变量）"}
            },"required":["document","document_type"]})
        }

        "ExaminerSimulate" => {
            json!({"type":"object","properties":{"oa_text":{"type":"string","description":"审查意见全文"},"claims":{"type":"array","items":{"type":"string"},"description":"权利要求列表"}},"required":["oa_text","claims"]})
        }
        "ExaminerRespond" => {
            json!({"type":"object","properties":{"applicant_argument":{"type":"string","description":"申请人论点"},"rejection_type":{"type":"string","description":"驳回类型"},"round_number":{"type":"integer","description":"答复轮次","default":1}},"required":["applicant_argument"]})
        }
        "ResponseEvaluate" => {
            json!({"type":"object","properties":{"response_text":{"type":"string","description":"答复意见书全文"}},"required":["response_text"]})
        }
        "RuleAnalysis" => {
            json!({"type":"object","properties":{"analysis_type":{"type":"string","description":"分析类型","enum":["novelty","inventiveness","oa_strategy"]},"differences":{"type":"array","items":{"type":"string"},"description":"区别特征"},"technical_effects":{"type":"array","items":{"type":"string"},"description":"技术效果"},"rejection_type":{"type":"string","description":"驳回类型"}},"required":["analysis_type"]})
        }
        "OaFeedbackRecord" => {
            json!({"type":"object","properties":{"oa_id":{"type":"string","description":"审查意见ID"},"patent_id":{"type":"string","description":"专利ID"},"feedback_type":{"type":"string","description":"反馈类型","enum":["success","partial_success","failure","quality_issue"]},"outcome":{"type":"string","description":"结果（allowed/rejected/partial）"},"quality_score":{"type":"number","description":"质量评分(0.0-1.0)"},"strategy_used":{"type":"string","description":"使用的策略"},"comments":{"type":"string","description":"备注"}},"required":["oa_id","patent_id","feedback_type","outcome","quality_score"]})
        }
        "OaPatternExtract" => {
            json!({"type":"object","properties":{"min_support":{"type":"integer","description":"最小支持度","default":3},"min_success_rate":{"type":"number","description":"最小成功率","default":0.6}},"required":[]})
        }
        "ScenarioDispatch" => {
            json!({"type":"object","properties":{"task_type":{"type":"string","description":"任务类型","enum":["oa_strategy","novelty_analysis","inventiveness_rejection","infringement_analysis","quality_review"]}},"required":["task_type"]})
        }

        _ => fallback(),
    }
}
