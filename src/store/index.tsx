import * as React from "react";
import { VideoDataProvider } from "./DataContext";
import { SettingsProvider } from "./SettingsProvider";

interface CombinedStoreProps {
  children: React.ReactNode;
}

export const CombinedStore: React.FC<CombinedStoreProps> = ({ children }) => (
  <VideoDataProvider>
    <SettingsProvider>{children}</SettingsProvider>
  </VideoDataProvider>
);
