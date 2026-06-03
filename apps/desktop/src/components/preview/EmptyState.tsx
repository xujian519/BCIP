export default function EmptyState() {
  return (
    <div className="flex flex-col items-center justify-center h-full">
      <div className="text-6xl mb-4 opacity-50">📁</div>
      <h3 className="text-lg font-medium mb-2" style={{ color: 'var(--text-primary)' }}>
        选择文件以开始预览
      </h3>
      <p className="text-sm text-center max-w-md" style={{ color: 'var(--text-tertiary)' }}>
        支持的格式：PDF、图片、Markdown、Word、文本、代码
      </p>
    </div>
  );
}
