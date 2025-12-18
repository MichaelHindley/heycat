import { useMemo } from "react";
import { useRecording } from "./useRecording";
import { useTranscription } from "./useTranscription";
import { useListening } from "./useListening";
import type { StatusPillStatus } from "../components/ui/StatusPill";

export interface UseAppStatusResult {
  /** Current app status derived from all state hooks */
  status: StatusPillStatus;
  /** Whether recording is in progress */
  isRecording: boolean;
  /** Whether transcription is in progress */
  isTranscribing: boolean;
  /** Whether listening mode is active */
  isListening: boolean;
  /** Any error from the hooks */
  error: string | null;
}

/**
 * Derives the combined app status from recording, transcription, and listening hooks.
 * Priority order: recording > processing > listening > idle
 */
export function useAppStatus(): UseAppStatusResult {
  const { isRecording, error: recordingError } = useRecording();
  const { isTranscribing, error: transcriptionError } = useTranscription();
  const { isListening, error: listeningError } = useListening();

  const status = useMemo<StatusPillStatus>(() => {
    if (isRecording) return "recording";
    if (isTranscribing) return "processing";
    if (isListening) return "listening";
    return "idle";
  }, [isRecording, isTranscribing, isListening]);

  const error = recordingError ?? transcriptionError ?? listeningError ?? null;

  return {
    status,
    isRecording,
    isTranscribing,
    isListening,
    error,
  };
}
