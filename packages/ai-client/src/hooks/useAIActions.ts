import { useState, useCallback, useRef } from 'react';
import type {
  AIAction,
  AIActionResult,
  AIActionTarget,
  AIActionCategory,
} from '../types';

/** Handler function type for an AI action */
export type AIActionHandler = (
  action: AIAction,
  target: AIActionTarget,
) => Promise<AIActionResult>;

interface RegisteredAction {
  action: AIAction;
  handler: AIActionHandler;
}

/**
 * Hook for registering and invoking AI actions within an app.
 *
 * Apps use this hook to register their own AI capabilities (e.g.,
 * "Improve Writing", "Summarize Selection") that appear in the
 * AI context menu and app toolbar.
 *
 * @param appId - The unique identifier of the registering app
 * @returns Action registration and invocation utilities
 */
export function useAIActions(appId: string) {
  const [actions, setActions] = useState<RegisteredAction[]>([]);
  const handlersRef = useRef<Map<string, AIActionHandler>>(new Map());

  /**
   * Register a new AI action for this app.
   * If an action with the same actionId exists, it will be replaced.
   */
  const registerAction = useCallback(
    (action: Omit<AIAction, 'appId'>, handler: AIActionHandler) => {
      const fullAction: AIAction = {
        ...action,
        appId,
      };

      handlersRef.current.set(action.actionId, handler);
      setActions((prev) => {
        const filtered = prev.filter(
          (r) => r.action.actionId !== action.actionId,
        );
        return [...filtered, { action: fullAction, handler }];
      });

      return fullAction;
    },
    [appId],
  );

  /**
   * Unregister a specific AI action by its actionId.
   */
  const unregisterAction = useCallback((actionId: string) => {
    handlersRef.current.delete(actionId);
    setActions((prev) => prev.filter((r) => r.action.actionId !== actionId));
  }, []);

  /**
   * Unregister all actions for this app.
   */
  const unregisterAll = useCallback(() => {
    handlersRef.current.clear();
    setActions([]);
  }, []);

  /**
   * Get all registered actions.
   */
  const getActions = useCallback((): AIAction[] => {
    return actions.map((r) => r.action);
  }, [actions]);

  /**
   * Invoke a registered action by its actionId.
   */
  const invokeAction = useCallback(
    async (
      actionId: string,
      target: AIActionTarget,
    ): Promise<AIActionResult> => {
      const handler = handlersRef.current.get(actionId);
      if (!handler) {
        throw new Error(`No handler registered for action: ${actionId}`);
      }

      const registered = actions.find((r) => r.action.actionId === actionId);
      if (!registered) {
        throw new Error(`Action not found: ${actionId}`);
      }

      return handler(registered.action, target);
    },
    [actions],
  );

  return {
    actions: actions.map((r) => r.action),
    registerAction,
    unregisterAction,
    unregisterAll,
    getActions,
    invokeAction,
  };
}

export default useAIActions;
