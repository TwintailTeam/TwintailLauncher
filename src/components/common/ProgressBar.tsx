export default function ProgressBar({ id, progress, className }: {progress: any, className: any, id: any}) {
    return (
        <div className={`w-full bg-white/20 rounded-full h-2.5 ${className}`}>
            <div id={id} className="bg-blue-600 h-2.5 rounded-full" style={{ width: `${progress}%` }}></div>
        </div>
    );
};