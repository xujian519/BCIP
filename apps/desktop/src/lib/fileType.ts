export type FileType = 'pdf' | 'image' | 'markdown' | 'docx' | 'doc' | 'text' | 'code' | 'unknown';

const imageExtensions = ['png', 'jpg', 'jpeg', 'gif', 'svg', 'webp', 'bmp', 'ico'];
const codeExtensions = ['js', 'ts', 'tsx', 'jsx', 'html', 'css', 'scss', 'json', 'xml', 'yaml', 'yml', 'rs', 'py', 'go', 'java', 'cpp', 'c', 'h', 'swift', 'kt'];
const textExtensions = ['txt', 'log', 'csv', 'ini', 'conf', 'cfg'];

export function getFileType(fileName: string): FileType {
  const extension = fileName.split('.').pop()?.toLowerCase() || '';
  
  if (extension === 'pdf') return 'pdf';
  if (imageExtensions.includes(extension)) return 'image';
  if (extension === 'md' || extension === 'markdown') return 'markdown';
  if (extension === 'docx') return 'docx';
  if (extension === 'doc') return 'doc';
  if (codeExtensions.includes(extension)) return 'code';
  if (textExtensions.includes(extension)) return 'text';
  
  return 'unknown';
}

export function getFileIcon(fileName: string): string {
  const type = getFileType(fileName);
  
  switch (type) {
    case 'pdf': return '📄';
    case 'image': return '🖼️';
    case 'markdown': return '📝';
    case 'docx':
    case 'doc':
      return '📃';
    case 'code': return '💻';
    case 'text': return '📋';
    default: return '📎';
  }
}

export function isPreviewable(fileName: string): boolean {
  const type = getFileType(fileName);
  return ['pdf', 'image', 'markdown', 'docx', 'doc', 'text', 'code'].includes(type);
}
