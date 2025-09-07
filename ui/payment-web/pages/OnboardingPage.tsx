// src/pages/OnboardingPage.tsx
import React from 'react';
import { Link } from 'react-router-dom';

const OnboardingPage: React.FC = () => {
  return (
    <div className="p-6 pt-12">
      <div className="text-center">
        <h1 className="text-3xl font-bold text-gray-900 mb-2">Welcome to PayRust</h1>
        <p className="text-gray-600 mb-8">India's fastest payment app — built with Rust</p>
        <img src="/logo.svg" alt="Logo" className="w-32 h-32 mx-auto mb-8" />
        <Link to="/login">
          <button className="w-full bg-primary text-white py-3 rounded-lg font-medium">
            Get Started
          </button>
        </Link>
      </div>
    </div>
  );
};

export default OnboardingPage;