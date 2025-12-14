import { useEffect, useRef, useState } from "react";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { primaryMonitor, LogicalPosition } from "@tauri-apps/api/window";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { useRecording } from "./useRecording";

const OVERLAY_LABEL = "cat-overlay";
const OVERLAY_SIZE = 120;

/** Overlay visual mode based on app state */
export type OverlayMode = "hidden" | "listening" | "recording";

/** Payload for listening_started event */
interface ListeningStartedPayload {
  timestamp: string;
}

/** Payload for listening_stopped event */
interface ListeningStoppedPayload {
  timestamp: string;
}

/** Payload for wake_word_detected event */
interface WakeWordDetectedPayload {
  confidence: number;
  transcription: string;
  timestamp: string;
}

/** Payload for listening_unavailable event */
interface ListeningUnavailablePayload {
  reason: string;
  timestamp: string;
}

function getOverlayUrl(): string {
  if (import.meta.env.DEV) {
    return "http://localhost:1420/overlay.html";
  }
  return "/overlay.html";
}

async function calculateOverlayPosition(): Promise<{ x: number; y: number } | null> {
  const monitor = await primaryMonitor();
  if (!monitor) return null;

  const monitorPosition = monitor.position;
  const monitorSize = monitor.size;
  const scale = monitor.scaleFactor;

  const logicalWidth = monitorSize.width / scale;
  const logicalHeight = monitorSize.height / scale;
  const logicalX = monitorPosition.x / scale;
  const logicalY = monitorPosition.y / scale;

  return {
    x: Math.round(logicalX + (logicalWidth - OVERLAY_SIZE) / 2),
    y: Math.round(logicalY + logicalHeight - OVERLAY_SIZE - 50),
  };
}

export function useCatOverlay() {
  const { isRecording } = useRecording();
  const [isListening, setIsListening] = useState(false);
  const [isMicUnavailable, setIsMicUnavailable] = useState(false);
  const initializedRef = useRef(false);

  // Determine the overlay mode based on state
  const overlayMode: OverlayMode = isRecording
    ? "recording"
    : isListening
      ? "listening"
      : "hidden";

  // Subscribe to listening events
  useEffect(() => {
    const unlistenFns: UnlistenFn[] = [];

    /* v8 ignore start -- @preserve */
    const setupListeners = async () => {
      const unlistenStarted = await listen<ListeningStartedPayload>(
        "listening_started",
        () => {
          setIsListening(true);
          setIsMicUnavailable(false);
        }
      );
      unlistenFns.push(unlistenStarted);

      const unlistenStopped = await listen<ListeningStoppedPayload>(
        "listening_stopped",
        () => {
          setIsListening(false);
        }
      );
      unlistenFns.push(unlistenStopped);

      // Wake word detected - listening continues but recording will start
      // (handled by recording events, no state change needed here)
      const unlistenWakeWord = await listen<WakeWordDetectedPayload>(
        "wake_word_detected",
        () => {
          // Wake word detection is handled - recording_started will follow
        }
      );
      unlistenFns.push(unlistenWakeWord);

      const unlistenUnavailable = await listen<ListeningUnavailablePayload>(
        "listening_unavailable",
        () => {
          setIsMicUnavailable(true);
        }
      );
      unlistenFns.push(unlistenUnavailable);
    };

    setupListeners();
    /* v8 ignore stop */

    return () => {
      /* v8 ignore start -- @preserve */
      unlistenFns.forEach((unlisten) => unlisten());
      /* v8 ignore stop */
    };
  }, []);

  // Initialize overlay window once (hidden) on mount
  useEffect(() => {
    /* v8 ignore start -- @preserve */
    if (initializedRef.current) return;
    initializedRef.current = true;

    const initOverlay = async () => {
      const existing = await WebviewWindow.getByLabel(OVERLAY_LABEL);
      if (existing) return;

      const position = await calculateOverlayPosition();
      if (!position) return;

      const overlayWindow = new WebviewWindow(OVERLAY_LABEL, {
        url: getOverlayUrl(),
        width: OVERLAY_SIZE,
        height: OVERLAY_SIZE,
        x: position.x,
        y: position.y,
        transparent: true,
        decorations: false,
        alwaysOnTop: true,
        resizable: false,
        skipTaskbar: true,
        focus: false,
        visible: false,
      });

      overlayWindow.once("tauri://created", async () => {
        await overlayWindow.setIgnoreCursorEvents(true);
      });
    };

    initOverlay();
    /* v8 ignore stop */
  }, []);

  // Show/hide and update mode based on state
  useEffect(() => {
    /* v8 ignore start -- @preserve */
    const updateOverlay = async () => {
      const window = await WebviewWindow.getByLabel(OVERLAY_LABEL);
      if (!window) return;

      const shouldShow = overlayMode !== "hidden";

      if (shouldShow) {
        // Recalculate position in case monitor setup changed
        const position = await calculateOverlayPosition();
        if (position) {
          await window.setPosition(new LogicalPosition(position.x, position.y));
        }
        // Emit mode to overlay window for visual distinction
        await window.emit("overlay-mode", { mode: overlayMode, isMicUnavailable });
        await window.show();
      } else {
        await window.hide();
      }
    };

    updateOverlay();
    /* v8 ignore stop */
  }, [overlayMode, isMicUnavailable]);

  return { isRecording, isListening, overlayMode, isMicUnavailable };
}
