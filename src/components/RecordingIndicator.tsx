/* v8 ignore file -- @preserve */
import { useRecording } from "../hooks/useRecording";
import { useSettings } from "../hooks/useSettings";
import "./RecordingIndicator.css";

export interface RecordingIndicatorProps {
  className?: string;
  /** When true, recording is blocked (e.g., during transcription) */
  isBlocked?: boolean;
}

export function RecordingIndicator({
  className = "",
  isBlocked = false,
}: RecordingIndicatorProps) {
  const { settings } = useSettings();
  const { isRecording, error, wasCancelled } = useRecording({
    deviceName: settings.audio.selectedDevice,
  });

  const stateClass = isBlocked
    ? "recording-indicator--blocked"
    : isRecording
      ? "recording-indicator--recording"
      : wasCancelled
        ? "recording-indicator--cancelled"
        : "recording-indicator--idle";
  const statusText = isBlocked
    ? "Recording blocked"
    : isRecording
      ? "Recording"
      : wasCancelled
        ? "Cancelled"
        : "Idle";

  return (
    <div
      className={`recording-indicator ${stateClass} ${className}`.trim()}
      role="status"
      aria-live="polite"
      aria-label={`Recording status: ${statusText}`}
    >
      <span className="recording-indicator__dot" aria-hidden="true" />
      <span className="recording-indicator__label">{statusText}</span>
      {error && (
        <span className="recording-indicator__error" role="alert">
          {error}
        </span>
      )}
    </div>
  );
}
