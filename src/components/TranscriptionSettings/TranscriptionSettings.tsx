import { useCallback } from "react";
import { useMultiModelStatus } from "../../hooks/useMultiModelStatus";
import { ModelDownloadCard } from "./ModelDownloadCard";
import "./TranscriptionSettings.css";

export interface TranscriptionSettingsProps {
  className?: string;
}

export function TranscriptionSettings({
  className = "",
}: TranscriptionSettingsProps) {
  const { models, downloadModel } = useMultiModelStatus();

  const handleDownloadTdt = useCallback(() => {
    downloadModel("tdt");
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
            status={models}
            onDownload={handleDownloadTdt}
          />
        </div>
      </section>
    </div>
  );
}
