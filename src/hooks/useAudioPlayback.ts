import { useState, useRef, useCallback, useEffect } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";

export interface UseAudioPlaybackReturn {
  isPlaying: boolean;
  currentFilePath: string | null;
  play: (filePath: string) => Promise<void>;
  pause: () => void;
  stop: () => void;
  toggle: (filePath: string) => Promise<void>;
  error: string | null;
}

export function useAudioPlayback(): UseAudioPlaybackReturn {
  const [isPlaying, setIsPlaying] = useState(false);
  const [currentFilePath, setCurrentFilePath] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const audioRef = useRef<HTMLAudioElement | null>(null);

  const cleanup = useCallback(() => {
    if (audioRef.current) {
      audioRef.current.pause();
      audioRef.current.src = "";
      audioRef.current = null;
    }
  }, []);

  useEffect(() => {
    return () => {
      cleanup();
    };
  }, [cleanup]);

  const play = useCallback(async (filePath: string) => {
    try {
      setError(null);

      // If playing a different file, stop the current one
      if (audioRef.current && currentFilePath !== filePath) {
        cleanup();
      }

      // Create new audio element if needed
      if (!audioRef.current || currentFilePath !== filePath) {
        const assetUrl = convertFileSrc(filePath);
        audioRef.current = new Audio(assetUrl);

        audioRef.current.onended = () => {
          setIsPlaying(false);
        };

        audioRef.current.onerror = () => {
          setError("Failed to play audio file");
          setIsPlaying(false);
          setCurrentFilePath(null);
        };
      }

      setCurrentFilePath(filePath);
      await audioRef.current.play();
      setIsPlaying(true);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to play audio");
      setIsPlaying(false);
    }
  }, [currentFilePath, cleanup]);

  const pause = useCallback(() => {
    if (audioRef.current) {
      audioRef.current.pause();
      setIsPlaying(false);
    }
  }, []);

  const stop = useCallback(() => {
    if (audioRef.current) {
      audioRef.current.pause();
      audioRef.current.currentTime = 0;
      setIsPlaying(false);
    }
  }, []);

  const toggle = useCallback(async (filePath: string) => {
    // If clicking on the same file that's playing, pause it
    if (currentFilePath === filePath && isPlaying) {
      pause();
    } else {
      // If clicking on a different file or the same file that's paused, play it
      await play(filePath);
    }
  }, [currentFilePath, isPlaying, pause, play]);

  return {
    isPlaying,
    currentFilePath,
    play,
    pause,
    stop,
    toggle,
    error,
  };
}
