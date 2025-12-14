import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

/** Types of models that can be downloaded */
export type ModelType = "tdt";

/** Download state for a model */
export type DownloadState = "idle" | "downloading" | "completed" | "error";

/** Status for a single model */
export interface ModelStatus {
  isAvailable: boolean;
  downloadState: DownloadState;
  progress: number; // 0-100
  error: string | null;
}

/** Payload for model_file_download_progress event */
export interface ModelFileDownloadProgressPayload {
  modelType: string;
  fileName: string;
  percent: number;
  bytesDownloaded: number;
  totalBytes: number;
}

/** Payload for model_download_completed event */
interface ModelDownloadCompletedPayload {
  modelType: string;
  modelPath: string;
}

/** Return type of the useMultiModelStatus hook */
export interface UseMultiModelStatusResult {
  /** Status for the TDT model */
  models: ModelStatus;
  /** Function to start downloading the model */
  downloadModel: (modelType: ModelType) => Promise<void>;
  /** Function to refresh model status */
  refreshStatus: () => Promise<void>;
}

const initialModelStatus: ModelStatus = {
  isAvailable: false,
  downloadState: "idle",
  progress: 0,
  error: null,
};

/**
 * Custom hook for managing TDT model status
 * Provides methods to check availability and trigger download
 */
export function useMultiModelStatus(): UseMultiModelStatusResult {
  const [models, setModels] = useState<ModelStatus>({ ...initialModelStatus });

  const updateModelStatus = useCallback(
    (_modelType: ModelType, updates: Partial<ModelStatus>) => {
      setModels((prev) => ({
        ...prev,
        ...updates,
      }));
    },
    []
  );

  const refreshStatus = useCallback(async () => {
    /* v8 ignore start -- @preserve */
    try {
      const tdtAvailable = await invoke<boolean>("check_parakeet_model_status", { modelType: "tdt" });

      setModels((prev) => ({
        ...prev,
        isAvailable: tdtAvailable,
        downloadState: tdtAvailable ? "completed" : prev.downloadState,
      }));
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e);
      setModels((prev) => ({
        ...prev,
        error: errorMsg,
      }));
    }
    /* v8 ignore stop */
  }, []);

  const downloadModel = useCallback(
    async (modelType: ModelType) => {
      updateModelStatus(modelType, {
        error: null,
        downloadState: "downloading",
        progress: 0,
      });
      /* v8 ignore start -- @preserve */
      try {
        await invoke("download_model", { modelType });
        // State will be updated by model_download_completed event
      } catch (e) {
        updateModelStatus(modelType, {
          downloadState: "error",
          error: e instanceof Error ? e.message : String(e),
        });
      }
      /* v8 ignore stop */
    },
    [updateModelStatus]
  );

  useEffect(() => {
    const unlistenFns: UnlistenFn[] = [];

    /* v8 ignore start -- @preserve */
    const setupListeners = async () => {
      // Check initial model status
      await refreshStatus();

      // Listen for download progress
      const unlistenProgress = await listen<ModelFileDownloadProgressPayload>(
        "model_file_download_progress",
        (event) => {
          const modelType = event.payload.modelType as ModelType;
          if (modelType === "tdt") {
            updateModelStatus(modelType, {
              progress: event.payload.percent,
            });
          }
        }
      );
      unlistenFns.push(unlistenProgress);

      // Listen for download completion
      const unlistenCompleted = await listen<ModelDownloadCompletedPayload>(
        "model_download_completed",
        (event) => {
          const modelType = event.payload.modelType as ModelType;
          if (modelType === "tdt") {
            updateModelStatus(modelType, {
              isAvailable: true,
              downloadState: "completed",
              progress: 100,
              error: null,
            });
          }
        }
      );
      unlistenFns.push(unlistenCompleted);
    };

    setupListeners();
    /* v8 ignore stop */

    return () => {
      /* v8 ignore start -- @preserve */
      unlistenFns.forEach((unlisten) => unlisten());
      /* v8 ignore stop */
    };
  }, [refreshStatus, updateModelStatus]);

  return {
    models,
    downloadModel,
    refreshStatus,
  };
}
