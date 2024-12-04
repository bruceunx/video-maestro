import * as React from "react";

interface DataContextType {
  item: string;
}

const DataContext = React.createContext<DataContextType>({
  item: "",
});

const DataProvider = ({ children }: { children: React.ReactNode }) => {
  return (
    <DataContext.Provider value={{ item: "" }}>{children}</DataContext.Provider>
  );
};

export const useData = () => React.useContext(DataContext);

export default DataProvider;
