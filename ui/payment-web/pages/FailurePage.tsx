// src/pages/FailurePage.tsx
import React from 'react';
import { useLocation, useNavigate } from 'react-router-dom';
import { XCircleIcon, ArrowLeftIcon } from '@heroicons/react/24/outline';

const FailurePage: React.FC = () => {
  const navigate = useNavigate();
  const location = useLocation();
  const message = location.state?.message || 'Payment failed. Please try again.';

  return (
    <div className="p-6 pt-12 text-center">
      <XCircleIcon className="w-24 h-24 text-red-500 mx-auto mb-6" />
      <h2 className="text-2xl font-bold text-red-600 mb-2">Payment Failed</h2>
      <p className="text-gray-600 mb-6">{message}</p>
      <button
        onClick={() => navigate(-1)}
        className="w-full bg-gray-500 text-white py-3 rounded-lg font-medium flex items-center justify-center"
      >
        <ArrowLeftIcon className="w-5 h-5 mr-2" />
        Try Again
      </button>
    </div>
  );
};

export default FailurePage;