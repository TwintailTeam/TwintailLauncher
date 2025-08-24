import ProgressBar from "../common/ProgressBar";

type DownloadProgressProps = {
  hidden: boolean;
  name: string;
  percentText: string;
  pretty: number | string;
  prettyTotal: number | string;
  progressVal: number;
};

export default function DownloadProgress(props: DownloadProgressProps) {
  const { hidden, name, percentText, pretty, prettyTotal, progressVal } = props;
  return (
    <div
      className={`absolute items-center justify-center bottom-6 left-16 right-96 p-8 z-20 pointer-events-none ${
        hidden ? "hidden" : ""
      }`}
      id={"progress_bar"}
    >
      <h4 className={"pl-4 pb-1 text-white text-stroke inline"} id={"progress_name"}>
        {name}
      </h4>
      <h4 className={"pl-4 pb-1 text-white text-stroke inline"}>
        (<span id={"progress_percent"}>{percentText}</span> |
        <span id={"progress_pretty"}>
          {pretty} / {prettyTotal}
        </span>)
      </h4>
      <ProgressBar id={"progress_value"} progress={progressVal} className={"transition-all duration-500 ease-out"} />
    </div>
  );
}
