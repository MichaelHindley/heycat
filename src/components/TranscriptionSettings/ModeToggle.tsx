export type TranscriptionMode = "batch" | "streaming";

export interface ModeToggleProps {
  currentMode: TranscriptionMode;
  tdtAvailable: boolean;
  eouAvailable: boolean;
  onChange: (mode: TranscriptionMode) => void;
  disabled?: boolean;
}

export function ModeToggle({
  currentMode,
  tdtAvailable,
  eouAvailable,
  onChange,
  disabled = false,
}: ModeToggleProps) {
  const isBatchDisabled = disabled || !tdtAvailable;
  const isStreamingDisabled = disabled || !eouAvailable;

  return (
    <div
      className="transcription-settings__mode-toggle"
      role="radiogroup"
      aria-label="Transcription mode"
    >
      <label
        className={`transcription-settings__mode-option ${currentMode === "batch" ? "transcription-settings__mode-option--selected" : ""} ${isBatchDisabled ? "transcription-settings__mode-option--disabled" : ""}`.trim()}
      >
        <input
          type="radio"
          name="transcription-mode"
          value="batch"
          checked={currentMode === "batch"}
          onChange={() => onChange("batch")}
          disabled={isBatchDisabled}
          aria-describedby="batch-mode-description"
        />
        <span className="transcription-settings__mode-label">Batch</span>
        <span
          id="batch-mode-description"
          className="transcription-settings__mode-description"
        >
          {tdtAvailable
            ? "Process after recording"
            : "TDT model required"}
        </span>
      </label>

      <label
        className={`transcription-settings__mode-option ${currentMode === "streaming" ? "transcription-settings__mode-option--selected" : ""} ${isStreamingDisabled ? "transcription-settings__mode-option--disabled" : ""}`.trim()}
      >
        <input
          type="radio"
          name="transcription-mode"
          value="streaming"
          checked={currentMode === "streaming"}
          onChange={() => onChange("streaming")}
          disabled={isStreamingDisabled}
          aria-describedby="streaming-mode-description"
        />
        <span className="transcription-settings__mode-label">Streaming</span>
        <span
          id="streaming-mode-description"
          className="transcription-settings__mode-description"
        >
          {eouAvailable
            ? "Real-time transcription"
            : "EOU model required"}
        </span>
      </label>
    </div>
  );
}
