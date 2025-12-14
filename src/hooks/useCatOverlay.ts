import { useEffect, useRef } from "react";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { primaryMonitor, LogicalPosition } from "@tauri-apps/api/window";
import { useRecording } from "./useRecording";

const OVERLAY_LABEL = "cat-overlay";
const OVERLAY_SIZE = 120;

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
  const initializedRef = useRef(false);

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

  // Show/hide based on recording state
  useEffect(() => {
    /* v8 ignore start -- @preserve */
    const toggleOverlay = async () => {
      const window = await WebviewWindow.getByLabel(OVERLAY_LABEL);
      if (!window) return;

      if (isRecording) {
        // Recalculate position in case monitor setup changed
        const position = await calculateOverlayPosition();
        if (position) {
          await window.setPosition(new LogicalPosition(position.x, position.y));
        }
        await window.show();
      } else {
        await window.hide();
      }
    };

    toggleOverlay();
    /* v8 ignore stop */
  }, [isRecording]);

  return { isRecording };
}
