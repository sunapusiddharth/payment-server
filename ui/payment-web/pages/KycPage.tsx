// src/pages/KycPage.tsx
import React from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { kycApi } from '../api/api';
import { useAuth } from '../contexts/AuthContext';
import { DocumentTextIcon, ArrowLeftIcon } from '@heroicons/react/24/outline';

const kycSchema = z.object({
  pan: z.string().length(10, "PAN must be 10 characters").optional(),
  aadhaar: z.string().length(12, "Aadhaar must be 12 digits").optional(),
  name: z.string().min(2, "Name required"),
  dob: z.string().refine(val => !isNaN(Date.parse(val)), "Invalid date"),
});

type KycForm = z.infer<typeof kycSchema>;

const KycPage: React.FC = () => {
  const { user } = useAuth();
  const { register, handleSubmit, formState: { errors } } = useForm<KycForm>({
    resolver: zodResolver(kycSchema),
  });

  const onSubmit = async (data: KycForm) => {
    if (!user) return;
    const {  res, error } = await kycApi.verifyKyc({
      ...data,
      user_id: user.user_id,
    });
    if (res?.status === 'approved') {
      alert('KYC Approved! Your limits are increased.');
    } else {
      alert(error || 'KYC Failed');
    }
  };

  return (
    <div className="p-6 pt-12">
      <div className="flex items-center mb-6">
        <button onClick={() => window.history.back()} className="mr-3">
          <ArrowLeftIcon className="w-6 h-6 text-gray-600" />
        </button>
        <h2 className="text-2xl font-bold">Complete KYC</h2>
      </div>

      <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">Full Name</label>
          <input
            {...register('name')}
            type="text"
            placeholder="John Doe"
            className="w-full p-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary"
          />
          {errors.name && <p className="text-red-500 text-sm mt-1">{errors.name.message}</p>}
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">Date of Birth</label>
          <input
            {...register('dob')}
            type="date"
            className="w-full p-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary"
          />
          {errors.dob && <p className="text-red-500 text-sm mt-1">{errors.dob.message}</p>}
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">PAN Card (Optional)</label>
          <input
            {...register('pan')}
            type="text"
            placeholder="ABCDE1234F"
            className="w-full p-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary"
          />
          {errors.pan && <p className="text-red-500 text-sm mt-1">{errors.pan.message}</p>}
          <p className="text-xs text-gray-500 mt-1">Use "REJECT_ME" to test rejection</p>
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">Aadhaar (Optional)</label>
          <input
            {...register('aadhaar')}
            type="text"
            placeholder="123456789012"
            className="w-full p-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary"
          />
          {errors.aadhaar && <p className="text-red-500 text-sm mt-1">{errors.aadhaar.message}</p>}
        </div>

        <button type="submit" className="w-full bg-primary text-white py-3 rounded-lg font-medium mt-6">
          Submit KYC
        </button>
      </form>
    </div>
  );
};

export default KycPage;