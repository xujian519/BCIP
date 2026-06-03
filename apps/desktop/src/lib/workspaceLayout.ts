import type {
  WorkspaceLeafNode,
  WorkspaceNode,
  WorkspaceSplitNode,
  WorkspaceSplitSide,
  WorkspaceTab,
} from '@/types';

const MIN_SPLIT_RATIO = 0.15;
const MAX_SPLIT_RATIO = 0.85;

/** 允许的最大 pane 嵌套深度（根 pane 为 1） */
export const MAX_WORKSPACE_SPLIT_DEPTH = 3;

function visitLeaves(
  root: WorkspaceNode | null,
  callback: (leaf: WorkspaceLeafNode) => void,
): void {
  if (!root) return;
  if (root.type === 'leaf') { callback(root); return; }
  visitLeaves(root.first, callback);
  visitLeaves(root.second, callback);
}

function newId(prefix: string): string {
  if (typeof crypto !== 'undefined' && 'randomUUID' in crypto) {
    return `${prefix}-${crypto.randomUUID()}`;
  }
  return `${prefix}-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`;
}

export function createLeaf(tab?: WorkspaceTab): WorkspaceLeafNode {
  return {
    type: 'leaf',
    id: newId('pane'),
    tabs: tab ? [tab] : [],
    activeTabId: tab?.id ?? null,
  };
}

export function findLeaf(
  root: WorkspaceNode | null,
  paneId: string,
): WorkspaceLeafNode | null {
  if (!root) {
    return null;
  }
  if (root.type === 'leaf') {
    return root.id === paneId ? root : null;
  }
  return findLeaf(root.first, paneId) ?? findLeaf(root.second, paneId);
}

export function findFirstLeaf(root: WorkspaceNode | null): WorkspaceLeafNode | null {
  if (!root) {
    return null;
  }
  if (root.type === 'leaf') {
    return root;
  }
  return findFirstLeaf(root.first) ?? findFirstLeaf(root.second);
}

export function findLeafByTabId(
  root: WorkspaceNode | null,
  tabId: string,
): WorkspaceLeafNode | null {
  if (!root) {
    return null;
  }
  if (root.type === 'leaf') {
    return root.tabs.some((tab) => tab.id === tabId) ? root : null;
  }
  return (
    findLeafByTabId(root.first, tabId) ?? findLeafByTabId(root.second, tabId)
  );
}

export function findTabByPath(
  root: WorkspaceNode | null,
  filePath: string,
): { pane: WorkspaceLeafNode; tab: WorkspaceTab } | null {
  if (!root) {
    return null;
  }
  if (root.type === 'leaf') {
    const tab = root.tabs.find((entry) => entry.filePath === filePath);
    return tab ? { pane: root, tab } : null;
  }
  return (
    findTabByPath(root.first, filePath) ?? findTabByPath(root.second, filePath)
  );
}

export function getPaneDepth(root: WorkspaceNode | null, paneId: string): number {
  function walk(node: WorkspaceNode, depth: number): number | null {
    if (node.type === 'leaf') {
      return node.id === paneId ? depth : null;
    }
    return walk(node.first, depth + 1) ?? walk(node.second, depth + 1);
  }
  if (!root) {
    return 0;
  }
  return walk(root, 1) ?? 0;
}

export function countWorkspacePanes(root: WorkspaceNode | null): number {
  let count = 0;
  visitLeaves(root, () => count++);
  return count;
}

export function getPaneOrdinal(root: WorkspaceNode | null, paneId: string): number {
  if (!root) {
    return 0;
  }
  let ordinal = 0;
  let found = 0;
  function walk(node: WorkspaceNode): void {
    if (node.type === 'leaf') {
      ordinal += 1;
      if (node.id === paneId) {
        found = ordinal;
      }
      return;
    }
    walk(node.first);
    walk(node.second);
  }
  walk(root);
  return found;
}

export function findLastLeaf(root: WorkspaceNode | null): WorkspaceLeafNode | null {
  if (!root) {
    return null;
  }
  if (root.type === 'leaf') {
    return root;
  }
  return findLastLeaf(root.second) ?? findLastLeaf(root.first);
}

interface SplitParentInfo {
  parentSplit: WorkspaceSplitNode;
  sourceLeaf: WorkspaceLeafNode;
  sourceIsFirst: boolean;
}

