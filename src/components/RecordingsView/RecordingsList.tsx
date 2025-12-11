import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { EmptyState } from "./EmptyState";
import "./RecordingsList.css";

export interface RecordingInfo {
  filename: string;
  file_path: string;
  duration_secs: number;
  created_at: string;
  file_size_bytes: number;
}

export interface RecordingsListProps {
  className?: string;
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
    hour: "2-digit",
    minute: "2-digit",
  });
}

export function RecordingsList({ className = "" }: RecordingsListProps) {
  const [recordings, setRecordings] = useState<RecordingInfo[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    async function fetchRecordings() {
      try {
        setIsLoading(true);
        setError(null);
        const result = await invoke<RecordingInfo[]>("list_recordings");
        setRecordings(result);
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
      } finally {
        setIsLoading(false);
      }
    }

    fetchRecordings();
  }, []);

  if (isLoading) {
    return (
      <div
        className={`recordings-list recordings-list--loading ${className}`.trim()}
        role="status"
        aria-busy="true"
        aria-label="Loading recordings"
      >
        <span className="recordings-list__loading-text">
          Loading recordings...
        </span>
      </div>
    );
  }

  if (error) {
    return (
      <div
        className={`recordings-list recordings-list--error ${className}`.trim()}
        role="alert"
      >
        <span className="recordings-list__error-text">
          Failed to load recordings: {error}
        </span>
      </div>
    );
  }

  if (recordings.length === 0) {
    return <EmptyState hasFiltersActive={false} className={className} />;
  }

  return (
    <div className={`recordings-list ${className}`.trim()}>
      <ul className="recordings-list__items" role="list">
        {recordings.map((recording) => (
          <li key={recording.file_path} className="recordings-list__item">
            <span className="recordings-list__filename">{recording.filename}</span>
            <span className="recordings-list__duration">
              {formatDuration(recording.duration_secs)}
            </span>
            <span className="recordings-list__date">
              {formatDate(recording.created_at)}
            </span>
          </li>
        ))}
      </ul>
    </div>
  );
}
