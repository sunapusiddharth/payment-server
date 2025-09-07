// src/pages/LoginPage.tsx
import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { useAuth } from '../contexts/AuthContext';

const loginSchema = z.object({
  mobile: z.string().regex(/^\+91[6-9]\d{9}$/, "Invalid Indian mobile number"),
  otp: z.string().length(6, "OTP must be 6 digits").optional(),
});

type LoginForm = z.infer<typeof loginSchema>;

const LoginPage: React.FC = () => {
  const { login } = useAuth();
  const navigate = useNavigate();
  const [step, setStep] = useState<'mobile' | 'otp'>('mobile');
  const [mobile, setMobile] = useState('');

  const { register, handleSubmit, formState: { errors } } = useForm<LoginForm>({
    resolver: zodResolver(loginSchema),
  });

  const onSubmitMobile = ( LoginForm) => {
    setMobile(data.mobile);
    // In real app: call authApi.register(data.mobile)
    setStep('otp');
  };

  const onSubmitOtp = async (data: LoginForm) => {
    const success = await login(mobile, data.otp || '');
    if (success) {
      navigate('/home');
    } else {
      alert('Invalid OTP');
    }
  };


   const handleBiometricLogin = async () => {
    try {
      // Check if WebAuthn is supported
      if (!window.PublicKeyCredential) {
        alert('Biometric login not supported on this device');
        return;
      }

      // Start authentication
      const response = await fetch('/auth/webauthn/start', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ user_id: 'user123' }), // In real app: get from mobile login
      });

      if (!response.ok) {
        throw new Error('Failed to start biometric login');
      }

      const { request } = await response.json();

      // Authenticate with biometrics
      const credential = await navigator.credentials.get({
        publicKey: request.publicKey,
      });

      // Finish authentication
      const finishResponse = await fetch('/auth/webauthn/finish', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          user_id: 'user123',
          credential: credential,
        }),
      });

      if (finishResponse.ok) {
        // Login successful
        navigate('/home');
      } else {
        throw new Error('Biometric authentication failed');
      }
    } catch (error) {
      console.error('Biometric login error:', error);
      alert('Biometric login failed. Please try again.');
    }
  };

  return (
    <div className="p-6 pt-12">
      <h2 className="text-2xl font-bold mb-6 text-center">
        {step === 'mobile' ? 'Enter Mobile Number' : 'Enter OTP'}
      </h2>

      <form onSubmit={handleSubmit(step === 'mobile' ? onSubmitMobile : onSubmitOtp)} className="space-y-4">
        {step === 'mobile' ? (
          <>
            <div>
              <input
                {...register('mobile')}
                type="tel"
                placeholder="+91 98765 43210"
                className="w-full p-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary"
              />
              {errors.mobile && <p className="text-red-500 text-sm mt-1">{errors.mobile.message}</p>}
            </div>
            <button type="submit" className="w-full bg-primary text-white py-3 rounded-lg font-medium">
              Send OTP
            </button>
            <button
              type="button"
              onClick={handleBiometricLogin}
              className="w-full flex items-center justify-center py-3 border border-gray-300 rounded-lg font-medium text-gray-700 mt-2"
            >
              <FingerprintIcon className="w-5 h-5 mr-2" />
              Use Biometric Login
            </button>
          </>
        ) : (
          <>
            <p className="text-center text-gray-600 mb-4">Enter OTP sent to {mobile}</p>
            <div>
              <input
                {...register('otp')}
                type="text"
                placeholder="123456"
                className="w-full p-3 border border-gray-300 rounded-lg text-center text-2xl tracking-widest font-mono focus:outline-none focus:ring-2 focus:ring-primary"
                maxLength={6}
              />
              {errors.otp && <p className="text-red-500 text-sm mt-1">{errors.otp.message}</p>}
            </div>
            <button type="submit" className="w-full bg-primary text-white py-3 rounded-lg font-medium">
              Verify OTP
            </button>
            <button
              type="button"
              onClick={() => setStep('mobile')}
              className="w-full text-primary py-2 font-medium"
            >
              ← Change Number
            </button>
          </>
        )}
      </form>
    </div>
  );
};

export default LoginPage;