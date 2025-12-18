/* v8 ignore file -- @preserve */
import { useState, useCallback } from "react";
import "./App.css";
import { RecordingIndicator } from "./components/RecordingIndicator";
import { TranscriptionIndicator } from "./components/TranscriptionIndicator";
import { TranscriptionNotification } from "./components/TranscriptionNotification";
import { AudioErrorDialog } from "./components/AudioErrorDialog";
import { Sidebar, SidebarTab } from "./components/Sidebar";
import { AppShell } from "./components/layout/AppShell";
import { UIToggle } from "./components/dev";
import { useTranscription } from "./hooks/useTranscription";
import { useCatOverlay } from "./hooks/useCatOverlay";
import { useAutoStartListening } from "./hooks/useAutoStartListening";
import { useAudioErrorHandler } from "./hooks/useAudioErrorHandler";
import { useRecording } from "./hooks/useRecording";
import { useSettings } from "./hooks/useSettings";
import { useUIMode } from "./hooks/useUIMode";

function App() {
  const { settings } = useSettings();
  const { isTranscribing } = useTranscription();
  const { error: audioError, clearError } = useAudioErrorHandler();
  const { startRecording } = useRecording({
    deviceName: settings.audio.selectedDevice,
  });
  const [sidebarTab, setSidebarTab] = useState<SidebarTab>("history");
  const [navItem, setNavItem] = useState("dashboard");
  const { mode } = useUIMode();
  useCatOverlay();
  useAutoStartListening();

  const handleRetry = useCallback(() => {
    clearError();
    startRecording();
  }, [clearError, startRecording]);

  const handleSelectDevice = useCallback(() => {
    clearError();
    // Navigate to the Listening tab where device selection is available
    setSidebarTab("listening");
  }, [clearError]);

  // New UI mode - render AppShell layout
  if (mode === "new") {
    return (
      <>
        <AppShell
          activeNavItem={navItem}
          onNavigate={setNavItem}
          status="idle"
          footerStateDescription="Ready for your command."
        >
          <div className="flex items-center justify-center h-full text-text-secondary">
            <p>New UI - Page content coming soon</p>
          </div>
        </AppShell>
        <UIToggle />
      </>
    );
  }

  // Old UI mode - render existing Sidebar-based layout
  return (
    <>
      <div className="app-layout">
        <Sidebar className="app-sidebar" activeTab={sidebarTab} onTabChange={setSidebarTab} />
        <main className="container">
          <RecordingIndicator className="app-recording-indicator" isBlocked={isTranscribing} />
          <TranscriptionIndicator className="app-transcription-indicator" />
          <TranscriptionNotification className="app-transcription-notification" />
        </main>
        <AudioErrorDialog
          error={audioError}
          onRetry={handleRetry}
          onSelectDevice={handleSelectDevice}
          onDismiss={clearError}
        />
      </div>
      <UIToggle />
    </>
  );
}

export default App;
