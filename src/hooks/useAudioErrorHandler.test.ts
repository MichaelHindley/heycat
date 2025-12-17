import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { useAudioErrorHandler } from "./useAudioErrorHandler";
import { AudioDeviceError } from "../types/audio";

// Mock Tauri APIs
const mockListen = vi.fn();
const mockUnlisten = vi.fn();

vi.mock("@tauri-apps/api/event", () => ({
  listen: (...args: unknown[]) => mockListen(...args),
}));

describe("useAudioErrorHandler", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockListen.mockResolvedValue(mockUnlisten);
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("initializes with error: null", () => {
    const { result } = renderHook(() => useAudioErrorHandler());

    expect(result.current.error).toBeNull();
  });

  it("sets up event listener on mount", async () => {
    renderHook(() => useAudioErrorHandler());

    await waitFor(() => {
      expect(mockListen).toHaveBeenCalledTimes(1);
    });

    expect(mockListen).toHaveBeenCalledWith(
      "audio_device_error",
      expect.any(Function)
    );
  });

  it("cleans up event listener on unmount", async () => {
    const { unmount } = renderHook(() => useAudioErrorHandler());

    await waitFor(() => {
      expect(mockListen).toHaveBeenCalledTimes(1);
    });

    unmount();

    expect(mockUnlisten).toHaveBeenCalledTimes(1);
  });

  it("updates error state when backend emits audio_device_error", async () => {
    let errorCallback: ((event: { payload: AudioDeviceError }) => void) | null =
      null;

    mockListen.mockImplementation(
      (
        eventName: string,
        callback: (event: { payload: AudioDeviceError }) => void
      ) => {
        if (eventName === "audio_device_error") {
          errorCallback = callback;
        }
        return Promise.resolve(mockUnlisten);
      }
    );

    const { result } = renderHook(() => useAudioErrorHandler());

    await waitFor(() => {
      expect(errorCallback).not.toBeNull();
    });

    const error: AudioDeviceError = { type: "noDevicesAvailable" };

    act(() => {
      errorCallback!({ payload: error });
    });

    expect(result.current.error).toEqual(error);
  });

  it("handles deviceNotFound error with deviceName", async () => {
    let errorCallback: ((event: { payload: AudioDeviceError }) => void) | null =
      null;

    mockListen.mockImplementation(
      (
        eventName: string,
        callback: (event: { payload: AudioDeviceError }) => void
      ) => {
        if (eventName === "audio_device_error") {
          errorCallback = callback;
        }
        return Promise.resolve(mockUnlisten);
      }
    );

    const { result } = renderHook(() => useAudioErrorHandler());

    await waitFor(() => {
      expect(errorCallback).not.toBeNull();
    });

    const error: AudioDeviceError = {
      type: "deviceNotFound",
      deviceName: "USB Mic",
    };

    act(() => {
      errorCallback!({ payload: error });
    });

    expect(result.current.error).toEqual(error);
    expect((result.current.error as { type: "deviceNotFound"; deviceName: string }).deviceName).toBe("USB Mic");
  });

  it("handles deviceDisconnected error", async () => {
    let errorCallback: ((event: { payload: AudioDeviceError }) => void) | null =
      null;

    mockListen.mockImplementation(
      (
        eventName: string,
        callback: (event: { payload: AudioDeviceError }) => void
      ) => {
        if (eventName === "audio_device_error") {
          errorCallback = callback;
        }
        return Promise.resolve(mockUnlisten);
      }
    );

    const { result } = renderHook(() => useAudioErrorHandler());

    await waitFor(() => {
      expect(errorCallback).not.toBeNull();
    });

    const error: AudioDeviceError = { type: "deviceDisconnected" };

    act(() => {
      errorCallback!({ payload: error });
    });

    expect(result.current.error).toEqual(error);
  });

  it("handles captureError with message", async () => {
    let errorCallback: ((event: { payload: AudioDeviceError }) => void) | null =
      null;

    mockListen.mockImplementation(
      (
        eventName: string,
        callback: (event: { payload: AudioDeviceError }) => void
      ) => {
        if (eventName === "audio_device_error") {
          errorCallback = callback;
        }
        return Promise.resolve(mockUnlisten);
      }
    );

    const { result } = renderHook(() => useAudioErrorHandler());

    await waitFor(() => {
      expect(errorCallback).not.toBeNull();
    });

    const error: AudioDeviceError = {
      type: "captureError",
      message: "Stream initialization failed",
    };

    act(() => {
      errorCallback!({ payload: error });
    });

    expect(result.current.error).toEqual(error);
    expect((result.current.error as { type: "captureError"; message: string }).message).toBe("Stream initialization failed");
  });

  it("clearError() sets error back to null", async () => {
    let errorCallback: ((event: { payload: AudioDeviceError }) => void) | null =
      null;

    mockListen.mockImplementation(
      (
        eventName: string,
        callback: (event: { payload: AudioDeviceError }) => void
      ) => {
        if (eventName === "audio_device_error") {
          errorCallback = callback;
        }
        return Promise.resolve(mockUnlisten);
      }
    );

    const { result } = renderHook(() => useAudioErrorHandler());

    await waitFor(() => {
      expect(errorCallback).not.toBeNull();
    });

    // Set an error
    const error: AudioDeviceError = { type: "noDevicesAvailable" };
    act(() => {
      errorCallback!({ payload: error });
    });
    expect(result.current.error).toEqual(error);

    // Clear it
    act(() => {
      result.current.clearError();
    });

    expect(result.current.error).toBeNull();
  });

  it("returns stable clearError function reference", async () => {
    const { result, rerender } = renderHook(() => useAudioErrorHandler());

    const clearError1 = result.current.clearError;

    rerender();

    expect(result.current.clearError).toBe(clearError1);
  });

  it("logs error to console when event is received", async () => {
    const consoleSpy = vi.spyOn(console, "error").mockImplementation(() => {});

    let errorCallback: ((event: { payload: AudioDeviceError }) => void) | null =
      null;

    mockListen.mockImplementation(
      (
        eventName: string,
        callback: (event: { payload: AudioDeviceError }) => void
      ) => {
        if (eventName === "audio_device_error") {
          errorCallback = callback;
        }
        return Promise.resolve(mockUnlisten);
      }
    );

    renderHook(() => useAudioErrorHandler());

    await waitFor(() => {
      expect(errorCallback).not.toBeNull();
    });

    const error: AudioDeviceError = { type: "noDevicesAvailable" };

    act(() => {
      errorCallback!({ payload: error });
    });

    expect(consoleSpy).toHaveBeenCalledWith("[AudioError]", error);

    consoleSpy.mockRestore();
  });
});
