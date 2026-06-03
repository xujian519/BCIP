import { useCallback, useEffect, useState } from 'react';
import { File, Loader2, Search as SearchIcon } from 'lucide-react';
import { useAppStore } from '@/hooks/useAppStore';
import { useProjects } from '@/hooks/useProjects';
import { searchWorkspace, type WorkspaceSearchHit } from '@/lib/workspaceSearch';
import { tabIdForPath } from '@/lib/workspaceLayout';

export default function SearchPanel() {
  const { state, dispatch } = useAppStore();
  const { projects } = useProjects();
  const root = state.workspaceCwd ?? projects[0]?.path ?? '';
  const [query, setQuery] = useState('');
  const [hits, setHits] = useState<WorkspaceSearchHit[]>([]);
  const [searching, setSearching] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!query.trim() || !root) {
      setHits([]);
      setError(null);
      setSearching(false);
      return;
    }

    setSearching(true);
    setError(null);
    const timer = window.setTimeout(() => {
      void searchWorkspace(root, query)
        .then((results) => {
          setHits(results);
        })
        .catch((err) => {
          setError(err instanceof Error ? err.message : String(err));
          setHits([]);
        })
        .finally(() => {
          setSearching(false);
        });
    }, 250);

    return () => window.clearTimeout(timer);
  }, [query, root]);

  const openHit = useCallback(
    (hit: WorkspaceSearchHit) => {
      dispatch({ type: 'SET_CURRENT_FILE', payload: hit.path });
      dispatch({
        type: 'OPEN_TAB',
        payload: {
          id: tabIdForPath(hit.path),
          filePath: hit.path,
          title: hit.name,
        },
      });
    },
    [dispatch],
  );

  return (
    <div className="flex h-full flex-col overflow-hidden">
      <div
        className="shrink-0 px-3 py-2 text-xs font-medium"
        style={{ color: 'var(--text-secondary)' }}
      >
        搜索
      </div>
      <div className="shrink-0 px-3 pb-2">
        <div className="relative">
          <SearchIcon
            size={14}
            className="pointer-events-none absolute"
            style={{
              left: 10,
              top: '50%',
              transform: 'translateY(-50%)',
              color: 'var(--text-tertiary)',
            }}
          />
          <input
            type="search"
            placeholder="搜索文件名或内容…"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            className="w-full outline-none transition-colors duration-fast"
            style={{
              height: 30,
              padding: '6px 10px 6px 32px',
              fontSize: 12,
              borderRadius: 8,
              backgroundColor: 'var(--bg-elevated)',
              border: '1px solid var(--border-primary)',
              color: 'var(--text-primary)',
            }}
          />
        </div>
      </div>

      <div className="min-h-0 flex-1 overflow-y-auto px-2 pb-2">
        {!root && (
          <p className="p-4 text-center text-xs" style={{ color: 'var(--text-tertiary)' }}>
            请先打开工作区目录
          </p>
        )}
        {root && !query.trim() && (
          <p className="p-4 text-center text-xs" style={{ color: 'var(--text-tertiary)' }}>
            输入关键词搜索文件内容
          </p>
        )}
        {searching && (
          <div
            className="flex items-center justify-center gap-2 p-4 text-xs"
            style={{ color: 'var(--text-tertiary)' }}
          >
            <Loader2 size={14} className="animate-spin" />
            搜索中…
          </div>
        )}
        {error && (
          <p className="p-4 text-center text-xs" style={{ color: 'var(--status-error)' }}>
            {error}
          </p>
        )}
        {!searching && query.trim() && hits.length === 0 && !error && (
          <p className="p-4 text-center text-xs" style={{ color: 'var(--text-tertiary)' }}>
            未找到匹配项
          </p>
        )}
        {hits.map((hit) => (
          <button
            key={`${hit.path}-${hit.kind}`}
            type="button"
            onClick={() => openHit(hit)}
            className="mb-1 flex w-full flex-col gap-0.5 rounded-md px-2 py-2 text-left transition-colors duration-fast hover:bg-[var(--bg-hover)]"
          >
            <span className="flex items-center gap-1.5 text-xs font-medium text-[var(--text-primary)]">
              <File size={12} className="shrink-0 text-[var(--text-tertiary)]" />
              <span className="truncate">{hit.name}</span>
              <span
                className="shrink-0 rounded px-1 py-0.5 text-[10px] uppercase"
                style={{
                  backgroundColor: 'var(--bg-active)',
                  color: 'var(--text-tertiary)',
                }}
              >
                {hit.kind === 'filename' ? '文件名' : '内容'}
              </span>
            </span>
            {hit.snippet && (
              <span className="truncate pl-5 text-[11px] text-[var(--text-secondary)]">
                {hit.snippet}
              </span>
            )}
            <span className="truncate pl-5 font-mono text-[10px] text-[var(--text-tertiary)]">
              {hit.path}
            </span>
          </button>
        ))}
      </div>
    </div>
  );
}
