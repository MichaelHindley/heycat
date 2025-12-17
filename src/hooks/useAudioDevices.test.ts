import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { useAudioDevices } from "./useAudioDevices";

// Mock invoke
const { mockInvoke } = vi.hoisted(() => ({
  mockInvoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mockInvoke,
}));

describe("useAudioDevices", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("basic functionality", () => {
    it("returns loading state initially", () => {
      mockInvoke.mockReturnValue(new Promise(() => {}));
      const { result } = renderHook(() =>
        useAudioDevices({ autoRefresh: false })
      );

      expect(result.current.isLoading).toBe(true);
      expect(result.current.devices).toEqual([]);
      expect(result.current.error).toBeNull();
    });

    it("fetches devices on mount", async () => {
      const mockDevices = [
        { name: "Built-in Microphone", isDefault: true },
        { name: "USB Microphone", isDefault: false },
      ];
      mockInvoke.mockResolvedValue(mockDevices);

      const { result } = renderHook(() =>
        useAudioDevices({ autoRefresh: false })
      );

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      expect(mockInvoke).toHaveBeenCalledWith("list_audio_devices");
      expect(result.current.devices).toEqual(mockDevices);
      expect(result.current.error).toBeNull();
    });

    it("handles fetch error", async () => {
      mockInvoke.mockRejectedValue(new Error("Device enumeration failed"));

      const { result } = renderHook(() =>
        useAudioDevices({ autoRefresh: false })
      );

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      expect(result.current.devices).toEqual([]);
      expect(result.current.error?.message).toBe("Device enumeration failed");
    });

    it("handles non-Error error", async () => {
      mockInvoke.mockRejectedValue("String error");

      const { result } = renderHook(() =>
        useAudioDevices({ autoRefresh: false })
      );

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      expect(result.current.error?.message).toBe("String error");
    });

    it("refresh function refetches devices", async () => {
      const initialDevices = [
        { name: "Built-in Microphone", isDefault: true },
      ];
      const updatedDevices = [
        { name: "Built-in Microphone", isDefault: true },
        { name: "USB Microphone", isDefault: false },
      ];

      mockInvoke.mockResolvedValueOnce(initialDevices);
      mockInvoke.mockResolvedValueOnce(updatedDevices);

      const { result } = renderHook(() =>
        useAudioDevices({ autoRefresh: false })
      );

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      expect(result.current.devices).toEqual(initialDevices);

      await act(async () => {
        result.current.refresh();
      });

      await waitFor(() => {
        expect(result.current.devices).toEqual(updatedDevices);
      });

      expect(mockInvoke).toHaveBeenCalledTimes(2);
    });

    it("clears error on successful refresh", async () => {
      mockInvoke.mockRejectedValueOnce(new Error("First error"));
      mockInvoke.mockResolvedValueOnce([
        { name: "Built-in Microphone", isDefault: true },
      ]);

      const { result } = renderHook(() =>
        useAudioDevices({ autoRefresh: false })
      );

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      expect(result.current.error).not.toBeNull();

      await act(async () => {
        result.current.refresh();
      });

      await waitFor(() => {
        expect(result.current.error).toBeNull();
      });
    });

    it("returns empty array when no devices found", async () => {
      mockInvoke.mockResolvedValue([]);

      const { result } = renderHook(() =>
        useAudioDevices({ autoRefresh: false })
      );

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      expect(result.current.devices).toEqual([]);
      expect(result.current.error).toBeNull();
    });
  });

  describe("window focus refresh", () => {
    it("refreshes on window focus", async () => {
      const mockDevices = [{ name: "Built-in Microphone", isDefault: true }];
      mockInvoke.mockResolvedValue(mockDevices);

      renderHook(() => useAudioDevices({ autoRefresh: false }));

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledTimes(1);
      });

      // Simulate window focus event
      await act(async () => {
        window.dispatchEvent(new Event("focus"));
      });

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledTimes(2);
      });
    });

    it("removes focus event listener on unmount", async () => {
      const removeEventListenerSpy = vi.spyOn(window, "removeEventListener");
      const mockDevices = [{ name: "Built-in Microphone", isDefault: true }];
      mockInvoke.mockResolvedValue(mockDevices);

      const { unmount } = renderHook(() =>
        useAudioDevices({ autoRefresh: false })
      );

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledTimes(1);
      });

      unmount();

      expect(removeEventListenerSpy).toHaveBeenCalledWith(
        "focus",
        expect.any(Function)
      );

      removeEventListenerSpy.mockRestore();
    });
  });

  describe("periodic refresh with fake timers", () => {
    beforeEach(() => {
      vi.useFakeTimers();
    });

    afterEach(() => {
      vi.useRealTimers();
    });

    it("sets up interval when autoRefresh is enabled", async () => {
      const setIntervalSpy = vi.spyOn(global, "setInterval");
      const mockDevices = [{ name: "Built-in Microphone", isDefault: true }];
      mockInvoke.mockResolvedValue(mockDevices);

      const { unmount } = renderHook(() =>
        useAudioDevices({ autoRefresh: true, refreshInterval: 5000 })
      );

      // Flush promises to let initial fetch complete
      await vi.runOnlyPendingTimersAsync();

      expect(setIntervalSpy).toHaveBeenCalledWith(expect.any(Function), 5000);

      unmount();
      setIntervalSpy.mockRestore();
    });

    it("does not set up interval when autoRefresh is disabled", async () => {
      const setIntervalSpy = vi.spyOn(global, "setInterval");
      const mockDevices = [{ name: "Built-in Microphone", isDefault: true }];
      mockInvoke.mockResolvedValue(mockDevices);

      const { unmount } = renderHook(() =>
        useAudioDevices({ autoRefresh: false })
      );

      await vi.runOnlyPendingTimersAsync();

      // Check no interval was set with our refresh intervals
      const refreshIntervalCalls = setIntervalSpy.mock.calls.filter(
        (call) => call[1] === 5000
      );
      expect(refreshIntervalCalls).toHaveLength(0);

      unmount();
      setIntervalSpy.mockRestore();
    });

    it("refreshes periodically when autoRefresh is enabled", async () => {
      const mockDevices = [{ name: "Built-in Microphone", isDefault: true }];
      mockInvoke.mockResolvedValue(mockDevices);

      renderHook(() =>
        useAudioDevices({ autoRefresh: true, refreshInterval: 1000 })
      );

      // Initial fetch
      await vi.runOnlyPendingTimersAsync();
      const initialCallCount = mockInvoke.mock.calls.length;

      // Advance timer by 1 second
      await vi.advanceTimersByTimeAsync(1000);
      expect(mockInvoke).toHaveBeenCalledTimes(initialCallCount + 1);

      // Advance timer by another second
      await vi.advanceTimersByTimeAsync(1000);
      expect(mockInvoke).toHaveBeenCalledTimes(initialCallCount + 2);
    });

    it("does not refresh periodically when autoRefresh is disabled", async () => {
      const mockDevices = [{ name: "Built-in Microphone", isDefault: true }];
      mockInvoke.mockResolvedValue(mockDevices);

      renderHook(() => useAudioDevices({ autoRefresh: false }));

      await vi.runOnlyPendingTimersAsync();
      expect(mockInvoke).toHaveBeenCalledTimes(1);

      // Advance timer - should not trigger additional fetches
      await vi.advanceTimersByTimeAsync(10000);
      expect(mockInvoke).toHaveBeenCalledTimes(1);
    });

    it("cleans up interval on unmount", async () => {
      const clearIntervalSpy = vi.spyOn(global, "clearInterval");
      const mockDevices = [{ name: "Built-in Microphone", isDefault: true }];
      mockInvoke.mockResolvedValue(mockDevices);

      const { unmount } = renderHook(() =>
        useAudioDevices({ autoRefresh: true, refreshInterval: 1000 })
      );

      await vi.runOnlyPendingTimersAsync();
      const callCountAfterInitialFetch = mockInvoke.mock.calls.length;

      const clearCallsBefore = clearIntervalSpy.mock.calls.length;
      unmount();

      expect(clearIntervalSpy.mock.calls.length).toBeGreaterThan(
        clearCallsBefore
      );

      // Verify no more fetches happen after unmount
      await vi.advanceTimersByTimeAsync(2000);
      expect(mockInvoke).toHaveBeenCalledTimes(callCountAfterInitialFetch);

      clearIntervalSpy.mockRestore();
    });
  });

  describe("console logging", () => {
    it("logs device changes to console", async () => {
      const consoleSpy = vi
        .spyOn(console, "log")
        .mockImplementation(() => {});
      const initialDevices = [
        { name: "Built-in Microphone", isDefault: true },
      ];
      const updatedDevices = [
        { name: "Built-in Microphone", isDefault: true },
        { name: "USB Microphone", isDefault: false },
      ];

      mockInvoke.mockResolvedValueOnce(initialDevices);
      mockInvoke.mockResolvedValueOnce(updatedDevices);

      const { result } = renderHook(() =>
        useAudioDevices({ autoRefresh: false })
      );

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      // First fetch logs the initial devices
      expect(consoleSpy).toHaveBeenCalledWith(
        "[AudioDevices] Device list changed:",
        initialDevices
      );

      // Trigger refresh
      await act(async () => {
        result.current.refresh();
      });

      await waitFor(() => {
        expect(result.current.devices).toEqual(updatedDevices);
      });

      // Second fetch logs the change
      expect(consoleSpy).toHaveBeenCalledWith(
        "[AudioDevices] Device list changed:",
        updatedDevices
      );

      consoleSpy.mockRestore();
    });
  });
});
