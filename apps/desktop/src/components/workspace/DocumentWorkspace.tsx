import WelcomeScreen from './WelcomeScreen';
import WorkspaceSplit from './WorkspaceSplit';
import { WorkspaceDragProvider } from './WorkspaceDragContext';
import { useAppStore } from '@/hooks/useAppStore';
import { useWorkspaceLayoutPersistence } from '@/hooks/useWorkspaceLayoutPersistence';

export default function DocumentWorkspace() {
  const { state } = useAppStore();
  useWorkspaceLayoutPersistence();

  return (
    <WorkspaceDragProvider>
      <div
        className="flex h-full flex-col overflow-hidden"
        style={{ backgroundColor: 'var(--bg-surface)' }}
      >
        {state.workspaceRoot ? (
          <WorkspaceSplit node={state.workspaceRoot} />
        ) : (
          <WelcomeScreen />
        )}
      </div>
    </WorkspaceDragProvider>
  );
}
