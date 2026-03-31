import React from "react";
import type { BackgroundConfig } from "../types";

interface DesktopBackgroundProps {
  background: BackgroundConfig;
}

export function DesktopBackground({ background }: DesktopBackgroundProps) {
  const style: React.CSSProperties = {
    position: "absolute",
    inset: 0,
    zIndex: 0,
  };

  switch (background.type) {
    case "solid":
      style.backgroundColor = background.color;
      break;
    case "gradient":
      style.background = `linear-gradient(${background.direction || "135deg"}, ${background.from}, ${background.to})`;
      break;
    case "image":
      style.backgroundImage = `url(${background.path})`;
      style.backgroundSize = "cover";
      style.backgroundPosition = "center";
      style.backgroundRepeat = "no-repeat";
      break;
  }

  return <div className="desktop-background" style={style} aria-hidden="true" />;
}
