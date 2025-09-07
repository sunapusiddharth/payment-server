// src/pages/SendToPhonePage.tsx
import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { paymentApi, generateIdempotencyKey } from '../api/api';
import { PhoneIcon, ArrowLeftIcon } from '@heroicons/react/24/outline';

const sendSchema = z.object({
  to_mobile: z.string().regex(/^\+91[6-9]\d{9}$/, "Invalid Indian mobile number"),
  amount: z.number().min(1, "Amount must be greater than 0").max(500000, "Max ₹5,000"),
});

type SendForm = z.infer<typeof sendSchema>;

const SendToPhonePage: React.FC = () => {
  const navigate = useNavigate();
  const { register, handleSubmit, formState: { errors } } = useForm<SendForm>({
    resolver: zodResolver(sendSchema),
    defaultValues: { amount: 0 },
  });

  const onSubmit = async ( SendForm) => {
    const {  res, error } = await paymentApi.payByPhone({
      ...data,
      idempotency_key: generateIdempotencyKey(),
    });
    if (res) {
      navigate('/success', { state: { tx: res } });
    } else {
      navigate('/failure', { state: { message: error } });
    }
  };

  return (
    <div className="p-6 pt-12">
      <div className="flex items-center mb-6">
        <button onClick={() => navigate(-1)} className="mr-3">
          <ArrowLeftIcon className="w-6 h-6 text-gray-600" />
        </button>
        <h2 className="text-2xl font-bold">Send to Phone</h2>
      </div>

      <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">Phone Number</label>
          <input
            {...register('to_mobile')}
            type="tel"
            placeholder="+91 98765 43210"
            className="w-full p-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary"
          />
          {errors.to_mobile && <p className="text-red-500 text-sm mt-1">{errors.to_mobile.message}</p>}
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">Amount (₹)</label>
          <input
            {...register('amount', { valueAsNumber: true })}
            type="number"
            placeholder="100"
            className="w-full p-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary"
          />
          {errors.amount && <p className="text-red-500 text-sm mt-1">{errors.amount.message}</p>}
        </div>

        <button type="submit" className="w-full bg-primary text-white py-3 rounded-lg font-medium mt-6">
          Send Money
        </button>
      </form>
    </div>
  );
};

export default SendToPhonePage;