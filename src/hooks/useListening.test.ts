import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { useListening } from "./useListening";

// Mock Tauri APIs
const mockInvoke = vi.fn();
const mockListen = vi.fn();
const mockUnlisten = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: (...args: unknown[]) => mockListen(...args),
}));

describe("useListening", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Default listen mock returns an unlisten function
    mockListen.mockResolvedValue(mockUnlisten);
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("initializes with correct default state", () => {
    const { result } = renderHook(() => useListening());

    expect(result.current.isListening).toBe(false);
    expect(result.current.isWakeWordDetected).toBe(false);
    expect(result.current.isMicAvailable).toBe(true);
    expect(result.current.error).toBeNull();
  });

  it("enableListening() calls Tauri command and updates state on event", async () => {
    let startedCallback: ((event: { payload: { timestamp: string } }) => void) | null = null;

    mockListen.mockImplementation(
      (
        eventName: string,
        callback: (event: { payload: unknown }) => void
      ) => {
        if (eventName === "listening_started") {
          startedCallback = callback;
        }
        return Promise.resolve(mockUnlisten);
      }
    );

    mockInvoke.mockResolvedValueOnce(undefined);
    const { result } = renderHook(() => useListening());

    await waitFor(() => {
      expect(startedCallback).not.toBeNull();
    });

    await act(async () => {
      await result.current.enableListening();
    });

    expect(mockInvoke).toHaveBeenCalledWith("enable_listening", {
      deviceName: undefined,
    });
    // State doesn't update immediately - needs event
    expect(result.current.isListening).toBe(false);

    // Simulate backend emitting the event
    act(() => {
      startedCallback!({ payload: { timestamp: "2025-01-01T12:00:00Z" } });
    });

    expect(result.current.isListening).toBe(true);
    expect(result.current.error).toBeNull();
  });

  it("disableListening() calls Tauri command and updates state on event", async () => {
    let startedCallback: ((event: { payload: { timestamp: string } }) => void) | null = null;
    let stoppedCallback: ((event: { payload: { timestamp: string } }) => void) | null = null;

    mockListen.mockImplementation(
      (
        eventName: string,
        callback: (event: { payload: unknown }) => void
      ) => {
        if (eventName === "listening_started") {
          startedCallback = callback;
        } else if (eventName === "listening_stopped") {
          stoppedCallback = callback;
        }
        return Promise.resolve(mockUnlisten);
      }
    );

    mockInvoke.mockResolvedValue(undefined);

    const { result } = renderHook(() => useListening());

    await waitFor(() => {
      expect(startedCallback).not.toBeNull();
      expect(stoppedCallback).not.toBeNull();
    });

    // Enable listening first
    await act(async () => {
      await result.current.enableListening();
    });
    act(() => {
      startedCallback!({ payload: { timestamp: "2025-01-01T12:00:00Z" } });
    });
    expect(result.current.isListening).toBe(true);

    // Disable listening
    await act(async () => {
      await result.current.disableListening();
    });

    expect(mockInvoke).toHaveBeenCalledWith("disable_listening");

    // Simulate backend emitting the stopped event
    act(() => {
      stoppedCallback!({ payload: { timestamp: "2025-01-01T12:00:01Z" } });
    });

    expect(result.current.isListening).toBe(false);
    expect(result.current.error).toBeNull();
  });

  it("wake word detection updates isWakeWordDetected temporarily", async () => {
    // Use fake timers just for this test
    vi.useFakeTimers();

    let wakeWordCallback: ((event: {
      payload: { confidence: number; transcription: string; timestamp: string };
    }) => void) | null = null;

    mockListen.mockImplementation(
      (
        eventName: string,
        callback: (event: { payload: unknown }) => void
      ) => {
        if (eventName === "wake_word_detected") {
          wakeWordCallback = callback;
        }
        return Promise.resolve(mockUnlisten);
      }
    );

    const { result } = renderHook(() => useListening());

    // Manually advance to allow the async setupListeners to complete
    await act(async () => {
      await vi.runAllTimersAsync();
    });

    expect(wakeWordCallback).not.toBeNull();
    expect(result.current.isWakeWordDetected).toBe(false);

    // Simulate wake word detection
    act(() => {
      wakeWordCallback!({
        payload: {
          confidence: 0.95,
          transcription: "hey cat",
          timestamp: "2025-01-01T12:00:00Z",
        },
      });
    });

    expect(result.current.isWakeWordDetected).toBe(true);

    // After timeout, should reset to false
    act(() => {
      vi.advanceTimersByTime(500);
    });

    expect(result.current.isWakeWordDetected).toBe(false);

    vi.useRealTimers();
  });

  it("mic unavailable updates isMicAvailable to false", async () => {
    let unavailableCallback: ((event: {
      payload: { reason: string; timestamp: string };
    }) => void) | null = null;

    mockListen.mockImplementation(
      (
        eventName: string,
        callback: (event: { payload: unknown }) => void
      ) => {
        if (eventName === "listening_unavailable") {
          unavailableCallback = callback;
        }
        return Promise.resolve(mockUnlisten);
      }
    );

    const { result } = renderHook(() => useListening());

    await waitFor(() => {
      expect(unavailableCallback).not.toBeNull();
    });

    expect(result.current.isMicAvailable).toBe(true);

    // Simulate mic unavailable
    act(() => {
      unavailableCallback!({
        payload: {
          reason: "Microphone disconnected",
          timestamp: "2025-01-01T12:00:00Z",
        },
      });
    });

    expect(result.current.isMicAvailable).toBe(false);
    expect(result.current.isListening).toBe(false);
    expect(result.current.error).toBe("Microphone disconnected");
  });

  it("cleanup unsubscribes from all events on unmount", async () => {
    const { unmount } = renderHook(() => useListening());

    await waitFor(() => {
      expect(mockListen).toHaveBeenCalledTimes(4);
    });

    unmount();

    // Each listener's unlisten function should be called
    expect(mockUnlisten).toHaveBeenCalledTimes(4);
  });

  it("sets up event listeners for all required events on mount", async () => {
    renderHook(() => useListening());

    await waitFor(() => {
      expect(mockListen).toHaveBeenCalledTimes(4);
    });

    expect(mockListen).toHaveBeenCalledWith(
      "listening_started",
      expect.any(Function)
    );
    expect(mockListen).toHaveBeenCalledWith(
      "listening_stopped",
      expect.any(Function)
    );
    expect(mockListen).toHaveBeenCalledWith(
      "wake_word_detected",
      expect.any(Function)
    );
    expect(mockListen).toHaveBeenCalledWith(
      "listening_unavailable",
      expect.any(Function)
    );
  });

  it("sets error state when enableListening fails", async () => {
    mockInvoke.mockRejectedValueOnce(new Error("Cannot enable while recording"));
    const { result } = renderHook(() => useListening());

    await act(async () => {
      await result.current.enableListening();
    });

    expect(result.current.error).toBe("Cannot enable while recording");
    expect(result.current.isListening).toBe(false);
  });

  it("sets error state when disableListening fails", async () => {
    mockInvoke.mockRejectedValueOnce("Disable failed");
    const { result } = renderHook(() => useListening());

    await act(async () => {
      await result.current.disableListening();
    });

    expect(result.current.error).toBe("Disable failed");
  });

  it("clears error on successful enableListening", async () => {
    let startedCallback: ((event: { payload: { timestamp: string } }) => void) | null = null;

    mockListen.mockImplementation(
      (
        eventName: string,
        callback: (event: { payload: unknown }) => void
      ) => {
        if (eventName === "listening_started") {
          startedCallback = callback;
        }
        return Promise.resolve(mockUnlisten);
      }
    );

    // First set an error
    mockInvoke.mockRejectedValueOnce(new Error("Initial error"));
    const { result } = renderHook(() => useListening());

    await waitFor(() => {
      expect(startedCallback).not.toBeNull();
    });

    await act(async () => {
      await result.current.enableListening();
    });
    expect(result.current.error).toBe("Initial error");

    // Now succeed - error clears immediately on calling enableListening
    mockInvoke.mockResolvedValueOnce(undefined);
    await act(async () => {
      await result.current.enableListening();
    });

    expect(result.current.error).toBeNull();
  });

  it("returns stable function references", async () => {
    const { result, rerender } = renderHook(() => useListening());

    const enableListening1 = result.current.enableListening;
    const disableListening1 = result.current.disableListening;

    rerender();

    expect(result.current.enableListening).toBe(enableListening1);
    expect(result.current.disableListening).toBe(disableListening1);
  });

  it("listening_started sets isMicAvailable to true", async () => {
    let startedCallback: ((event: { payload: { timestamp: string } }) => void) | null = null;
    let unavailableCallback: ((event: {
      payload: { reason: string; timestamp: string };
    }) => void) | null = null;

    mockListen.mockImplementation(
      (
        eventName: string,
        callback: (event: { payload: unknown }) => void
      ) => {
        if (eventName === "listening_started") {
          startedCallback = callback;
        } else if (eventName === "listening_unavailable") {
          unavailableCallback = callback;
        }
        return Promise.resolve(mockUnlisten);
      }
    );

    const { result } = renderHook(() => useListening());

    await waitFor(() => {
      expect(startedCallback).not.toBeNull();
      expect(unavailableCallback).not.toBeNull();
    });

    // First make mic unavailable
    act(() => {
      unavailableCallback!({
        payload: {
          reason: "Microphone disconnected",
          timestamp: "2025-01-01T12:00:00Z",
        },
      });
    });
    expect(result.current.isMicAvailable).toBe(false);

    // Now listening starts successfully (mic reconnected)
    act(() => {
      startedCallback!({ payload: { timestamp: "2025-01-01T12:00:01Z" } });
    });

    expect(result.current.isMicAvailable).toBe(true);
    expect(result.current.isListening).toBe(true);
  });
});
