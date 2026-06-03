import { api } from '@/api';

export function isLegacyDocPath(filePath: string): boolean {
  const extension = filePath.split('.').pop()?.toLowerCase() ?? '';
  return extension === 'doc';
}

export function isDocxPath(filePath: string): boolean {
  const extension = filePath.split('.').pop()?.toLowerCase() ?? '';
  return extension === 'docx';
}

/** 解析预览用 docx 路径：.docx 原样返回，.doc 经 LibreOffice 转换 */
export async function resolveDocxPreviewPath(filePath: string): Promise<{
  previewPath: string;
  converted: boolean;
  fromCache: boolean;
}> {
  if (isDocxPath(filePath)) {
    return { previewPath: filePath, converted: false, fromCache: false };
  }

  if (!isLegacyDocPath(filePath)) {
    throw new Error('不支持的 Word 格式');
  }

  const result = await api.convertDocToDocx(filePath);
  return {
    previewPath: result.outputPath,
    converted: true,
    fromCache: result.fromCache,
  };
}

export async function loadLibreOfficeStatus() {
  return api.getLibreOfficeStatus();
}
