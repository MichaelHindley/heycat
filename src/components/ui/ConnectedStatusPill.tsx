import { type HTMLAttributes, forwardRef, useState, useEffect } from "react";
import { StatusPill, type StatusPillProps } from "./StatusPill";
import { useAppStatus } from "../../hooks/useAppStatus";

export interface ConnectedStatusPillProps
  extends Omit<HTMLAttributes<HTMLDivElement>, "children"> {
  /** Custom label override (uses auto-derived label if not provided) */
  label?: string;
}

/**
 * StatusPill connected to app state hooks.
 * Automatically derives status from useRecording, useTranscription, and useListening hooks.
 * Includes auto-incrementing duration timer when recording.
 */
export const ConnectedStatusPill = forwardRef<
  HTMLDivElement,
  ConnectedStatusPillProps
>(({ label, ...props }, ref) => {
  const { status, isRecording } = useAppStatus();
  const [duration, setDuration] = useState(0);

  // Auto-increment duration while recording
  useEffect(() => {
    if (!isRecording) {
      setDuration(0);
      return;
    }

    const interval = setInterval(() => {
      setDuration((prev) => prev + 1);
    }, 1000);

    return () => clearInterval(interval);
  }, [isRecording]);

  return (
    <StatusPill
      ref={ref}
      status={status}
      label={label}
      recordingDuration={isRecording ? duration : undefined}
      {...props}
    />
  );
});

ConnectedStatusPill.displayName = "ConnectedStatusPill";