function findSplitParent(
  root: WorkspaceNode,
  paneId: string,
): SplitParentInfo | null {
  if (root.type === 'leaf') {
    return null;
  }
  if (root.first.type === 'leaf' && root.first.id === paneId) {
    return {
      parentSplit: root,
      sourceLeaf: root.first,
      sourceIsFirst: true,
    };
  }
  if (root.second.type === 'leaf' && root.second.id === paneId) {
    return {
      parentSplit: root,
      sourceLeaf: root.second,
      sourceIsFirst: false,
    };
  }
  return (
    findSplitParent(root.first, paneId) ?? findSplitParent(root.second, paneId)
  );
}

export function canMergePane(
  root: WorkspaceNode | null,
  paneId: string,
): boolean {
  if (!root) {
    return false;
  }
  return findSplitParent(root, paneId) !== null;
}

function collectLeafIds(node: WorkspaceNode): string[] {
  const ids: string[] = [];
  visitLeaves(node, (leaf) => ids.push(leaf.id));
  return ids;
}

/** 反复合并 pane，直到只剩一个编辑区（关闭全部分屏） */
export function collapseAllSplitsInLayout(
  root: WorkspaceNode,
): { root: WorkspaceNode; focusedPaneId: string } {
  let current = root;
  let focusedPaneId = findFirstLeaf(current)?.id ?? '';

  while (countWorkspacePanes(current) > 1) {
    let progressed = false;
    for (const paneId of collectLeafIds(current)) {
      if (!canMergePane(current, paneId)) {
        continue;
      }
      const merged = mergePaneInLayout(current, paneId);
      if (!merged) {
        continue;
      }
      current = merged.root;
      focusedPaneId = merged.focusedPaneId;
      progressed = true;
      break;
    }
    if (!progressed) {
      break;
    }
  }

  return { root: current, focusedPaneId };
}

export function mergePaneInLayout(
  root: WorkspaceNode,
  paneId: string,
): { root: WorkspaceNode; focusedPaneId: string } | null {
  const info = findSplitParent(root, paneId);
  if (!info) {
    return null;
  }

  const { parentSplit, sourceLeaf, sourceIsFirst } = info;
  const sibling = sourceIsFirst ? parentSplit.second : parentSplit.first;
  const absorbLeaf = sourceIsFirst
    ? findFirstLeaf(sibling)
    : findLastLeaf(sibling);
  if (!absorbLeaf) {
    return null;
  }

  const withMergedTabs = updateLeaf(root, absorbLeaf.id, (leaf) => ({
    ...leaf,
    tabs: sourceIsFirst
      ? [...sourceLeaf.tabs, ...leaf.tabs]
      : [...leaf.tabs, ...sourceLeaf.tabs],
    activeTabId: sourceLeaf.activeTabId ?? leaf.activeTabId,
  }));

  const finalRoot = replaceNodeById(withMergedTabs, parentSplit.id, (node) => {
    if (node.type !== 'split') {
      return node;
    }
    return sourceIsFirst ? node.second : node.first;
  });
  if (!finalRoot) {
    return null;
  }

  const normalized = normalizeTree(finalRoot);
  if (!normalized) {
    return null;
  }

  return { root: normalized, focusedPaneId: absorbLeaf.id };
}

function splitDirectionForSide(
  side: WorkspaceSplitSide,
): import('@/types').WorkspaceSplitDirection {
  return side === 'left' || side === 'right' ? 'horizontal' : 'vertical';
}

function isTargetPaneFirst(side: WorkspaceSplitSide): boolean {
  return side === 'left' || side === 'top';
}

export function canSplitPane(root: WorkspaceNode, paneId: string): boolean {
  return getPaneDepth(root, paneId) < MAX_WORKSPACE_SPLIT_DEPTH;
}

export function tabIdForPath(filePath: string): string {
  return `tab-${filePath}`;
}

function updateLeaf(
  root: WorkspaceNode,
  paneId: string,
  updater: (leaf: WorkspaceLeafNode) => WorkspaceLeafNode,
): WorkspaceNode {
  if (root.type === 'leaf') {
    return root.id === paneId ? updater(root) : root;
  }
  return {
    ...root,
    first: updateLeaf(root.first, paneId, updater),
    second: updateLeaf(root.second, paneId, updater),
  };
}

