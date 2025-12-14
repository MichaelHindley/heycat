/* v8 ignore file -- @preserve */
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import App from "./App";
import * as useRecordingModule from "./hooks/useRecording";
import * as useTranscriptionModule from "./hooks/useTranscription";
import * as useCatOverlayModule from "./hooks/useCatOverlay";

vi.mock("./hooks/useRecording");
vi.mock("./hooks/useCatOverlay");
vi.mock("./hooks/useTranscription");
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue([]),
}));

const mockUseRecording = vi.mocked(useRecordingModule.useRecording);
const mockUseTranscription = vi.mocked(useTranscriptionModule.useTranscription);
const mockUseCatOverlay = vi.mocked(useCatOverlayModule.useCatOverlay);

describe("App Integration", () => {
  const defaultRecordingMock: useRecordingModule.UseRecordingResult = {
    isRecording: false,
    error: null,
    startRecording: vi.fn(),
    stopRecording: vi.fn(),
    lastRecording: null,
  };

  const defaultTranscriptionMock: useTranscriptionModule.UseTranscriptionResult = {
    isTranscribing: false,
    transcribedText: null,
    error: null,
    durationMs: null,
  };

  beforeEach(() => {
    vi.clearAllMocks();
    mockUseRecording.mockReturnValue(defaultRecordingMock);
    mockUseTranscription.mockReturnValue(defaultTranscriptionMock);
    mockUseCatOverlay.mockReturnValue({ isRecording: false });
  });

  it("renders RecordingIndicator component without errors", async () => {
    render(<App />);

    const indicator = document.querySelector(".recording-indicator");
    expect(indicator).not.toBeNull();
    expect(screen.getByText("Idle")).toBeDefined();
    // Wait for RecordingsList async effect to complete
    await waitFor(() => {
      expect(screen.getByText("No recordings yet")).toBeDefined();
    });
  });

  it("syncs state when backend emits recording events", async () => {
    const { rerender } = render(<App />);

    expect(screen.getByText("Idle")).toBeDefined();

    mockUseRecording.mockReturnValue({
      ...defaultRecordingMock,
      isRecording: true,
    });

    rerender(<App />);

    expect(screen.getByText("Recording")).toBeDefined();
    // Wait for RecordingsList async effect to complete
    await waitFor(() => {
      expect(screen.getByText("No recordings yet")).toBeDefined();
    });
  });

  it("App renders without console errors", async () => {
    const consoleSpy = vi.spyOn(console, "error").mockImplementation(() => {});

    render(<App />);

    // Wait for RecordingsList async effect to complete
    await waitFor(() => {
      expect(screen.getByText("No recordings yet")).toBeDefined();
    });

    expect(consoleSpy).not.toHaveBeenCalled();
    consoleSpy.mockRestore();
  });
});
