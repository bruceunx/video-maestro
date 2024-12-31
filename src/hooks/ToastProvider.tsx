import type React from "react";
import { createContext, useCallback, useContext, useState } from "react";
import * as RadixToast from "@radix-ui/react-toast";
import {
  AlertCircle as ErrorIcon,
  AlertTriangle as WarningIcon,
  CheckCircle as SuccessIcon,
  Info as InfoIcon,
  X as CloseIcon,
} from "lucide-react";

export type ToastVariant = "success" | "error" | "warning" | "info";

export interface ToastMessage {
  id?: string;
  message: string;
  variant?: ToastVariant;
  duration?: number;
}

interface ToastContextType {
  addToast: (message: Omit<ToastMessage, "id">) => void;
}

const ToastContext = createContext<ToastContextType | undefined>(undefined);

export const ToastProvider: React.FC<{ children: React.ReactNode }> = ({
  children,
}) => {
  const [toasts, setToasts] = useState<ToastMessage[]>([]);

  const addToast = useCallback((message: Omit<ToastMessage, "id">) => {
    const id = crypto.randomUUID();
    setToasts((current) => [...current, { ...message, id }]);
  }, []);

  const removeToast = useCallback((id: string | undefined) => {
    if (id !== undefined) {
      setToasts((current) => current.filter((toast) => toast.id !== id));
    }
  }, []);

  const variantConfig = {
    success: {
      icon: <SuccessIcon className="text-green-500" />,
      bg: "bg-green-50",
      border: "border-green-200",
      text: "text-green-800",
    },
    error: {
      icon: <ErrorIcon className="text-red-500" />,
      bg: "bg-red-50",
      border: "border-red-200",
      text: "text-red-800",
    },
    warning: {
      icon: <WarningIcon className="text-yellow-500" />,
      bg: "bg-yellow-50",
      border: "border-yellow-200",
      text: "text-yellow-800",
    },
    info: {
      icon: <InfoIcon className="text-blue-500" />,
      bg: "bg-blue-50",
      border: "border-blue-200",
      text: "text-blue-800",
    },
  };

  return (
    <ToastContext.Provider value={{ addToast }}>
      {children}

      <RadixToast.Provider swipeDirection="right">
        {toasts.map((toast) => (
          <RadixToast.Root
            key={toast.id}
            open={true}
            onOpenChange={() => removeToast(toast.id)}
            duration={toast.duration || 3000}
            className={`
              fixed top-4 right-4 z-50 p-4 rounded-lg shadow-lg 
              ${variantConfig[toast.variant || "info"].bg}
              ${variantConfig[toast.variant || "info"].border}
              flex items-center space-x-3
              animate-slide-in-bottom
            `}
          >
            <div className="flex items-center space-x-3">
              {variantConfig[toast.variant || "info"].icon}
              <RadixToast.Description
                className={`text-sm ${
                  variantConfig[toast.variant || "info"].text
                }`}
              >
                {toast.message}
              </RadixToast.Description>
            </div>
            <RadixToast.Close
              className="ml-4 hover:bg-gray-100 rounded-full p-1"
              aria-label="Close"
              onClick={() => removeToast(toast.id)}
            >
              <CloseIcon size={16} />
            </RadixToast.Close>
          </RadixToast.Root>
        ))}
        <RadixToast.Viewport />
      </RadixToast.Provider>
    </ToastContext.Provider>
  );
};

export const useToast = () => {
  const context = useContext(ToastContext);
  if (context === undefined) {
    throw new Error("useToast must be used within a ToastProvider");
  }
  return context;
};
