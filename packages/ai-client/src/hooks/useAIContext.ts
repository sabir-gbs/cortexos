import { useState, useCallback, useRef } from 'react';
import type { AIContextPayload, AIConversation } from '../types';

/**
 * Hook that provides application context to the AI system.
 *
 * This hook allows apps to register their context (active file, selected text,
 * etc.) so that the AI assistant can provide more relevant responses.
 *
 * @param initialContext - Initial context payload for the app
 * @returns Context state and updater functions
 */
export function useAIContext(initialContext?: Partial<AIContextPayload>) {
  const defaultContext: AIContextPayload = {
    appId: '',
    appName: '',
    ...initialContext,
  };

  const [context, setContext] = useState<AIContextPayload>(defaultContext);
  const contextRef = useRef<AIContextPayload>(context);
  contextRef.current = context;

  /** Update the active file in context */
  const setActiveFile = useCallback((filePath: string | undefined) => {
    setContext((prev) => ({
      ...prev,
      activeFile: filePath,
    }));
  }, []);

  /** Update the selected text in context */
  const setSelectedText = useCallback((text: string | undefined) => {
    setContext((prev) => ({
      ...prev,
      selectedText: text,
    }));
  }, []);

  /** Update the selected files in context */
  const setSelectedFiles = useCallback((files: string[] | undefined) => {
    setContext((prev) => ({
      ...prev,
      selectedFiles: files,
    }));
  }, []);

  /** Set a custom context value */
  const setCustomContext = useCallback(
    (key: string, value: unknown) => {
      setContext((prev) => ({
        ...prev,
        customContext: {
          ...prev.customContext,
          [key]: value,
        },
      }));
    },
    [],
  );

  /** Get the current context payload (stable reference via ref) */
  const getContext = useCallback((): AIContextPayload => {
    return contextRef.current;
  }, []);

  return {
    context,
    setActiveFile,
    setSelectedText,
    setSelectedFiles,
    setCustomContext,
    getContext,
  };
}

export default useAIContext;
