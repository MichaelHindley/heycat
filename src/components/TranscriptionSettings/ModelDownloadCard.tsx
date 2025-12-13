import { ModelStatus, DownloadState, ModelType } from "../../hooks/useMultiModelStatus";

export interface ModelDownloadCardProps {
  modelType: ModelType;
  title: string;
  description: string;
  status: ModelStatus;
  onDownload: () => void;
}

function getButtonText(state: DownloadState, isAvailable: boolean, progress: number): string {
  if (isAvailable || state === "completed") {
    return "Model Ready";
  }
  if (state === "downloading") {
    return `Downloading... ${Math.round(progress)}%`;
  }
  if (state === "error") {
    return "Retry Download";
  }
  return "Download Model";
}

function getAriaLabel(state: DownloadState, isAvailable: boolean, title: string): string {
  if (isAvailable || state === "completed") {
    return `${title} model is ready`;
  }
  if (state === "downloading") {
    return `Downloading ${title} model, please wait`;
  }
  if (state === "error") {
    return `${title} download failed, click to retry`;
  }
  return `Click to download ${title} model`;
}

export function ModelDownloadCard({
  modelType,
  title,
  description,
  status,
  onDownload,
}: ModelDownloadCardProps) {
  const { isAvailable, downloadState, progress, error } = status;

  const isDisabled =
    downloadState === "downloading" ||
    downloadState === "completed" ||
    isAvailable;

  const buttonText = getButtonText(downloadState, isAvailable, progress);
  const ariaLabel = getAriaLabel(downloadState, isAvailable, title);

  const stateClass =
    downloadState === "completed" || isAvailable
      ? "transcription-settings__model-card--ready"
      : downloadState === "downloading"
        ? "transcription-settings__model-card--downloading"
        : downloadState === "error"
          ? "transcription-settings__model-card--error"
          : "";

  return (
    <div
      className={`transcription-settings__model-card ${stateClass}`.trim()}
      data-model-type={modelType}
    >
      <div className="transcription-settings__model-info">
        <h3 className="transcription-settings__model-title">{title}</h3>
        <p className="transcription-settings__model-description">{description}</p>
      </div>

      <div className="transcription-settings__model-actions">
        <button
          className="transcription-settings__download-button"
          onClick={onDownload}
          disabled={isDisabled}
          aria-label={ariaLabel}
          aria-busy={downloadState === "downloading"}
          type="button"
        >
          {downloadState === "downloading" && (
            <span
              className="transcription-settings__spinner"
              aria-hidden="true"
            />
          )}
          {buttonText}
        </button>

        {downloadState === "downloading" && (
          <div
            className="transcription-settings__progress-bar"
            role="progressbar"
            aria-valuenow={progress}
            aria-valuemin={0}
            aria-valuemax={100}
            aria-label={`${title} download progress`}
          >
            <div
              className="transcription-settings__progress-fill"
              style={{ width: `${progress}%` }}
            />
          </div>
        )}

        {error && (
          <span className="transcription-settings__error" role="alert">
            {error}
          </span>
        )}
      </div>
    </div>
  );
}
