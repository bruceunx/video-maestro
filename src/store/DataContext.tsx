import * as React from "react";

interface DataContextType {
  item: string;
}

const DataContext = React.createContext<DataContextType>({
  item: "",
});

export const DataProvider = ({ children }: { children: React.ReactNode }) => {
  return (
    <DataContext.Provider value={{ item: "" }}>{children}</DataContext.Provider>
  );
};

export const useData = () => React.useContext(DataContext);
