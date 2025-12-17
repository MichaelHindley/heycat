import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { useTranscription } from "./useTranscription";

// Mock Tauri APIs
const mockListen = vi.fn();
const mockUnlisten = vi.fn();

vi.mock("@tauri-apps/api/event", () => ({
  listen: (...args: unknown[]) => mockListen(...args),
}));

describe("useTranscription", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockListen.mockResolvedValue(mockUnlisten);
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("user sees transcription progress and result when transcription completes", async () => {
    let startedCallback: ((event: { payload: { timestamp: string } }) => void) | null = null;
    let completedCallback: ((event: {
      payload: { text: string; duration_ms: number };
    }) => void) | null = null;

    mockListen.mockImplementation(
      (
        eventName: string,
        callback: (event: { payload: unknown }) => void
      ) => {
        if (eventName === "transcription_started") {
          startedCallback = callback;
        } else if (eventName === "transcription_completed") {
          completedCallback = callback;
        }
        return Promise.resolve(mockUnlisten);
      }
    );

    const { result } = renderHook(() => useTranscription());

    await waitFor(() => {
      expect(startedCallback).not.toBeNull();
      expect(completedCallback).not.toBeNull();
    });

    // Transcription starts
    act(() => {
      startedCallback!({ payload: { timestamp: "2025-01-01T12:00:00Z" } });
    });
    expect(result.current.isTranscribing).toBe(true);

    // Transcription completes
    act(() => {
      completedCallback!({
        payload: { text: "Hello, world!", duration_ms: 1234 },
      });
    });

    expect(result.current.isTranscribing).toBe(false);
    expect(result.current.transcribedText).toBe("Hello, world!");
    expect(result.current.durationMs).toBe(1234);
    expect(result.current.error).toBeNull();
  });

  it("user sees error when transcription fails", async () => {
    let startedCallback: ((event: { payload: { timestamp: string } }) => void) | null = null;
    let errorCallback: ((event: { payload: { error: string } }) => void) | null = null;

    mockListen.mockImplementation(
      (
        eventName: string,
        callback: (event: { payload: unknown }) => void
      ) => {
        if (eventName === "transcription_started") {
          startedCallback = callback;
        } else if (eventName === "transcription_error") {
          errorCallback = callback;
        }
        return Promise.resolve(mockUnlisten);
      }
    );

    const { result } = renderHook(() => useTranscription());

    await waitFor(() => {
      expect(startedCallback).not.toBeNull();
      expect(errorCallback).not.toBeNull();
    });

    // Start transcription
    act(() => {
      startedCallback!({ payload: { timestamp: "2025-01-01T12:00:00Z" } });
    });
    expect(result.current.isTranscribing).toBe(true);

    // Error during transcription
    act(() => {
      errorCallback!({ payload: { error: "Model not loaded" } });
    });

    expect(result.current.isTranscribing).toBe(false);
    expect(result.current.error).toBe("Model not loaded");
  });

  it("user can retry after timeout error", async () => {
    let startedCallback: ((event: { payload: { timestamp: string } }) => void) | null = null;
    let errorCallback: ((event: { payload: { error: string } }) => void) | null = null;
    let completedCallback: ((event: {
      payload: { text: string; duration_ms: number };
    }) => void) | null = null;

    mockListen.mockImplementation(
      (
        eventName: string,
        callback: (event: { payload: unknown }) => void
      ) => {
        if (eventName === "transcription_started") {
          startedCallback = callback;
        } else if (eventName === "transcription_error") {
          errorCallback = callback;
        } else if (eventName === "transcription_completed") {
          completedCallback = callback;
        }
        return Promise.resolve(mockUnlisten);
      }
    );

    const { result } = renderHook(() => useTranscription());

    await waitFor(() => {
      expect(startedCallback).not.toBeNull();
      expect(errorCallback).not.toBeNull();
      expect(completedCallback).not.toBeNull();
    });

    // First attempt times out
    act(() => {
      startedCallback!({ payload: { timestamp: "2025-01-01T12:00:00Z" } });
    });
    expect(result.current.isTranscribing).toBe(true);

    act(() => {
      errorCallback!({ payload: { error: "Transcription timed out" } });
    });

    expect(result.current.isTranscribing).toBe(false);
    expect(result.current.error).toBe("Transcription timed out");

    // Retry succeeds
    act(() => {
      startedCallback!({ payload: { timestamp: "2025-01-01T12:01:00Z" } });
    });
    expect(result.current.isTranscribing).toBe(true);
    expect(result.current.error).toBeNull(); // Error cleared on retry

    act(() => {
      completedCallback!({
        payload: { text: "Recovery transcription", duration_ms: 500 },
      });
    });
    expect(result.current.isTranscribing).toBe(false);
    expect(result.current.transcribedText).toBe("Recovery transcription");
  });

  it("previous transcription result clears when new transcription starts", async () => {
    let startedCallback: ((event: { payload: { timestamp: string } }) => void) | null = null;
    let completedCallback: ((event: {
      payload: { text: string; duration_ms: number };
    }) => void) | null = null;

    mockListen.mockImplementation(
      (
        eventName: string,
        callback: (event: { payload: unknown }) => void
      ) => {
        if (eventName === "transcription_started") {
          startedCallback = callback;
        } else if (eventName === "transcription_completed") {
          completedCallback = callback;
        }
        return Promise.resolve(mockUnlisten);
      }
    );

    const { result } = renderHook(() => useTranscription());

    await waitFor(() => {
      expect(startedCallback).not.toBeNull();
      expect(completedCallback).not.toBeNull();
    });

    // Complete first transcription
    act(() => {
      startedCallback!({ payload: { timestamp: "2025-01-01T12:00:00Z" } });
    });
    act(() => {
      completedCallback!({
        payload: { text: "First transcription", duration_ms: 1000 },
      });
    });
    expect(result.current.transcribedText).toBe("First transcription");
    expect(result.current.durationMs).toBe(1000);

    // Start second transcription - previous result should clear
    act(() => {
      startedCallback!({ payload: { timestamp: "2025-01-01T12:01:00Z" } });
    });

    expect(result.current.isTranscribing).toBe(true);
    expect(result.current.transcribedText).toBeNull();
    expect(result.current.durationMs).toBeNull();
  });
});
