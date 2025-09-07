// src/pages/TransactionHistoryPage.tsx
import React, { useState, useEffect } from 'react';
import { transactionApi } from '../api/api';
import { formatINR } from '../api/api';
import { useNavigate } from 'react-router-dom';
import { ClockIcon, ArrowLeftIcon, MagnifyingGlassIcon, FunnelIcon } from '@heroicons/react/24/outline';
import { SkeletonTransaction } from '../components/SkeletonLoader';

const TransactionHistoryPage: React.FC = () => {
  const navigate = useNavigate();
  const [transactions, setTransactions] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [searchTerm, setSearchTerm] = useState('');
  const [filterType, setFilterType] = useState<'all' | 'sent' | 'received'>('all');

  useEffect(() => {
    const fetchTransactions = async () => {
      setLoading(true);
      try {
        const { data } = await transactionApi.getHistory(50, 0, searchTerm, filterType === 'all' ? undefined : filterType);
        if (data) setTransactions(data);
      } catch (error) {
        console.error(error);
      } finally {
        setLoading(false);
      }
    };
    fetchTransactions();
  }, [searchTerm, filterType]);

  return (
    <div className="p-6 pt-12">
      <div className="flex items-center mb-6">
        <button onClick={() => navigate(-1)} className="mr-3">
          <ArrowLeftIcon className="w-6 h-6 text-gray-600" />
        </button>
        <h2 className="text-2xl font-bold">Transaction History</h2>
      </div>

      {/* Search Bar */}
      <div className="relative mb-4">
        <MagnifyingGlassIcon className="absolute left-3 top-1/2 transform -translate-y-1/2 w-5 h-5 text-gray-400" />
        <input
          type="text"
          placeholder="Search contacts..."
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
          className="w-full pl-10 pr-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary dark:bg-gray-800 dark:border-gray-600"
        />
      </div>

      {/* Filter Tabs */}
      <div className="flex space-x-2 mb-6">
        {(['all', 'sent', 'received'] as const).map((type) => (
          <button
            key={type}
            onClick={() => setFilterType(type)}
            className={`px-4 py-2 rounded-lg font-medium text-sm ${
              filterType === type
                ? 'bg-primary text-white'
                : 'bg-gray-200 text-gray-700 dark:bg-gray-700 dark:text-gray-300'
            }`}
          >
            {type.charAt(0).toUpperCase() + type.slice(1)}
          </button>
        ))}
      </div>

      {loading ? (
        <div className="space-y-4">
          {[1, 2, 3].map(i => (
            <SkeletonTransaction key={i} />
          ))}
        </div>
      ) : transactions.length === 0 ? (
        <p className="text-center text-gray-500 py-8">No transactions found</p>
      ) : (
        <div className="space-y-4">
          {transactions.map((tx) => (
            <div key={tx.tx_id} className="p-4 bg-white rounded-xl shadow border dark:bg-gray-800 dark:border-gray-700">
              <div className="flex justify-between items-start">
                <div>
                  <p className="font-medium">
                    {tx.transaction_type === 'sent' ? 'Sent to' : 'Received from'}
                  </p>
                  <p className="text-gray-600 dark:text-gray-400">{tx.counterparty_mobile}</p>
                </div>
                <div className="text-right">
                  <p className={`font-bold ${tx.transaction_type === 'sent' ? 'text-red-500' : 'text-green-500'}`}>
                    {tx.transaction_type === 'sent' ? '-' : '+'}{formatINR(tx.amount)}
                  </p>
                  <p className="text-xs text-gray-500 dark:text-gray-400">{new Date(tx.timestamp).toLocaleString()}</p>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

export default TransactionHistoryPage;