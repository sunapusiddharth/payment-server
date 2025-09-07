// src/components/SkeletonLoader.tsx
import React from 'react';

interface SkeletonProps {
  height?: string;
  width?: string;
  className?: string;
}

export const Skeleton: React.FC<SkeletonProps> = ({ height = 'h-4', width = 'w-full', className = '' }) => {
  return (
    <div className={`bg-gray-200 rounded animate-pulse ${height} ${width} ${className}`}></div>
  );
};

export const SkeletonTransaction = () => (
  <div className="p-4 bg-white rounded-xl shadow border animate-pulse">
    <div className="flex justify-between items-start">
      <div className="space-y-2">
        <Skeleton width="w-1/3" />
        <Skeleton width="w-1/2" height="h-3" />
      </div>
      <div className="text-right space-y-2">
        <Skeleton width="w-20" />
        <Skeleton width="w-16" height="h-3" />
      </div>
    </div>
  </div>
);