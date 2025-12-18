import { Play, Pause, Copy, FolderOpen, Trash2, ChevronDown, ChevronUp } from "lucide-react";
import { Card, CardContent, Button } from "../../components/ui";

export interface RecordingInfo {
  filename: string;
  file_path: string;
  duration_secs: number;
  created_at: string;
  file_size_bytes: number;
  error?: string;
  transcription?: string;
}

export interface RecordingItemProps {
  recording: RecordingInfo;
  isExpanded: boolean;
  onToggleExpand: () => void;
  onPlay: () => void;
  onTranscribe: () => void;
  onCopyText: () => void;
  onOpenFile: () => void;
  onDelete: () => void;
  isPlaying?: boolean;
  isTranscribing?: boolean;
  isDeleting?: boolean;
  onConfirmDelete?: () => void;
  onCancelDelete?: () => void;
}

export function formatDuration(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins}:${secs.toString().padStart(2, "0")}`;
}

export function formatDate(isoString: string): string {
  const date = new Date(isoString);
  return date.toLocaleDateString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
  });
}

export function formatFileSize(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  const k = 1024;
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  const value = bytes / Math.pow(k, i);
  return `${value.toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
}

export function RecordingItem({
  recording,
  isExpanded,
  onToggleExpand,
  onPlay,
  onTranscribe,
  onCopyText,
  onOpenFile,
  onDelete,
  isPlaying = false,
  isTranscribing = false,
  isDeleting = false,
  onConfirmDelete,
  onCancelDelete,
}: RecordingItemProps) {
  const hasTranscription = Boolean(recording.transcription);
  const hasError = Boolean(recording.error);

  return (
    <Card
      className={`transition-all ${hasError ? "border-error/50" : ""}`}
      role="listitem"
    >
      <CardContent className="p-0">
        {/* Collapsed Row - Always visible */}
        <div className="flex items-center gap-3 p-4 hover:bg-surface-hover transition-colors">
          {/* Play/Pause Button */}
          <button
            type="button"
            className="flex items-center justify-center w-10 h-10 rounded-full bg-heycat-orange/20 text-heycat-orange hover:bg-heycat-orange hover:text-white transition-colors flex-shrink-0"
            onClick={onPlay}
            aria-label={isPlaying ? `Pause ${recording.filename}` : `Play ${recording.filename}`}
            disabled={hasError}
          >
            {isPlaying ? (
              <Pause className="h-4 w-4" />
            ) : (
              <Play className="h-4 w-4 ml-0.5" />
            )}
          </button>

          {/* Filename and Metadata - Clickable to expand */}
          <button
            type="button"
            className="flex-1 min-w-0 text-left bg-transparent border-none cursor-pointer p-0"
            onClick={onToggleExpand}
            aria-expanded={isExpanded}
            aria-label={`${isExpanded ? "Collapse" : "Expand"} ${recording.filename}`}
          >
            <span className="text-sm font-medium text-text-primary truncate block">
              {recording.filename}
            </span>
            <span className="text-xs text-text-secondary">
              {formatDate(recording.created_at)} • {formatDuration(recording.duration_secs)} • {formatFileSize(recording.file_size_bytes)}
            </span>
          </button>

          {/* Status Badge */}
          {hasError ? (
            <span className="px-2 py-1 text-xs font-medium rounded-full bg-error/10 text-error flex-shrink-0">
              Error
            </span>
          ) : hasTranscription ? (
            <span className="px-2 py-1 text-xs font-medium rounded-full bg-success/10 text-success flex-shrink-0">
              Transcribed
            </span>
          ) : (
            <Button
              variant="secondary"
              size="sm"
              onClick={onTranscribe}
              loading={isTranscribing}
              disabled={isTranscribing}
              className="flex-shrink-0"
            >
              Transcribe
            </Button>
          )}

          {/* Expand/Collapse Button */}
          <button
            type="button"
            className="text-text-secondary flex-shrink-0 p-1 hover:text-text-primary transition-colors bg-transparent border-none cursor-pointer"
            onClick={onToggleExpand}
            aria-label={isExpanded ? "Collapse details" : "Expand details"}
          >
            {isExpanded ? (
              <ChevronUp className="h-4 w-4" />
            ) : (
              <ChevronDown className="h-4 w-4" />
            )}
          </button>
        </div>

        {/* Expanded Content */}
        {isExpanded && (
          <div className="border-t border-border p-4 space-y-4">
            {/* Transcription Section */}
            <div>
              <h4 className="text-xs font-semibold text-text-secondary uppercase tracking-wider mb-2">
                Transcription
              </h4>
              {hasTranscription ? (
                <div className="bg-background rounded-lg p-3 max-h-32 overflow-y-auto">
                  <p className="text-sm text-text-primary whitespace-pre-wrap">
                    {recording.transcription}
                  </p>
                </div>
              ) : (
                <p className="text-sm text-text-secondary italic">
                  No transcription available
                </p>
              )}
            </div>

            {/* Action Buttons */}
            <div className="flex items-center gap-2">
              {isDeleting ? (
                <>
                  <Button
                    variant="danger"
                    size="sm"
                    onClick={onConfirmDelete}
                    aria-label="Confirm delete"
                  >
                    Confirm Delete
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={onCancelDelete}
                    aria-label="Cancel delete"
                  >
                    Cancel
                  </Button>
                </>
              ) : (
                <>
                  <Button
                    variant="secondary"
                    size="sm"
                    onClick={onCopyText}
                    disabled={!hasTranscription}
                    aria-label="Copy transcription text"
                  >
                    <Copy className="h-4 w-4" />
                    Copy Text
                  </Button>
                  <Button
                    variant="secondary"
                    size="sm"
                    onClick={onOpenFile}
                    aria-label="Open file in system"
                  >
                    <FolderOpen className="h-4 w-4" />
                    Open File
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={onDelete}
                    aria-label={`Delete ${recording.filename}`}
                    className="ml-auto text-error hover:bg-error/10"
                  >
                    <Trash2 className="h-4 w-4" />
                    Delete
                  </Button>
                </>
              )}
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
