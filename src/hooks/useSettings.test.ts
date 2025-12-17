import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { useSettings } from "./useSettings";

// Mock store instance - must be hoisted with vi.hoisted
const { mockStore } = vi.hoisted(() => ({
  mockStore: {
    get: vi.fn(),
    set: vi.fn().mockResolvedValue(undefined),
  },
}));

// Mock Tauri store plugin
vi.mock("@tauri-apps/plugin-store", () => ({
  load: vi.fn().mockResolvedValue(mockStore),
}));

describe("useSettings", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockStore.get.mockResolvedValue(undefined);
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("user sees persisted settings loaded from store", async () => {
    mockStore.get.mockImplementation((key: string) => {
      if (key === "listening.enabled") return Promise.resolve(true);
      if (key === "listening.autoStartOnLaunch") return Promise.resolve(true);
      if (key === "audio.selectedDevice") return Promise.resolve("USB Microphone");
      return Promise.resolve(undefined);
    });

    const { result } = renderHook(() => useSettings());

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.settings.listening.enabled).toBe(true);
    expect(result.current.settings.listening.autoStartOnLaunch).toBe(true);
    expect(result.current.settings.audio.selectedDevice).toBe("USB Microphone");
  });

  it("user can update settings and changes persist to store", async () => {
    const { result } = renderHook(() => useSettings());

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    // Update listening enabled
    await act(async () => {
      await result.current.updateListeningEnabled(true);
    });
    expect(mockStore.set).toHaveBeenCalledWith("listening.enabled", true);
    expect(result.current.settings.listening.enabled).toBe(true);

    // Update auto-start listening
    await act(async () => {
      await result.current.updateAutoStartListening(true);
    });
    expect(mockStore.set).toHaveBeenCalledWith("listening.autoStartOnLaunch", true);
    expect(result.current.settings.listening.autoStartOnLaunch).toBe(true);

    // Update audio device
    await act(async () => {
      await result.current.updateAudioDevice("USB Microphone");
    });
    expect(mockStore.set).toHaveBeenCalledWith("audio.selectedDevice", "USB Microphone");
    expect(result.current.settings.audio.selectedDevice).toBe("USB Microphone");
  });

  it("user sees error when store operations fail", async () => {
    const { load } = await import("@tauri-apps/plugin-store");
    vi.mocked(load).mockRejectedValueOnce(new Error("Store failed"));

    const { result } = renderHook(() => useSettings());

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.error).toBe("Store failed");
  });

  it("user can clear audio device selection", async () => {
    mockStore.get.mockImplementation((key: string) => {
      if (key === "audio.selectedDevice") return Promise.resolve("USB Microphone");
      return Promise.resolve(undefined);
    });

    const { result } = renderHook(() => useSettings());

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.settings.audio.selectedDevice).toBe("USB Microphone");

    await act(async () => {
      await result.current.updateAudioDevice(null);
    });

    expect(mockStore.set).toHaveBeenCalledWith("audio.selectedDevice", null);
    expect(result.current.settings.audio.selectedDevice).toBeNull();
  });
});
