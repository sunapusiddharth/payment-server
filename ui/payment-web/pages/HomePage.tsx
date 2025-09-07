// src/pages/HomePage.tsx
import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAuth } from '../contexts/AuthContext';
import { walletApi } from '../api/api';
import { formatINR } from '../api/api';
import { UserIcon } from '@heroicons/react/24/outline';
import { useWebSocket } from '../hooks/useWebSocket';
import { BellIcon, CheckCircleIcon } from '@heroicons/react/24/outline';
import { HomeIcon, CurrencyRupeeIcon, QrCodeIcon, PhoneIcon, DocumentTextIcon, BanknotesIcon, ClockIcon } from '@heroicons/react/24/outline';
import { useTheme } from '../contexts/ThemeContext';
import { MoonIcon, SunIcon } from '@heroicons/react/24/outline';
import { theme } from '../tailwind.config';
const HomePage: React.FC = () => {
  const { user, logout } = useAuth();
  const navigate = useNavigate();
  const [balance, setBalance] = React.useState<number | null>(null);
    const [notifications, setNotifications] = useState<any[]>([]);

  const { connected } = useWebSocket((msg) => {
    if (msg.type === 'payment') {
      setNotifications(prev => [...prev, msg]);
    }
  });

  React.useEffect(() => {
    const fetchBalance = async () => {
      const { data } = await walletApi.getBalance();
      if (data !== null) setBalance(data);
    };
    fetchBalance();
  }, []);

  return (
    <div className="p-6 pt-12">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold">Hello!</h1>
        <button
          onClick={() => logout()}
          className="text-gray-500 text-sm"
        >
          Logout
        </button>
        <button
      onClick={toggleTheme}
      className="p-2 rounded-full bg-gray-200 dark:bg-gray-700"
    >
      {theme === 'dark' ? <SunIcon className="w-5 h-5" /> : <MoonIcon className="w-5 h-5" />}
    </button>
      </div>

{/* Notifications */}
      {notifications.length > 0 && (
        <div className="mb-6 space-y-2">
          {notifications.slice(0, 3).map((notif, i) => (
            <div key={i} className="p-3 bg-green-50 border border-green-200 rounded-lg flex items-center">
              <CheckCircleIcon className="w-5 h-5 text-green-500 mr-2" />
              <div className="flex-1">
                <p className="text-sm font-medium">Payment of {formatINR(notif.amount)} successful</p>
                <p className="text-xs text-green-600">ID: {notif.tx_id.slice(0, 8)}...</p>
              </div>
              <button
                onClick={() => setNotifications(prev => prev.filter((_, idx) => idx !== i))}
                className="text-green-500 hover:text-green-700"
              >
                <XMarkIcon className="w-4 h-4" />
              </button>
            </div>
          ))}
          {notifications.length > 3 && (
            <p className="text-xs text-gray-500 text-center">+{notifications.length - 3} more</p>
          )}
        </div>
      )}

      <div className="bg-gradient-to-r from-blue-500 to-purple-600 rounded-2xl p-6 text-white mb-8">
        <p className="text-blue-100">Available Balance</p>
        <p className="text-3xl font-bold">{balance !== null ? formatINR(balance) : 'Loading...'}</p>
      </div>

      <div className="grid grid-cols-2 gap-4 mb-8">
        <button
          onClick={() => navigate('/send/phone')}
          className="p-4 bg-white rounded-xl shadow text-center border"
        >
          <PhoneIcon className="w-8 h-8 mx-auto text-primary mb-2" />
          <p className="font-medium">Send to Phone</p>
        </button>
        <button
          onClick={() => navigate('/send/qr')}
          className="p-4 bg-white rounded-xl shadow text-center border"
        >
          <QrCodeIcon className="w-8 h-8 mx-auto text-primary mb-2" />
          <p className="font-medium">Scan QR</p>
        </button>
        <button
          onClick={() => navigate('/kyc')}
          className="p-4 bg-white rounded-xl shadow text-center border"
        >
          <DocumentTextIcon className="w-8 h-8 mx-auto text-primary mb-2" />
          <p className="font-medium">Complete KYC</p>
        </button>
        <button
          onClick={() => navigate('/bank')}
          className="p-4 bg-white rounded-xl shadow text-center border"
        >
          <BanknotesIcon className="w-8 h-8 mx-auto text-primary mb-2" />
          <p className="font-medium">Bank Accounts</p>
        </button>
      </div>

      <button
        onClick={() => navigate('/history')}
        className="w-full p-4 bg-white rounded-xl shadow text-left border flex items-center"
      >
        <ClockIcon className="w-6 h-6 text-primary mr-3" />
        <div>
          <p className="font-medium">Transaction History</p>
          <p className="text-gray-500 text-sm">View all transactions</p>
        </div>
      </button>
      <button
  onClick={() => navigate('/contacts')}
  className="p-4 bg-white rounded-xl shadow text-center border"
>
  <UserIcon className="w-8 h-8 mx-auto text-primary mb-2" />
  <p className="font-medium">Contacts</p>
</button>
<button
  onClick={() => navigate('/profile')}
  className="p-4 bg-white rounded-xl shadow text-center border"
>
  <UserIcon className="w-8 h-8 mx-auto text-primary mb-2" />
  <p className="font-medium">Profile</p>
</button>
    </div>
  );
};

export default HomePage;