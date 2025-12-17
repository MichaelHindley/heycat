import { describe, it, expect } from "vitest";
import {
  AudioDeviceError,
  getErrorType,
  getErrorMessage,
} from "./audio";

describe("AudioDeviceError types", () => {
  describe("getErrorType", () => {
    it("returns deviceNotFound for deviceNotFound error", () => {
      const error: AudioDeviceError = {
        type: "deviceNotFound",
        deviceName: "Test Mic",
      };
      expect(getErrorType(error)).toBe("deviceNotFound");
    });

    it("returns noDevicesAvailable for noDevicesAvailable error", () => {
      const error: AudioDeviceError = { type: "noDevicesAvailable" };
      expect(getErrorType(error)).toBe("noDevicesAvailable");
    });

    it("returns permissionDenied for permissionDenied error", () => {
      const error: AudioDeviceError = { type: "permissionDenied" };
      expect(getErrorType(error)).toBe("permissionDenied");
    });

    it("returns deviceDisconnected for deviceDisconnected error", () => {
      const error: AudioDeviceError = { type: "deviceDisconnected" };
      expect(getErrorType(error)).toBe("deviceDisconnected");
    });

    it("returns captureError for captureError error", () => {
      const error: AudioDeviceError = {
        type: "captureError",
        message: "test",
      };
      expect(getErrorType(error)).toBe("captureError");
    });
  });

  describe("getErrorMessage", () => {
    it("returns device-specific message for deviceNotFound", () => {
      const error: AudioDeviceError = {
        type: "deviceNotFound",
        deviceName: "USB Microphone",
      };
      expect(getErrorMessage(error)).toBe(
        'The selected microphone "USB Microphone" is not connected.'
      );
    });

    it("returns appropriate message for noDevicesAvailable", () => {
      const error: AudioDeviceError = { type: "noDevicesAvailable" };
      expect(getErrorMessage(error)).toBe(
        "No audio input devices were found. Please connect a microphone."
      );
    });

    it("returns appropriate message for permissionDenied", () => {
      const error: AudioDeviceError = { type: "permissionDenied" };
      expect(getErrorMessage(error)).toBe(
        "Microphone access was denied. Please grant permission in System Preferences."
      );
    });

    it("returns appropriate message for deviceDisconnected", () => {
      const error: AudioDeviceError = { type: "deviceDisconnected" };
      expect(getErrorMessage(error)).toBe(
        "The microphone was disconnected during recording."
      );
    });

    it("returns custom message for captureError", () => {
      const error: AudioDeviceError = {
        type: "captureError",
        message: "Stream initialization failed",
      };
      expect(getErrorMessage(error)).toBe("Stream initialization failed");
    });

    it("returns fallback message for captureError with empty message", () => {
      const error: AudioDeviceError = {
        type: "captureError",
        message: "",
      };
      expect(getErrorMessage(error)).toBe(
        "An error occurred while recording."
      );
    });
  });
});
