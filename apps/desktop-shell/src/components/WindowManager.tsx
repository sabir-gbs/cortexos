import React from "react";
import type { WindowState, AppManifest, AppInstance } from "../types";
import { WindowFrame } from "./WindowFrame";

interface WindowManagerProps {
  windows: WindowState[];
  installedApps: AppManifest[];
  runningApps: AppInstance[];
  onFocus: (id: string) => void;
  onClose: (id: string) => void;
  onMinimize: (id: string) => void;
  onMaximize: (id: string) => void;
  onMove: (id: string, x: number, y: number) => void;
  onResize: (id: string, w: number, h: number) => void;
}

export function WindowManager({
  windows,
  installedApps,
  runningApps,
  onFocus,
  onClose,
  onMinimize,
  onMaximize,
  onMove,
  onResize,
}: WindowManagerProps) {
  return (
    <div className="window-manager" aria-label="Open windows">
      {windows.map((win) => {
        // Find the running app instance for this window to get the app_id,
        // then look up the manifest for the entry_point
        const instance = runningApps.find((r) => r.instance_id === win.instance_id);
        const appId = instance?.app_id;
        const manifest = appId ? installedApps.find((a) => a.id === appId) : undefined;
        const entryPoint = manifest?.entry_point;

        return (
          <WindowFrame
            key={win.id}
            window={win}
            entryPoint={entryPoint}
            onFocus={onFocus}
            onClose={onClose}
            onMinimize={onMinimize}
            onMaximize={onMaximize}
            onMove={onMove}
            onResize={onResize}
          />
        );
      })}
    </div>
  );
}
