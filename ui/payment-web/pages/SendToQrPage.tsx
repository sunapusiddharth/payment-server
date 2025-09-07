// src/pages/SendToQrPage.tsx
import React, { useState, useRef } from 'react';
import { useNavigate } from 'react-router-dom';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { paymentApi, generateIdempotencyKey } from '../api/api';
import { QrReader } from 'react-qr-reader';
import { QrCodeIcon, CameraIcon, ArrowLeftIcon, XMarkIcon } from '@heroicons/react/24/outline';

const qrSchema = z.object({
  amount: z.number().min(1).max(500000),
});

type QrForm = z.infer<typeof qrSchema>;

const SendToQrPage: React.FC = () => {
  const navigate = useNavigate();
  const [scannedQr, setScannedQr] = useState<string | null>(null);
  const [showScanner, setShowScanner] = useState(false);
  const videoRef = useRef<HTMLVideoElement>(null);
  const { register, handleSubmit, formState: { errors } } = useForm<QrForm>({
    resolver: zodResolver(qrSchema),
    defaultValues: { amount: 0 },
  });

  const handleScan = (result: any) => {
    if (result) {
      setScannedQr(result.getText());
      setShowScanner(false);
    }
  };

  const handleError = (err: any) => {
    console.error(err);
  };

  const onSubmit = async ( QrForm) => {
    if (!scannedQr) return;
    const {  res, error } = await paymentApi.payByQr({
      qr_code: scannedQr,
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
        <h2 className="text-2xl font-bold">Pay via QR</h2>
      </div>

      {showScanner ? (
        <div className="relative">
          <button
            onClick={() => setShowScanner(false)}
            className="absolute top-0 right-0 z-10 bg-red-500 text-white p-2 rounded-full m-2"
          >
            <XMarkIcon className="w-6 h-6" />
          </button>
          <QrReader
            onResult={handleScan}
            onError={handleError}
            videoId="qr-video"
            videoStyle={{ width: '100%', height: '300px', borderRadius: '12px' }}
            constraints={{ facingMode: 'environment' }}
          />
        </div>
      ) : !scannedQr ? (
        <div className="text-center">
          <QrCodeIcon className="w-24 h-24 mx-auto text-gray-300 mb-6" />
          <button
            onClick={() => setShowScanner(true)}
            className="w-full bg-primary text-white py-3 rounded-lg font-medium flex items-center justify-center"
          >
            <CameraIcon className="w-5 h-5 mr-2" />
            Scan QR Code
          </button>
        </div>
      ) : (
        <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
          <div className="text-center p-4 bg-green-50 rounded-lg border border-green-200">
            <p className="text-green-800 font-medium">QR Scanned Successfully!</p>
            <p className="text-sm text-green-600">Payment to: {scannedQr.substring(0, 30)}...</p>
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
            Confirm Payment
          </button>
        </form>
      )}
    </div>
  );
};

export default SendToQrPage;