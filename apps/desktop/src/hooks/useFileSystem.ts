import { useState, useCallback, useEffect, useRef } from 'react';
import { readDirectory, writeFile, createDirectory, deleteFile } from '@/lib/fileSystem';
import type { FileEntry } from '@/lib/fileSystem';
import {
  getCachedDirectory,
  invalidateDirectoryTree,
  setCachedDirectory,
} from '@/lib/fileTreeCache';
import { BCIP_FILE_TREE_REFRESH } from '@/lib/desktopEvents';

export interface FileSystemState {
  rootPath: string;
  files: FileEntry[];
  loading: boolean;
  error: string | null;
  selectedPath: string | null;
  expandedPaths: Set<string>;
}

const expandedByRoot = new Map<string, Set<string>>();

function loadExpanded(rootPath: string): Set<string> {
  return new Set(expandedByRoot.get(rootPath) ?? []);
}

function saveExpanded(rootPath: string, paths: Set<string>): void {
  expandedByRoot.set(rootPath, new Set(paths));
}

export function useFileSystem(rootPath: string) {
  const rootRef = useRef(rootPath);
  const [state, setState] = useState<FileSystemState>(() => ({
    rootPath,
    files: getCachedDirectory(rootPath) ?? [],
    loading: getCachedDirectory(rootPath) === null,
    error: null,
    selectedPath: null,
    expandedPaths: loadExpanded(rootPath),
  }));

  const refresh = useCallback(async (path: string, force = false) => {
    if (!force) {
      const cached = getCachedDirectory(path);
      if (cached) {
        setState((prev) => ({
          ...prev,
          rootPath: path,
          files: cached,
          loading: false,
          error: null,
        }));
        return;
      }
    }

    setState((prev) => ({ ...prev, rootPath: path, loading: true, error: null }));
    try {
      const entries = await readDirectory(path);
      setCachedDirectory(path, entries);
      setState((prev) => ({ ...prev, files: entries, loading: false }));
    } catch (error) {
      setState((prev) => ({
        ...prev,
        loading: false,
        error: error instanceof Error ? error.message : '未知错误',
      }));
    }
  }, []);

  useEffect(() => {
    if (rootRef.current !== rootPath) {
      setState((prev) => {
        saveExpanded(rootRef.current, prev.expandedPaths);
        const cached = getCachedDirectory(rootPath);
        return {
          rootPath,
          files: cached ?? [],
          loading: cached === null,
          error: null,
          selectedPath: null,
          expandedPaths: loadExpanded(rootPath),
        };
      });
      rootRef.current = rootPath;
    }
    void refresh(rootPath);
  }, [rootPath, refresh]);

  useEffect(() => {
    const handler = () => {
      invalidateDirectoryTree(rootRef.current);
      void refresh(rootRef.current, true);
    };
    window.addEventListener(BCIP_FILE_TREE_REFRESH, handler);
    return () => window.removeEventListener(BCIP_FILE_TREE_REFRESH, handler);
  }, [refresh]);

  const selectFile = useCallback((path: string) => {
    setState((prev) => ({ ...prev, selectedPath: path }));
  }, []);

  const toggleExpanded = useCallback((path: string) => {
    setState((prev) => {
      const newExpanded = new Set(prev.expandedPaths);
      if (newExpanded.has(path)) {
        newExpanded.delete(path);
      } else {
        newExpanded.add(path);
      }
      saveExpanded(prev.rootPath, newExpanded);
      return { ...prev, expandedPaths: newExpanded };
    });
  }, []);

  const loadChildren = useCallback(async (path: string): Promise<FileEntry[]> => {
    const cached = getCachedDirectory(path);
    if (cached) return cached;
    const entries = await readDirectory(path);
    setCachedDirectory(path, entries);
    return entries;
  }, []);

  const createFile = useCallback(
    async (path: string, content: string = '') => {
      await writeFile(path, content);
      invalidateDirectoryTree(rootRef.current);
      await refresh(rootRef.current, true);
    },
    [refresh],
  );

  const createFolder = useCallback(
    async (path: string) => {
      await createDirectory(path);
      invalidateDirectoryTree(rootRef.current);
      await refresh(rootRef.current, true);
    },
    [refresh],
  );

  const removeFile = useCallback(
    async (path: string) => {
      await deleteFile(path);
      invalidateDirectoryTree(rootRef.current);
      await refresh(rootRef.current, true);
    },
    [refresh],
  );

  const setRootPath = useCallback(
    (path: string) => {
      void refresh(path);
    },
    [refresh],
  );

  return {
    ...state,
    refresh: () => refresh(state.rootPath, true),
    selectFile,
    toggleExpanded,
    loadChildren,
    createFile,
    createFolder,
    removeFile,
    setRootPath,
  };
}
