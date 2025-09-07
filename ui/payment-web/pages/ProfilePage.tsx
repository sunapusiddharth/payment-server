// src/pages/ProfilePage.tsx
import React, { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAuth } from '../contexts/AuthContext';
import { ArrowLeftIcon, IdentificationIcon, BanknotesIcon, ShieldCheckIcon } from '@heroicons/react/24/outline';

interface UserProfile {
  mobile: string;
  kyc_tier: string;
  daily_limit_used: number;
  daily_limit_max: number;
}

const ProfilePage: React.FC = () => {
  const navigate = useNavigate();
  const { user, logout } = useAuth();
  const [profile, setProfile] = useState<UserProfile | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchProfile = async () => {
      try {
        const res = await fetch('/user/profile', {
          headers: { Authorization: `Bearer ${localStorage.getItem('access_token')}` },
        });
        const data = await res.json();
        setProfile(data);
      } catch (error) {
        console.error(error);
      } finally {
        setLoading(false);
      }
    };
    fetchProfile();
  }, []);

  const progress = profile ? (profile.daily_limit_used / profile.daily_limit_max) * 100 : 0;

  return (
    <div className="p-6 pt-12">
      <div className="flex items-center mb-6">
        <button onClick={() => navigate(-1)} className="mr-3">
          <ArrowLeftIcon className="w-6 h-6 text-gray-600" />
        </button>
        <h2 className="text-2xl font-bold">Profile</h2>
      </div>

      {loading ? (
        <div className="space-y-4">
          <div className="h-8 bg-gray-200 rounded animate-pulse"></div>
          <div className="h-4 bg-gray-200 rounded w-1/2 animate-pulse"></div>
          <div className="h-4 bg-gray-200 rounded animate-pulse"></div>
        </div>
      ) : profile && (
        <div className="space-y-6">
          {/* User Info */}
          <div className="p-4 bg-white rounded-xl shadow border dark:bg-gray-800 dark:border-gray-700">
            <div className="flex items-center space-x-4">
              <div className="w-16 h-16 bg-primary text-white rounded-full flex items-center justify-center text-2xl font-bold">
                {user?.user_id.slice(0, 1).toUpperCase()}
              </div>
              <div>
                <p className="text-xl font-bold">User {user?.user_id.slice(0, 8)}</p>
                <p className="text-gray-600 dark:text-gray-400">{profile.mobile}</p>
              </div>
            </div>
          </div>

          {/* KYC Status */}
          <div className="p-4 bg-white rounded-xl shadow border dark:bg-gray-800 dark:border-gray-700">
            <div className="flex items-center mb-3">
              <ShieldCheckIcon className="w-6 h-6 text-primary mr-2" />
              <h3 className="font-bold">KYC Status</h3>
            </div>
            <div className="flex items-center space-x-2">
              <span className={`px-3 py-1 rounded-full text-sm font-medium ${
                profile.kyc_tier === 'full'
                  ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200'
                  : 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200'
              }`}>
                {profile.kyc_tier === 'full' ? 'Verified' : 'Basic'}
              </span>
              {profile.kyc_tier !== 'full' && (
                <button
                  onClick={() => navigate('/kyc')}
                  className="text-primary text-sm font-medium"
                >
                  Upgrade →
                </button>
              )}
            </div>
          </div>

          {/* Daily Limit */}
          <div className="p-4 bg-white rounded-xl shadow border dark:bg-gray-800 dark:border-gray-700">
            <div className="flex items-center mb-3">
              <BanknotesIcon className="w-6 h-6 text-primary mr-2" />
              <h3 className="font-bold">Daily Limit</h3>
            </div>
            <div className="mb-2">
              <div className="flex justify-between text-sm mb-1">
                <span>Used</span>
                <span>{formatINR(profile.daily_limit_used)} / {formatINR(profile.daily_limit_max)}</span>
              </div>
              <div className="w-full bg-gray-200 rounded-full h-2 dark:bg-gray-700">
                <div
                  className="bg-primary h-2 rounded-full"
                  style={{ width: `${Math.min(progress, 100)}%` }}
                ></div>
              </div>
            </div>
          </div>

          {/* Logout Button */}
          <button
            onClick={() => logout()}
            className="w-full p-4 bg-white rounded-xl shadow text-left border border-red-200 text-red-600 dark:bg-gray-800 dark:border-red-900 dark:text-red-400"
          >
            <div className="flex items-center">
              <IdentificationIcon className="w-6 h-6 mr-3" />
              <div>
                <p className="font-medium">Logout</p>
                <p className="text-sm">Sign out of your account</p>
              </div>
            </div>
          </button>
        </div>
      )}
    </div>
  );
};

export default ProfilePage;