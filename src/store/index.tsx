import * as React from "react";
import { DataProvider } from "./DataContext";
import { SettingsProvider } from "./SettingsProvider";

interface CombinedStoreProps {
  children: React.ReactNode;
}

export const CombinedStore: React.FC<CombinedStoreProps> = ({ children }) => (
  <DataProvider>
    <SettingsProvider>{children}</SettingsProvider>
  </DataProvider>
);
