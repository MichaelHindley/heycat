import { describe, it, expect } from "vitest";
import { queryKeys } from "../queryKeys";
import { queryClient } from "../queryClient";

describe("queryKeys", () => {
  describe("static query keys", () => {
    it("listRecordings produces correct key", () => {
      expect(queryKeys.tauri.listRecordings).toEqual([
        "tauri",
        "list_recordings",
      ]);
    });

    it("getRecordingState produces correct key", () => {
      expect(queryKeys.tauri.getRecordingState).toEqual([
        "tauri",
        "get_recording_state",
      ]);
    });

    it("listAudioDevices produces correct key", () => {
      expect(queryKeys.tauri.listAudioDevices).toEqual([
        "tauri",
        "list_audio_devices",
      ]);
    });

    it("getListeningStatus produces correct key", () => {
      expect(queryKeys.tauri.getListeningStatus).toEqual([
        "tauri",
        "get_listening_status",
      ]);
    });
  });

  describe("parameterized query keys", () => {
    it("checkModelStatus produces correct key with tdt type", () => {
      expect(queryKeys.tauri.checkModelStatus("tdt")).toEqual([
        "tauri",
        "check_parakeet_model_status",
        "tdt",
      ]);
    });

    it("checkModelStatus produces correct key with ctc type", () => {
      expect(queryKeys.tauri.checkModelStatus("ctc")).toEqual([
        "tauri",
        "check_parakeet_model_status",
        "ctc",
      ]);
    });

    it("checkModelStatus produces unique keys for different types", () => {
      const tdtKey = queryKeys.tauri.checkModelStatus("tdt");
      const ctcKey = queryKeys.tauri.checkModelStatus("ctc");
      expect(tdtKey).not.toEqual(ctcKey);
    });
  });
});

describe("queryClient", () => {
  it("can be instantiated without errors", () => {
    expect(queryClient).toBeDefined();
    expect(queryClient.getDefaultOptions()).toBeDefined();
  });

  it("has expected default query options", () => {
    const defaults = queryClient.getDefaultOptions();
    expect(defaults.queries?.staleTime).toBe(60 * 1000);
    expect(defaults.queries?.gcTime).toBe(5 * 60 * 1000);
    expect(defaults.queries?.retry).toBe(3);
    expect(defaults.queries?.refetchOnWindowFocus).toBe(false);
  });
});
