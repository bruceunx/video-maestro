import React, { useState } from "react";
import * as Toast from "@radix-ui/react-toast";
import {
  IoMdClose as CloseIcon,
  IoMdCheckmarkCircle as SuccessIcon,
  IoMdInformationCircle as InfoIcon,
} from "react-icons/io";
import {
  FaExclamationCircle as ErrorIcon,
  FaExclamationTriangle as WarningIcon,
} from "react-icons/fa";

type ToastVariant = "success" | "error" | "warning" | "info";

interface ToastProps {
  message: string;
  variant?: ToastVariant;
  duration?: number;
}

const ToastNotification: React.FC<ToastProps> = ({
  message,
  variant = "info",
  duration = 3000,
}) => {
  const [open, setOpen] = useState(false);

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

  const showToast = () => {
    setOpen(true);
  };

  return (
    <Toast.Provider swipeDirection="right" duration={duration}>
      <button
        onClick={showToast}
        className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
      >
        Show Toast
      </button>

      <Toast.Root
        open={open}
        onOpenChange={setOpen}
        className={`
          fixed top-4 right-4 z-50 p-4 rounded-lg shadow-lg 
          border ${variantConfig[variant].bg} ${variantConfig[variant].border}
          flex items-center space-x-3
          animate-slide-in-bottom
        `}
      >
        <div className="flex items-center space-x-3">
          {variantConfig[variant].icon}
          <Toast.Description
            className={`text-sm ${variantConfig[variant].text}`}
          >
            {message}
          </Toast.Description>
        </div>
        <Toast.Close
          className="ml-4 hover:bg-gray-100 rounded-full p-1"
          aria-label="Close"
        >
          <CloseIcon size={16} />
        </Toast.Close>
      </Toast.Root>

      <Toast.Viewport />
    </Toast.Provider>
  );
};

export default ToastNotification;
