import React from 'react';

interface SkeletonProps {
    className?: string;
    width?: string;
    height?: string;
    rounded?: boolean;
}

const Skeleton: React.FC<SkeletonProps> = ({ 
    className = "", 
    width = "100%", 
    height = "1rem", 
    rounded = false 
}) => {
    return (
        <div 
            className={`bg-gradient-to-r from-slate-700 via-slate-600 to-slate-700 animate-pulse ${rounded ? 'rounded-full' : 'rounded'} ${className}`}
            style={{ width, height }}
        />
    );
};

export default Skeleton;
