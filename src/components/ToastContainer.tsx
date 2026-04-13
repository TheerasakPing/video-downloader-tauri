import React from "react";
import { CheckCircle, XCircle, Info, AlertTriangle, X } from "lucide-react";
import { Toast, ToastType } from "../hooks/useToast";

interface ToastItemProps {
  toast: Toast;
  onRemove: (id: string) => void;
}

const ToastItem: React.FC<ToastItemProps> = ({ toast, onRemove }) => {
  const [progress, setProgress] = React.useState(100);
  const duration = 5000; // 5 seconds
  const interval = 50; // Update every 50ms
  const step = 100 / (duration / interval);

  React.useEffect(() => {
    const timer = setInterval(() => {
      setProgress((prev) => {
        if (prev <= step) {
          clearInterval(timer);
          return 0;
        }
        return prev - step;
      });
    }, interval);

    return () => clearInterval(timer);
  }, []);

  const typeConfig: Record<ToastType, { icon: React.ReactNode; bgClass: string; borderClass: string; iconClass: string }> = {
    success: {
      icon: <CheckCircle size={18} />,
      bgClass: "bg-emerald-500/10",
      borderClass: "border-emerald-500/30",
      iconClass: "text-emerald-400",
    },
    error: {
      icon: <XCircle size={18} />,
      bgClass: "bg-red-500/10",
      borderClass: "border-red-500/30",
      iconClass: "text-red-400",
    },
    info: {
      icon: <Info size={18} />,
      bgClass: "bg-blue-500/10",
      borderClass: "border-blue-500/30",
      iconClass: "text-blue-400",
    },
    warning: {
      icon: <AlertTriangle size={18} />,
      bgClass: "bg-amber-500/10",
      borderClass: "border-amber-500/30",
      iconClass: "text-amber-400",
    },
  };

  const config = typeConfig[toast.type];

  return (
    <div
      className={`glass ${config.bgClass} ${config.borderClass} border rounded-lg shadow-lg overflow-hidden animate-slide-in`}
      style={{
        animation: "slideInRight 0.3s ease-out forwards",
      }}
    >
      <div className="flex items-start gap-3 p-3">
        <div className={`flex-shrink-0 ${config.iconClass} drop-shadow-[0_0_4px_currentColor]`}>
          {config.icon}
        </div>
        <div className="flex-1 min-w-0">
          <p className="text-sm text-white font-medium break-words">
            {toast.message}
          </p>
        </div>
        <button
          onClick={() => onRemove(toast.id)}
          className="flex-shrink-0 p-1 hover:bg-white/10 rounded transition-colors"
          aria-label="Dismiss notification"
        >
          <X size={14} className="text-slate-400 hover:text-white" />
        </button>
      </div>
      {/* Progress bar */}
      <div className="h-0.5 bg-slate-700/50">
        <div
          className={`h-full ${config.iconClass.replace('text-', 'bg-')} transition-all duration-${interval}`}
          style={{
            width: `${progress}%`,
            transition: `width ${interval}ms linear`,
          }}
        />
      </div>
    </div>
  );
};

interface ToastContainerProps {
  toasts: Toast[];
  onRemove: (id: string) => void;
}

export const ToastContainer: React.FC<ToastContainerProps> = ({ toasts, onRemove }) => {
  if (toasts.length === 0) return null;

  return (
    <div className="fixed bottom-4 right-4 z-[100] flex flex-col gap-2 max-w-md w-full pointer-events-none">
      {toasts.map((toast) => (
        <div key={toast.id} className="pointer-events-auto">
          <ToastItem toast={toast} onRemove={onRemove} />
        </div>
      ))}
    </div>
  );
};

export default ToastContainer;
