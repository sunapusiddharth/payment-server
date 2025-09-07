// src/App.tsx
import React from 'react';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { AuthProvider, useAuth } from './contexts/AuthContext';
import OnboardingPage from './pages/OnboardingPage';
import LoginPage from './pages/LoginPage';
import HomePage from './pages/HomePage';
import SendToPhonePage from './pages/SendToPhonePage';
import SendToQrPage from './pages/SendToQrPage';
import KycPage from './pages/KycPage';
import BankPage from './pages/BankPage';
import TransactionHistoryPage from './pages/TransactionHistoryPage';
import SuccessPage from './pages/SuccessPage';
import FailurePage from './pages/FailurePage';

const App: React.FC = () => {
  return (
    <AuthProvider>
      <BrowserRouter>
        <div className="w-full max-w-md mx-auto bg-white min-h-screen">
          <Routes>
            <Route path="/" element={<OnboardingPage />} />
            <Route path="/login" element={<LoginPage />} />
            <Route path="/home" element={<ProtectedRoute><HomePage /></ProtectedRoute>} />
            <Route path="/send/phone" element={<ProtectedRoute><SendToPhonePage /></ProtectedRoute>} />
            <Route path="/send/qr" element={<ProtectedRoute><SendToQrPage /></ProtectedRoute>} />
            <Route path="/kyc" element={<ProtectedRoute><KycPage /></ProtectedRoute>} />
            <Route path="/bank" element={<ProtectedRoute><BankPage /></ProtectedRoute>} />
            <Route path="/history" element={<ProtectedRoute><TransactionHistoryPage /></ProtectedRoute>} />
            <Route path="/success" element={<SuccessPage />} />
            <Route path="/failure" element={<FailurePage />} />
            <Route path="/contacts" element={<ProtectedRoute><ContactsPage /></ProtectedRoute>} />
            <Route path="/profile" element={<ProtectedRoute><ProfilePage /></ProtectedRoute>} />
          </Routes>
        </div>
      </BrowserRouter>
      <PwaPrompt />
    </AuthProvider>
    
  );
};

const ProtectedRoute: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const { user, loading } = useAuth();
  if (loading) return <div className="p-4">Loading...</div>;
  if (!user) return <Navigate to="/" />;
  return <>{children}</>;
};

export default App;