function replaceNodeById(
  node: WorkspaceNode,
  nodeId: string,
  replacer: (node: WorkspaceNode) => WorkspaceNode | null,
): WorkspaceNode | null {
  if (node.type === 'leaf') {
    if (node.id !== nodeId) {
      return node;
    }
    return replacer(node);
  }
  if (node.id === nodeId) {
    return replacer(node);
  }
  const first = replaceNodeById(node.first, nodeId, replacer);
  const second = replaceNodeById(node.second, nodeId, replacer);
  if (!first && !second) {
    return null;
  }
  if (!first) {
    return second;
  }
  if (!second) {
    return first;
  }
  return { ...node, first, second };
}

export function normalizeTree(node: WorkspaceNode): WorkspaceNode | null {
  if (node.type === 'leaf') {
    return node;
  }
  const first = normalizeTree(node.first);
  const second = normalizeTree(node.second);
  if (!first && !second) {
    return null;
  }
  if (!first) {
    return second;
  }
  if (!second) {
    return first;
  }
  return { ...node, first, second };
}

export function openTabInLayout(
  root: WorkspaceNode | null,
  focusedPaneId: string | null,
  tab: WorkspaceTab,
): {
  root: WorkspaceNode;
  focusedPaneId: string;
  activeTabId: string;
} {
  const existing = findTabByPath(root, tab.filePath);
  if (existing) {
    const nextRoot = updateLeaf(root!, existing.pane.id, (leaf) => ({
      ...leaf,
      activeTabId: existing.tab.id,
    }));
    return {
      root: nextRoot,
      focusedPaneId: existing.pane.id,
      activeTabId: existing.tab.id,
    };
  }

  if (!root) {
    const leaf = createLeaf(tab);
    return { root: leaf, focusedPaneId: leaf.id, activeTabId: tab.id };
  }

  const targetPane =
    (focusedPaneId ? findLeaf(root, focusedPaneId) : null) ??
    findFirstLeaf(root);
  const targetPaneId = targetPane!.id;

  const nextRoot = updateLeaf(root, targetPaneId, (leaf) => ({
    ...leaf,
    tabs: [...leaf.tabs, tab],
    activeTabId: tab.id,
  }));

  return { root: nextRoot, focusedPaneId: targetPaneId, activeTabId: tab.id };
}

export function closeTabInLayout(
  root: WorkspaceNode | null,
  tabId: string,
): {
  root: WorkspaceNode | null;
  focusedPaneId: string | null;
} {
  if (!root) {
    return { root: null, focusedPaneId: null };
  }

  const leaf = findLeafByTabId(root, tabId);
  if (!leaf) {
    return { root, focusedPaneId: focusedPaneIdForRoot(root) };
  }

  const remaining = leaf.tabs.filter((tab) => tab.id !== tabId);
  const nextActiveId =
    leaf.activeTabId === tabId
      ? (remaining.at(-1)?.id ?? null)
      : leaf.activeTabId;

  const updated = updateLeaf(root, leaf.id, (entry) => ({
    ...entry,
    tabs: remaining,
    activeTabId: nextActiveId,
  }));

  const normalized = normalizeTree(updated);
  return {
    root: normalized,
    focusedPaneId: focusedPaneIdForRoot(normalized),
  };
}

function focusedPaneIdForRoot(root: WorkspaceNode | null): string | null {
  return findFirstLeaf(root)?.id ?? null;
}

export function setActiveTabInLayout(
  root: WorkspaceNode | null,
  paneId: string,
  tabId: string,
): WorkspaceNode | null {
  if (!root) {
    return null;
  }
  const leaf = findLeaf(root, paneId);
  if (!leaf || !leaf.tabs.some((tab) => tab.id === tabId)) {
    return root;
  }
  return updateLeaf(root, paneId, (entry) => ({
    ...entry,
    activeTabId: tabId,
  }));
}

export function splitTabInLayout(
  root: WorkspaceNode,
  paneId: string,
  tabId: string,
  side: WorkspaceSplitSide,
): {
  root: WorkspaceNode;
  focusedPaneId: string;
  blocked?: boolean;
} {
  if (!canSplitPane(root, paneId)) {
    return { root, focusedPaneId: paneId, blocked: true };
  }

  const leaf = findLeaf(root, paneId);
  if (!leaf) {
    return { root, focusedPaneId: paneId };
  }

  const tab = leaf.tabs.find((entry) => entry.id === tabId);
  if (!tab) {
    return { root, focusedPaneId: paneId };
  }

  const remainingTabs = leaf.tabs.filter((entry) => entry.id !== tabId);
  const sourceLeaf: WorkspaceLeafNode = {
    ...leaf,
    tabs: remainingTabs,
    activeTabId:
      leaf.activeTabId === tabId
        ? (remainingTabs.at(-1)?.id ?? null)
        : leaf.activeTabId,
  };
  const targetLeaf = createLeaf(tab);

  const split: WorkspaceSplitNode = {
    type: 'split',
    id: newId('split'),
    direction: splitDirectionForSide(side),
    ratio: 0.5,
    first: isTargetPaneFirst(side) ? targetLeaf : sourceLeaf,
    second: isTargetPaneFirst(side) ? sourceLeaf : targetLeaf,
  };

  const replaced =
    replaceNodeById(root, paneId, () => split) ??
    normalizeTree(split) ??
    split;

  return { root: replaced, focusedPaneId: targetLeaf.id };
}

