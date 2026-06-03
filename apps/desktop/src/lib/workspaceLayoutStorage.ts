import type { WorkspaceNode, WorkspaceLeafNode } from '@/types';
import {
  findFirstLeaf,
  findTabByPath,
  tabIdForPath,
} from '@/lib/workspaceLayout';

const STORAGE_KEY = 'bcip-workspace-layouts-v1';

interface PersistedTab {
  filePath: string;
  title: string;
}

interface PersistedLeaf {
  type: 'leaf';
  tabs: PersistedTab[];
  activeFilePath: string | null;
}

interface PersistedSplit {
  type: 'split';
  direction: 'horizontal' | 'vertical';
  ratio: number;
  first: PersistedNode;
  second: PersistedNode;
}

type PersistedNode = PersistedLeaf | PersistedSplit;

interface PersistedProjectLayout {
  version: 1;
  focusedFilePath: string | null;
  root: PersistedNode | null;
}

type LayoutStore = Record<string, PersistedProjectLayout>;

function newId(prefix: string): string {
  if (typeof crypto !== 'undefined' && 'randomUUID' in crypto) {
    return `${prefix}-${crypto.randomUUID()}`;
  }
  return `${prefix}-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`;
}

function readStore(): LayoutStore {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) {
      return {};
    }
    const parsed = JSON.parse(raw) as unknown;
    if (!parsed || typeof parsed !== 'object') {
      return {};
    }
    return parsed as LayoutStore;
  } catch {
    return {};
  }
}

function writeStore(store: LayoutStore): void {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(store));
}

function serializeNode(node: WorkspaceNode): PersistedNode {
  if (node.type === 'leaf') {
    const activeFilePath =
      node.tabs.find((tab) => tab.id === node.activeTabId)?.filePath ?? null;
    return {
      type: 'leaf',
      tabs: node.tabs.map((tab) => ({
        filePath: tab.filePath,
        title: tab.title,
      })),
      activeFilePath,
    };
  }
  return {
    type: 'split',
    direction: node.direction,
    ratio: node.ratio,
    first: serializeNode(node.first),
    second: serializeNode(node.second),
  };
}

function deserializeNode(node: PersistedNode): WorkspaceNode {
  if (node.type === 'leaf') {
    const tabs = node.tabs.map((entry) => ({
      id: tabIdForPath(entry.filePath),
      filePath: entry.filePath,
      title: entry.title,
    }));
    const activeTabId = node.activeFilePath
      ? (tabs.find((tab) => tab.filePath === node.activeFilePath)?.id ?? null)
      : (tabs.at(-1)?.id ?? null);
    return {
      type: 'leaf',
      id: newId('pane'),
      tabs,
      activeTabId,
    };
  }
  return {
    type: 'split',
    id: newId('split'),
    direction: node.direction === 'vertical' ? 'vertical' : 'horizontal',
    ratio: node.ratio,
    first: deserializeNode(node.first),
    second: deserializeNode(node.second),
  };
}

export interface RestoredWorkspaceLayout {
  root: WorkspaceNode | null;
  focusedPaneId: string | null;
  currentFile: string | null;
}

function resolveFocus(
  root: WorkspaceNode | null,
  focusedFilePath: string | null,
): RestoredWorkspaceLayout {
  if (!root) {
    return { root: null, focusedPaneId: null, currentFile: null };
  }

  const byPath = focusedFilePath
    ? findTabByPath(root, focusedFilePath)
    : null;
  const focusedPaneId =
    byPath?.pane.id ?? findFirstLeaf(root)?.id ?? null;
  const currentFile =
    byPath?.tab.filePath ??
    activeFileInPane(root, focusedPaneId) ??
    focusedFilePath;

  return { root, focusedPaneId, currentFile };
}

function activeFileInPane(
  root: WorkspaceNode,
  paneId: string | null,
): string | null {
  if (!paneId) {
    return null;
  }
  const leaf =
    root.type === 'leaf'
      ? root.id === paneId
        ? root
        : null
      : findLeafInNode(root, paneId);
  if (!leaf?.activeTabId) {
    return null;
  }
  return leaf.tabs.find((tab) => tab.id === leaf.activeTabId)?.filePath ?? null;
}

function findLeafInNode(
  node: WorkspaceNode,
  paneId: string,
): WorkspaceLeafNode | null {
  if (node.type === 'leaf') {
    return node.id === paneId ? node : null;
  }
  return (
    findLeafInNode(node.first, paneId) ?? findLeafInNode(node.second, paneId)
  );
}

export function loadWorkspaceLayout(projectPath: string): RestoredWorkspaceLayout | null {
  const store = readStore();
  const entry = store[projectPath];
  if (!entry || entry.version !== 1) {
    return null;
  }
  if (!entry.root) {
    return { root: null, focusedPaneId: null, currentFile: null };
  }
  const root = deserializeNode(entry.root);
  return resolveFocus(root, entry.focusedFilePath);
}

export function saveWorkspaceLayout(
  projectPath: string,
  root: WorkspaceNode | null,
  focusedPaneId: string | null,
): void {
  const trimmed = projectPath.trim();
  if (!trimmed) {
    return;
  }

  const store = readStore();
  if (!root) {
    delete store[trimmed];
    writeStore(store);
    return;
  }

  const focusedFilePath = activeFileInPane(root, focusedPaneId);
  store[trimmed] = {
    version: 1,
    focusedFilePath,
    root: serializeNode(root),
  };
  writeStore(store);
}
