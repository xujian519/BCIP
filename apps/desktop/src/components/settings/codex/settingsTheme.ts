/** Codex 设置页共享样式 —— 对齐设计规范 §10 / §2 token */
export const settingsTheme = {
  pageTitle: 'text-xl font-semibold text-[var(--text-primary)]',
  pageLead: 'text-xs text-[var(--text-secondary)]',
  sectionTitle: 'text-base font-semibold text-[var(--text-primary)]',
  body: 'text-sm text-[var(--text-primary)]',
  bodyMuted: 'text-sm text-[var(--text-secondary)]',
  caption: 'text-xs text-[var(--text-secondary)]',
  monoCaption: 'text-[11px] font-mono text-[var(--text-tertiary)]',
  monoPath: 'text-[11px] font-mono text-[var(--text-secondary)]',

  card: 'bg-[var(--bg-elevated)] rounded-2xl p-4 mb-4 border border-[var(--border-default)]',
  cardInset: 'bg-[var(--bg-elevated)] rounded-lg border border-[var(--border-default)]',
  listShell:
    'bg-[var(--bg-elevated)] rounded-xl border border-[var(--border-default)] overflow-hidden',
  rowDivider: 'border-b border-[var(--border-default)] last:border-0',

  input:
    'w-full h-9 px-3 bg-[var(--bg-base)] rounded-lg border border-[var(--border-default)] text-sm text-[var(--text-primary)] placeholder:text-[var(--text-tertiary)] focus:outline-none focus:border-[var(--border-focus)] transition-colors duration-150',
  inputWithIcon: 'pl-9 pr-3',
  selectTrigger:
    'bg-[var(--bg-base)] border-[var(--border-default)] text-[var(--text-primary)]',
  selectContent: 'bg-[var(--bg-elevated)] border-[var(--border-default)]',

  ghostButton:
    'text-xs text-[var(--text-secondary)] hover:text-[var(--text-primary)] disabled:opacity-50 transition-colors duration-150',
  secondaryButton:
    'h-8 px-3 flex items-center gap-1.5 bg-[var(--bg-hover)] hover:bg-[var(--bg-active)] text-[var(--text-secondary)] hover:text-[var(--text-primary)] text-xs rounded-lg transition-colors duration-150 disabled:opacity-50',
  primaryButton:
    'h-8 px-4 flex items-center gap-2 bg-[var(--accent-primary)] hover:bg-[var(--accent-primary-hover)] text-[var(--text-inverse)] text-xs font-medium rounded-lg transition-colors duration-150',

  errorBanner:
    'mb-4 p-3 rounded-lg bg-[var(--status-error-bg)] border border-[var(--status-error)]/25',
  errorTitle: 'text-xs font-medium text-[var(--status-error)] mb-1',
  errorBody: 'text-[11px] text-[var(--text-secondary)] font-mono',

  accentText: 'text-[var(--accent-primary)]',
  accentHover:
    'text-[var(--accent-primary)] hover:bg-[var(--accent-primary-muted)]',

  shell: 'fixed inset-0 z-[200] flex min-h-0 bg-[var(--bg-surface)]',
  content: 'min-w-0 flex-1 overflow-y-auto bg-[var(--bg-surface)]',
  contentInner: 'mx-auto w-full max-w-[720px] px-10 py-8',

  navShell:
    'relative z-[1] flex h-full w-[200px] shrink-0 flex-col border-r border-[var(--border-default)] bg-[var(--bg-elevated)] p-3 select-none',
  navBack:
    'mb-4 flex h-8 cursor-pointer items-center gap-2 rounded-md px-2 text-[13px] font-medium text-[var(--text-secondary)] transition-colors duration-150 hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)]',
  navItem:
    'h-9 px-3 flex items-center gap-3 rounded-md text-[13px] transition-all duration-150',
  navItemActive: 'bg-[var(--bg-active)] text-[var(--text-primary)]',
  navItemIdle:
    'text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)]',
} as const;

export const mcpStatusColors = {
  starting: 'var(--status-warning)',
  ready: 'var(--status-success)',
  failed: 'var(--status-error)',
  cancelled: 'var(--text-tertiary)',
} as const;
