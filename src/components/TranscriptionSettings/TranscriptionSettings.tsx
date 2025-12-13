import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useMultiModelStatus, ModelType } from "../../hooks/useMultiModelStatus";
import { ModelDownloadCard } from "./ModelDownloadCard";
import { ModeToggle, TranscriptionMode } from "./ModeToggle";
import "./TranscriptionSettings.css";

export interface TranscriptionSettingsProps {
  className?: string;
}

export function TranscriptionSettings({
  className = "",
}: TranscriptionSettingsProps) {
  const { models, downloadModel } = useMultiModelStatus();
  const [currentMode, setCurrentMode] = useState<TranscriptionMode>("batch");
  const [isLoadingMode, setIsLoadingMode] = useState(true);
  const [modeError, setModeError] = useState<string | null>(null);

  // Load current mode on mount
  useEffect(() => {
    /* v8 ignore start -- @preserve */
    const loadMode = async () => {
      try {
        const mode = await invoke<string>("get_transcription_mode");
        if (mode === "batch" || mode === "streaming") {
          setCurrentMode(mode);
        }
      } catch (e) {
        setModeError(e instanceof Error ? e.message : String(e));
      } finally {
        setIsLoadingMode(false);
      }
    };
    loadMode();
    /* v8 ignore stop */
  }, []);

  const handleModeChange = useCallback(async (mode: TranscriptionMode) => {
    /* v8 ignore start -- @preserve */
    try {
      setModeError(null);
      await invoke("set_transcription_mode", { mode });
      setCurrentMode(mode);
    } catch (e) {
      setModeError(e instanceof Error ? e.message : String(e));
    }
    /* v8 ignore stop */
  }, []);

  const handleDownloadTdt = useCallback(() => {
    downloadModel("tdt");
  }, [downloadModel]);

  const handleDownloadEou = useCallback(() => {
    downloadModel("eou");
  }, [downloadModel]);

  return (
    <div
      className={`transcription-settings ${className}`.trim()}
      role="region"
      aria-label="Transcription settings"
    >
      <div className="transcription-settings__header">
        <h2 className="transcription-settings__title">Transcription</h2>
      </div>

      <section className="transcription-settings__section">
        <h3 className="transcription-settings__section-title">Models</h3>
        <div className="transcription-settings__models">
          <ModelDownloadCard
            modelType="tdt"
            title="Batch (TDT)"
            description="High-accuracy transcription after recording completes"
            status={models.tdt}
            onDownload={handleDownloadTdt}
          />
          <ModelDownloadCard
            modelType="eou"
            title="Streaming (EOU)"
            description="Real-time transcription as you speak"
            status={models.eou}
            onDownload={handleDownloadEou}
          />
        </div>
      </section>

      <section className="transcription-settings__section">
        <h3 className="transcription-settings__section-title">Mode</h3>
        <ModeToggle
          currentMode={currentMode}
          tdtAvailable={models.tdt.isAvailable}
          eouAvailable={models.eou.isAvailable}
          onChange={handleModeChange}
          disabled={isLoadingMode}
        />
        {modeError && (
          <span className="transcription-settings__error" role="alert">
            {modeError}
          </span>
        )}
      </section>
    </div>
  );
}