export function moveTabInLayout(
  root: WorkspaceNode,
  tabId: string,
  targetPaneId: string,
  insertIndex?: number,
): {
  root: WorkspaceNode | null;
  focusedPaneId: string;
} {
  const sourceLeaf = findLeafByTabId(root, tabId);
  if (!sourceLeaf || !findLeaf(root, targetPaneId)) {
    return { root, focusedPaneId: targetPaneId };
  }
  if (sourceLeaf.id === targetPaneId) {
    return { root, focusedPaneId: targetPaneId };
  }

  const tab = sourceLeaf.tabs.find((entry) => entry.id === tabId);
  if (!tab) {
    return { root, focusedPaneId: targetPaneId };
  }

  const sourceRemaining = sourceLeaf.tabs.filter((entry) => entry.id !== tabId);
  const withRemoved = updateLeaf(root, sourceLeaf.id, (leaf) => ({
    ...leaf,
    tabs: sourceRemaining,
    activeTabId:
      leaf.activeTabId === tabId
        ? (sourceRemaining.at(-1)?.id ?? null)
        : leaf.activeTabId,
  }));

  const withAdded = updateLeaf(withRemoved, targetPaneId, (leaf) => {
    const nextTabs = [...leaf.tabs];
    const at = insertIndex ?? nextTabs.length;
    const clamped = Math.max(0, Math.min(at, nextTabs.length));
    nextTabs.splice(clamped, 0, tab);
    return {
      ...leaf,
      tabs: nextTabs,
      activeTabId: tab.id,
    };
  });

  const normalized = normalizeTree(withAdded);
  return {
    root: normalized,
    focusedPaneId: targetPaneId,
  };
}

export function reorderTabInLayout(
  root: WorkspaceNode,
  paneId: string,
  tabId: string,
  toIndex: number,
): WorkspaceNode {
  return updateLeaf(root, paneId, (leaf) => {
    const fromIndex = leaf.tabs.findIndex((tab) => tab.id === tabId);
    if (fromIndex < 0) {
      return leaf;
    }

    let targetIndex = Math.max(0, Math.min(toIndex, leaf.tabs.length));
    if (fromIndex < targetIndex) {
      targetIndex -= 1;
    }
    if (fromIndex === targetIndex) {
      return leaf;
    }

    const tabs = [...leaf.tabs];
    const [item] = tabs.splice(fromIndex, 1);
    tabs.splice(targetIndex, 0, item);
    return { ...leaf, tabs };
  });
}

export function setSplitRatioInLayout(
  root: WorkspaceNode,
  splitId: string,
  ratio: number,
): WorkspaceNode {
  const clamped = Math.max(MIN_SPLIT_RATIO, Math.min(MAX_SPLIT_RATIO, ratio));

  const visit = (node: WorkspaceNode): WorkspaceNode => {
    if (node.type === 'leaf') {
      return node;
    }
    if (node.id === splitId) {
      return { ...node, ratio: clamped };
    }
    return {
      ...node,
      first: visit(node.first),
      second: visit(node.second),
    };
  };

  return visit(root);
}

export const TAB_DRAG_MIME = 'application/x-bcip-workspace-tab';

export interface TabDragPayload {
  tabId: string;
  paneId: string;
}

export function encodeTabDragPayload(payload: TabDragPayload): string {
  return JSON.stringify(payload);
}

export function decodeTabDragPayload(raw: string): TabDragPayload | null {
  try {
    const parsed = JSON.parse(raw) as TabDragPayload;
    if (
      typeof parsed.tabId === 'string' &&
      typeof parsed.paneId === 'string'
    ) {
      return parsed;
    }
    return null;
  } catch {
    return null;
  }
}
