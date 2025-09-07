// src/pages/BankPage.tsx
import React, { useState, useEffect } from 'react';
import { bankApi } from '../api/api';
import { useAuth } from '../contexts/AuthContext';
import { BanknotesIcon, PlusIcon, ArrowLeftIcon } from '@heroicons/react/24/outline';

const BankPage: React.FC = () => {
  const { user } = useAuth();
  const [accounts, setAccounts] = useState<any[]>([]);
  const [showForm, setShowForm] = useState(false);

  useEffect(() => {
    if (user) {
      // In real app: fetch linked accounts
      // For demo: hardcode or call API
      setAccounts([
        { account_number: 'XXXXXX7890', ifsc: 'HDFC0001234', name: 'John Doe' }
      ]);
    }
  }, [user]);

  const handleAddAccount = async () => {
    if (!user) return;
    const { data, error } = await bankApi.linkAccount({
      user_id: user.user_id,
      account_number: '1234567890',
      ifsc: 'HDFC0001234',
      name: 'John Doe',
    });
    if (!error) {
      setAccounts([...accounts, { account_number: 'XXXXXX7890', ifsc: 'HDFC0001234', name: 'John Doe' }]);
      setShowForm(false);
    }
  };

  return (
    <div className="p-6 pt-12">
      <div className="flex items-center mb-6">
        <button onClick={() => window.history.back()} className="mr-3">
          <ArrowLeftIcon className="w-6 h-6 text-gray-600" />
        </button>
        <h2 className="text-2xl font-bold">Bank Accounts</h2>
      </div>

      {accounts.map((acc, i) => (
        <div key={i} className="p-4 bg-white rounded-xl shadow mb-4 border">
          <p className="font-medium">{acc.name}</p>
          <p className="text-gray-600">{acc.account_number}</p>
          <p className="text-xs text-gray-500">{acc.ifsc}</p>
        </div>
      ))}

      {showForm ? (
        <div className="p-4 bg-white rounded-xl shadow border">
          <h3 className="font-medium mb-4">Add Bank Account</h3>
          <button
            onClick={handleAddAccount}
            className="w-full bg-primary text-white py-2 rounded-lg font-medium"
          >
            Link Demo Account
          </button>
          <button
            onClick={() => setShowForm(false)}
            className="w-full text-gray-500 py-2 text-sm mt-2"
          >
            Cancel
          </button>
        </div>
      ) : (
        <button
          onClick={() => setShowForm(true)}
          className="w-full p-4 bg-white rounded-xl shadow text-center border border-dashed"
        >
          <PlusIcon className="w-6 h-6 text-primary mx-auto mb-1" />
          <p className="font-medium text-primary">Add Bank Account</p>
        </button>
      )}
    </div>
  );
};

export default BankPage;