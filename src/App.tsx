/* v8 ignore file -- @preserve */
import { useState, useCallback } from "react";
import "./App.css";
import { RecordingIndicator } from "./components/RecordingIndicator";
import { TranscriptionIndicator } from "./components/TranscriptionIndicator";
import { TranscriptionNotification } from "./components/TranscriptionNotification";
import { AudioErrorDialog } from "./components/AudioErrorDialog";
import { Sidebar, SidebarTab } from "./components/Sidebar";
import { useTranscription } from "./hooks/useTranscription";
import { useCatOverlay } from "./hooks/useCatOverlay";
import { useAutoStartListening } from "./hooks/useAutoStartListening";
import { useAudioErrorHandler } from "./hooks/useAudioErrorHandler";
import { useRecording } from "./hooks/useRecording";
import { useSettings } from "./hooks/useSettings";

function App() {
  const { settings } = useSettings();
  const { isTranscribing } = useTranscription();
  const { error: audioError, clearError } = useAudioErrorHandler();
  const { startRecording } = useRecording({
    deviceName: settings.audio.selectedDevice,
  });
  const [sidebarTab, setSidebarTab] = useState<SidebarTab>("history");
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

  return (
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
  );
}

export default App;
