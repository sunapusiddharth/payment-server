// src/contexts/AuthContext.tsx
import React, { createContext, useContext, useEffect, useState } from 'react';
import { authApi, LoginResponse } from '../api/api';

interface AuthContextType {
  user: LoginResponse | null;
  login: (mobile: string, otp: string) => Promise<boolean>;
  logout: () => void;
  loading: boolean;
}

const AuthContext = createContext<AuthContextType>({
  user: null,
  login: async () => false,
  logout: () => {},
  loading: true,
});

export const AuthProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [user, setUser] = useState<LoginResponse | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const token = localStorage.getItem('access_token');
    if (token) {
      // Validate token (optional: call /auth/validate)
      const storedUser = JSON.parse(localStorage.getItem('user') || 'null');
      setUser(storedUser);
    }
    setLoading(false);
  }, []);

  const login = async (mobile: string, otp: string): Promise<boolean> => {
    const { data, error } = await authApi.verifyOtp({ mobile, otp });
    if (data) {
      setUser(data);
      localStorage.setItem('user', JSON.stringify(data));
      return true;
    }
    return false;
  };

  const logout = () => {
    authApi.logout(localStorage.getItem('refresh_token') || '');
    setUser(null);
    localStorage.removeItem('user');
    localStorage.removeItem('access_token');
    localStorage.removeItem('refresh_token');
  };

  return (
    <AuthContext.Provider value={{ user, login, logout, loading }}>
      {children}
    </AuthContext.Provider>
  );
};

export const useAuth = () => useContext(AuthContext);