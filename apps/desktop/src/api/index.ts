import * as tauriApi from './tauri';
import * as mockApi from './mock';

const useMock = import.meta.env.VITE_USE_MOCK === 'true';

function hasTauri(): boolean {
  return typeof window !== 'undefined' && !!(window as unknown as Record<string, unknown>).__TAURI__;
}

const isMock = useMock || !hasTauri();
const resolved = isMock ? mockApi : tauriApi;

export const api = resolved;
export { isMock };
