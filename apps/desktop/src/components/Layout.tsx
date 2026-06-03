import type { FC, ReactNode } from 'react';
import TitleBar from './TitleBar';
import StatusBar from './StatusBar';

interface LayoutProps {
  children: ReactNode;
  title?: string;
  showTrafficLights?: boolean;
  showTitleBar?: boolean;
  showStatusBar?: boolean;
}

const Layout: FC<LayoutProps> = ({
  children,
  title,
  showTrafficLights = true,
  showTitleBar = true,
  showStatusBar = true,
}) => {
  return (
    <div
      className="w-screen overflow-hidden"
      style={{
        height: '100vh',
        display: 'grid',
        gridTemplateColumns: 'auto 1fr auto',
        gridTemplateRows: '38px 1fr 28px',
        gridTemplateAreas: `
          "titlebar titlebar titlebar"
          "left main right"
          "statusbar statusbar statusbar"
        `,
      }}
    >
      {/* Title Bar */}
      {showTitleBar && (
        <div style={{ gridArea: 'titlebar' }}>
          <TitleBar title={title} showTrafficLights={showTrafficLights} />
        </div>
      )}

      {/* Main Content Area */}
      <main
        className="overflow-hidden"
        style={{
          gridArea: 'main',
          backgroundColor: 'var(--bg-base)',
          display: 'grid',
          gridTemplateColumns: 'auto 1fr auto',
        }}
      >
        {children}
      </main>

      {/* Status Bar */}
      {showStatusBar && (
        <div style={{ gridArea: 'statusbar' }}>
          <StatusBar />
        </div>
      )}
    </div>
  );
};

export default Layout;
