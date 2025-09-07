// src/pages/SuccessPage.tsx
import React from 'react';
import { useLocation, useNavigate } from 'react-router-dom';
import { CheckCircleIcon, HomeIcon } from '@heroicons/react/24/outline';

const SuccessPage: React.FC = () => {
  const navigate = useNavigate();
  const location = useLocation();
  const tx = location.state?.tx;

  return (
    <div className="p-6 pt-12 text-center">
      <CheckCircleIcon className="w-24 h-24 text-green-500 mx-auto mb-6" />
      <h2 className="text-2xl font-bold text-green-600 mb-2">Payment Successful!</h2>
      {tx && (
        <div className="bg-gray-50 p-4 rounded-xl mb-6">
          <p className="text-gray-600">Amount: <span className="font-bold">{formatINR(tx.amount)}</span></p>
          <p className="text-gray-600">To: <span className="font-bold">{tx.to_user_id.slice(0, 8)}...</span></p>
          <p className="text-gray-600">Time: <span className="font-bold">{new Date(tx.timestamp).toLocaleString()}</span></p>
        </div>
      )}
      <button
        onClick={() => navigate('/home')}
        className="w-full bg-primary text-white py-3 rounded-lg font-medium flex items-center justify-center"
      >
        <HomeIcon className="w-5 h-5 mr-2" />
        Go to Home
      </button>
    </div>
  );
};

export default SuccessPage;