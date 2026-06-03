/**
 * 全局浮层：设置、审批、MCP、命令面板（Codex parity）
 */
import CodexSettingsLayout from '@/components/settings/codex/SettingsLayout';
import McpElicitationModal from '@/components/overlays/McpElicitationModal';
import OAuthWaitingSheet from '@/components/overlays/OAuthWaitingSheet';
import CommandPalette from '@/components/overlays/CommandPalette';
import ToolUserInputModal from '@/components/overlays/ToolUserInputModal';

export default function GlobalOverlays() {
  return (
    <>
      <CodexSettingsLayout />
      <ToolUserInputModal />
      <McpElicitationModal />
      <OAuthWaitingSheet />
      <CommandPalette />
    </>
  );
}
