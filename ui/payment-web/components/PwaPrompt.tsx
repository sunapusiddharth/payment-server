// src/components/PwaPrompt.tsx
import React, { useState, useEffect } from 'react';

const PwaPrompt: React.FC = () => {
  const [show, setShow] = useState(false);
  const [deferredPrompt, setDeferredPrompt] = useState<any>(null);

  useEffect(() => {
    const handler = (e: any) => {
      e.preventDefault();
      setDeferredPrompt(e);
      setShow(true);
    };

    window.addEventListener('beforeinstallprompt', handler);

    return () => {
      window.removeEventListener('beforeinstallprompt', handler);
    };
  }, []);

  const handleInstall = () => {
    if (deferredPrompt) {
      deferredPrompt.prompt();
      deferredPrompt.userChoice.then((choiceResult: any) => {
        if (choiceResult.outcome === 'accepted') {
          console.log('User accepted the install prompt');
        } else {
          console.log('User dismissed the install prompt');
        }
        setDeferredPrompt(null);
      });
    }
    setShow(false);
  };

  const handleCancel = () => {
    setShow(false);
  };

  if (!show) return null;

  return (
    <div className="fixed bottom-4 left-4 right-4 bg-white p-4 rounded-xl shadow-lg z-50 border dark:bg-gray-800 dark:border-gray-700">
      <p className="mb-3">Install PayRust for faster access and offline use!</p>
      <div className="flex space-x-2">
        <button
          onClick={handleCancel}
          className="flex-1 py-2 px-4 border border-gray-300 rounded-lg text-gray-700 dark:border-gray-600 dark:text-gray-300"
        >
          Cancel
        </button>
        <button
          onClick={handleInstall}
          className="flex-1 py-2 px-4 bg-primary text-white rounded-lg font-medium"
        >
          Install
        </button>
      </div>
    </div>
  );
};

export default PwaPrompt;