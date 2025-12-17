import {
  AudioDeviceError,
  AudioDeviceErrorType,
  getErrorMessage,
} from "../../types/audio";
import "./AudioErrorDialog.css";

/** Configuration for each error type */
interface ErrorConfig {
  title: string;
  description: string;
  actions: ("retry" | "selectDevice" | "openSettings")[];
}

const ERROR_CONFIG: Record<AudioDeviceErrorType, ErrorConfig> = {
  deviceNotFound: {
    title: "Microphone Not Found",
    description:
      "The selected microphone is not connected. Please connect it or choose a different device.",
    actions: ["selectDevice", "retry"],
  },
  noDevicesAvailable: {
    title: "No Microphone Detected",
    description:
      "No audio input devices were found. Please connect a microphone.",
    actions: ["retry"],
  },
  permissionDenied: {
    title: "Microphone Access Required",
    description:
      "heycat needs permission to access your microphone. Please grant access in System Preferences.",
    actions: ["openSettings", "retry"],
  },
  deviceDisconnected: {
    title: "Microphone Disconnected",
    description:
      "The microphone was disconnected during recording. Your recording has been saved.",
    actions: ["selectDevice", "retry"],
  },
  captureError: {
    title: "Recording Error",
    description: "An error occurred while recording. Please try again.",
    actions: ["retry"],
  },
};

export interface AudioErrorDialogProps {
  /** The error to display, or null to hide the dialog */
  error: AudioDeviceError | null;
  /** Called when the user clicks "Try Again" */
  onRetry: () => void;
  /** Called when the user clicks "Select Device" */
  onSelectDevice: () => void;
  /** Called when the user dismisses the dialog */
  onDismiss: () => void;
  /** Optional CSS class name */
  className?: string;
}

export function AudioErrorDialog({
  error,
  onRetry,
  onSelectDevice,
  onDismiss,
  className = "",
}: AudioErrorDialogProps) {
  if (!error) return null;

  const config = ERROR_CONFIG[error.type];
  const customMessage =
    error.type === "captureError" || error.type === "deviceNotFound"
      ? getErrorMessage(error)
      : config.description;

  const handleOpenSettings = () => {
    // On macOS, open System Preferences > Security & Privacy > Microphone
    // This requires a shell command or Tauri API - for now, provide instructions
    window.open(
      "x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone",
      "_blank"
    );
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") {
      onDismiss();
    }
  };

  return (
    <div
      className={`audio-error-dialog-overlay ${className}`.trim()}
      role="dialog"
      aria-modal="true"
      aria-labelledby="audio-error-title"
      onKeyDown={handleKeyDown}
    >
      <div className="audio-error-dialog">
        <div className="audio-error-dialog__icon" aria-hidden="true">
          <svg
            width="24"
            height="24"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <circle cx="12" cy="12" r="10" />
            <line x1="12" y1="8" x2="12" y2="12" />
            <line x1="12" y1="16" x2="12.01" y2="16" />
          </svg>
        </div>
        <h2 id="audio-error-title" className="audio-error-dialog__title">
          {config.title}
        </h2>
        <p className="audio-error-dialog__description">{customMessage}</p>

        <div className="audio-error-dialog__actions">
          {config.actions.includes("selectDevice") && (
            <button
              type="button"
              className="audio-error-dialog__button audio-error-dialog__button--primary"
              onClick={onSelectDevice}
            >
              Select Device
            </button>
          )}
          {config.actions.includes("openSettings") && (
            <button
              type="button"
              className="audio-error-dialog__button audio-error-dialog__button--primary"
              onClick={handleOpenSettings}
            >
              Open Settings
            </button>
          )}
          {config.actions.includes("retry") && (
            <button
              type="button"
              className="audio-error-dialog__button audio-error-dialog__button--secondary"
              onClick={onRetry}
            >
              Try Again
            </button>
          )}
          <button
            type="button"
            className="audio-error-dialog__button audio-error-dialog__button--ghost"
            onClick={onDismiss}
          >
            Dismiss
          </button>
        </div>
      </div>
    </div>
  );
}
