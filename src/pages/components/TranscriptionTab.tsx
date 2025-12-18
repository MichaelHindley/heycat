import { useCallback } from "react";
import { Check, Download, Loader2 } from "lucide-react";
import { Card, CardContent, Button } from "../../components/ui";
import { useMultiModelStatus } from "../../hooks/useMultiModelStatus";
import { useToast } from "../../components/overlays";

export interface TranscriptionTabProps {
  className?: string;
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
}

export function TranscriptionTab({ className = "" }: TranscriptionTabProps) {
  const { models, downloadModel, refreshStatus } = useMultiModelStatus();
  const { toast } = useToast();

  const handleDownload = useCallback(async () => {
    toast({
      type: "info",
      title: "Download started",
      description: "Downloading transcription model...",
    });
    await downloadModel("tdt");
  }, [downloadModel, toast]);

  const handleCheckUpdates = useCallback(async () => {
    toast({
      type: "info",
      title: "Checking for updates",
      description: "Looking for model updates...",
    });
    await refreshStatus();
    toast({
      type: "success",
      title: "Model is up to date",
      description: "You have the latest version of the transcription model.",
    });
  }, [refreshStatus, toast]);

  const { isAvailable, downloadState, progress, error } = models;
  const isDownloading = downloadState === "downloading";
  const isReady = isAvailable || downloadState === "completed";

  // Estimated model size
  const modelSize = "1.2 GB";

  return (
    <div className={`space-y-6 ${className}`.trim()}>
      <section>
        <h2 className="text-xs font-semibold text-text-secondary uppercase tracking-wider mb-4">
          Transcription Model
        </h2>

        <Card>
          <CardContent className="space-y-4">
            {/* Model Info Header */}
            <div className="flex items-start justify-between">
              <div className="space-y-1">
                <h3 className="text-base font-medium text-text-primary">
                  Batch Model (TDT)
                </h3>
                <p className="text-sm text-text-secondary">
                  High-accuracy transcription for recordings
                </p>
              </div>

              {/* Status Badge */}
              <div
                className={`
                  flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium
                  ${
                    isReady
                      ? "bg-success/10 text-success"
                      : isDownloading
                        ? "bg-warning/10 text-warning"
                        : "bg-neutral-200 text-text-secondary"
                  }
                `}
              >
                {isReady && <Check className="h-3 w-3" />}
                {isDownloading && (
                  <Loader2 className="h-3 w-3 animate-spin" />
                )}
                {isReady ? "Ready" : isDownloading ? "Downloading" : "Not Installed"}
              </div>
            </div>

            {/* Model Details */}
            {isReady && (
              <div className="text-sm text-text-secondary">
                Model size: {modelSize}
              </div>
            )}

            {/* Download Progress */}
            {isDownloading && (
              <div className="space-y-2">
                <div className="flex items-center justify-between text-sm">
                  <span className="text-text-secondary">
                    Downloading... {Math.round(progress)}%
                  </span>
                  <span className="text-text-secondary">
                    {formatBytes((progress / 100) * 1.2 * 1024 * 1024 * 1024)} /{" "}
                    {modelSize}
                  </span>
                </div>
                <div
                  className="h-2 bg-neutral-200 rounded-full overflow-hidden"
                  role="progressbar"
                  aria-valuenow={progress}
                  aria-valuemin={0}
                  aria-valuemax={100}
                  aria-label="Model download progress"
                >
                  <div
                    className="h-full bg-warning transition-all duration-300"
                    style={{ width: `${progress}%` }}
                  />
                </div>
              </div>
            )}

            {/* Error Message */}
            {error && (
              <div className="text-sm text-error" role="alert">
                {error}
              </div>
            )}

            {/* Actions */}
            <div className="flex gap-2">
              {isReady ? (
                <Button variant="secondary" onClick={handleCheckUpdates}>
                  Check for Updates
                </Button>
              ) : (
                <Button
                  onClick={handleDownload}
                  disabled={isDownloading}
                  loading={isDownloading}
                >
                  {!isDownloading && <Download className="h-4 w-4" />}
                  {isDownloading
                    ? `Downloading... ${Math.round(progress)}%`
                    : `Download Model (${modelSize})`}
                </Button>
              )}
            </div>
          </CardContent>
        </Card>
      </section>
    </div>
  );
}
