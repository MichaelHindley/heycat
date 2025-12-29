import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useAudioPlayback } from "./useAudioPlayback";

// Mock convertFileSrc
const { mockConvertFileSrc } = vi.hoisted(() => ({
  mockConvertFileSrc: vi.fn((path: string) => `asset://localhost/${path}`),
}));

vi.mock("@tauri-apps/api/core", () => ({
  convertFileSrc: mockConvertFileSrc,
}));

// Mock HTMLAudioElement
const mockPlay = vi.fn();
const mockPause = vi.fn();
let mockOnEnded: (() => void) | null = null;
let mockOnError: (() => void) | null = null;

class MockAudio {
  src = "";
  currentTime = 0;

  play = mockPlay;
  pause = mockPause;

  set onended(handler: (() => void) | null) {
    mockOnEnded = handler;
  }

  set onerror(handler: (() => void) | null) {
    mockOnError = handler;
  }
}

// @ts-expect-error - overriding global Audio for testing
global.Audio = MockAudio;

describe("useAudioPlayback", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockPlay.mockResolvedValue(undefined);
    mockOnEnded = null;
    mockOnError = null;
  });

  it("returns initial state with no audio playing", () => {
    const { result } = renderHook(() => useAudioPlayback());

    expect(result.current.isPlaying).toBe(false);
    expect(result.current.currentFilePath).toBeNull();
    expect(result.current.error).toBeNull();
  });

  it("plays audio when play is called", async () => {
    const { result } = renderHook(() => useAudioPlayback());

    await act(async () => {
      await result.current.play("/path/to/audio.wav");
    });

    expect(mockConvertFileSrc).toHaveBeenCalledWith("/path/to/audio.wav");
    expect(mockPlay).toHaveBeenCalled();
    expect(result.current.isPlaying).toBe(true);
    expect(result.current.currentFilePath).toBe("/path/to/audio.wav");
  });

  it("pauses audio when pause is called", async () => {
    const { result } = renderHook(() => useAudioPlayback());

    await act(async () => {
      await result.current.play("/path/to/audio.wav");
    });

    act(() => {
      result.current.pause();
    });

    expect(mockPause).toHaveBeenCalled();
    expect(result.current.isPlaying).toBe(false);
  });

  it("stops audio and resets time when stop is called", async () => {
    const { result } = renderHook(() => useAudioPlayback());

    await act(async () => {
      await result.current.play("/path/to/audio.wav");
    });

    act(() => {
      result.current.stop();
    });

    expect(mockPause).toHaveBeenCalled();
    expect(result.current.isPlaying).toBe(false);
  });

  it("toggle pauses when same file is playing", async () => {
    const { result } = renderHook(() => useAudioPlayback());

    await act(async () => {
      await result.current.play("/path/to/audio.wav");
    });

    expect(result.current.isPlaying).toBe(true);

    await act(async () => {
      await result.current.toggle("/path/to/audio.wav");
    });

    expect(mockPause).toHaveBeenCalled();
    expect(result.current.isPlaying).toBe(false);
  });

  it("toggle plays when called on different file", async () => {
    const { result } = renderHook(() => useAudioPlayback());

    await act(async () => {
      await result.current.play("/path/to/audio1.wav");
    });

    mockPlay.mockClear();

    await act(async () => {
      await result.current.toggle("/path/to/audio2.wav");
    });

    expect(mockPlay).toHaveBeenCalled();
    expect(result.current.isPlaying).toBe(true);
    expect(result.current.currentFilePath).toBe("/path/to/audio2.wav");
  });

  it("sets isPlaying to false when audio ends", async () => {
    const { result } = renderHook(() => useAudioPlayback());

    await act(async () => {
      await result.current.play("/path/to/audio.wav");
    });

    expect(result.current.isPlaying).toBe(true);

    // Simulate audio ending
    act(() => {
      if (mockOnEnded) {
        mockOnEnded();
      }
    });

    expect(result.current.isPlaying).toBe(false);
  });

  it("sets error when audio fails to play", async () => {
    mockPlay.mockRejectedValueOnce(new Error("Playback failed"));

    const { result } = renderHook(() => useAudioPlayback());

    await act(async () => {
      await result.current.play("/path/to/audio.wav");
    });

    expect(result.current.error).toBe("Playback failed");
    expect(result.current.isPlaying).toBe(false);
  });

  it("sets error when audio element errors", async () => {
    const { result } = renderHook(() => useAudioPlayback());

    await act(async () => {
      await result.current.play("/path/to/audio.wav");
    });

    expect(result.current.isPlaying).toBe(true);

    // Simulate audio error
    act(() => {
      if (mockOnError) {
        mockOnError();
      }
    });

    expect(result.current.error).toBe("Failed to play audio file");
    expect(result.current.isPlaying).toBe(false);
    expect(result.current.currentFilePath).toBeNull();
  });
});
