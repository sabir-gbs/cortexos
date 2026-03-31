import React from "react";

interface ConnectionBannerProps {
  connected: boolean;
}

export function ConnectionBanner({ connected }: ConnectionBannerProps) {
  if (connected) return null;

  return (
    <div className="connection-banner" role="alert" aria-live="assertive">
      Connection lost. Attempting to reconnect...
    </div>
  );
}
