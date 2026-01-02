import { useState, useEffect, useCallback } from "react";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

/** A candidate command for disambiguation */
export interface CommandCandidate {
  id: string;
  trigger: string;
  confidence: number;
}

/** Payload for command_ambiguous event */
interface CommandAmbiguousPayload {
  transcription: string;
  candidates: CommandCandidate[];
}

/** Return type of the useDisambiguation hook */
export interface UseDisambiguationResult {
  /** Whether disambiguation is currently needed */
  isAmbiguous: boolean;
  /** The original transcription that triggered disambiguation */
  transcription: string | null;
  /** List of candidate commands to choose from */
  candidates: CommandCandidate[];
  /** Execute a specific command by ID */
  executeCommand: (commandId: string) => Promise<void>;
  /** Dismiss disambiguation without executing */
  dismiss: () => void;
}

/**
 * Custom hook for managing command disambiguation state
 * Listens to backend command_ambiguous events and handles user selection
 */
export function useDisambiguation(): UseDisambiguationResult {
  const [isAmbiguous, setIsAmbiguous] = useState(false);
  const [transcription, setTranscription] = useState<string | null>(null);
  const [candidates, setCandidates] = useState<CommandCandidate[]>([]);

  useEffect(() => {
    const unlistenFns: UnlistenFn[] = [];

    /* v8 ignore start -- @preserve */
    const setupListeners = async () => {
      const unlistenAmbiguous = await listen<CommandAmbiguousPayload>(
        "command_ambiguous",
        (event) => {
          setIsAmbiguous(true);
          setTranscription(event.payload.transcription);
          setCandidates(event.payload.candidates);
        }
      );
      unlistenFns.push(unlistenAmbiguous);

      // Also listen for command_executed to auto-dismiss
      const unlistenExecuted = await listen("command_executed", () => {
        setIsAmbiguous(false);
        setTranscription(null);
        setCandidates([]);
      });
      unlistenFns.push(unlistenExecuted);

      // And command_failed to auto-dismiss
      const unlistenFailed = await listen("command_failed", () => {
        setIsAmbiguous(false);
        setTranscription(null);
        setCandidates([]);
      });
      unlistenFns.push(unlistenFailed);
    };

    setupListeners();
    /* v8 ignore stop */

    return () => {
      /* v8 ignore start -- @preserve */
      unlistenFns.forEach((unlisten) => unlisten());
      /* v8 ignore stop */
    };
  }, []);

  const executeCommand = useCallback(async (commandId: string) => {
    /* v8 ignore start -- @preserve */
    try {
      await invoke("test_command", { id: commandId });
      // State will be cleared by the command_executed listener
    } catch (error) {
      console.error("[heycat] Failed to execute command:", error);
      // Clear state on error
      setIsAmbiguous(false);
      setTranscription(null);
      setCandidates([]);
    }
    /* v8 ignore stop */
  }, []);

  const dismiss = useCallback(() => {
    setIsAmbiguous(false);
    setTranscription(null);
    setCandidates([]);
  }, []);

  return {
    isAmbiguous,
    transcription,
    candidates,
    executeCommand,
    dismiss,
  };
}
