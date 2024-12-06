import * as React from "react";
import { CheckCircle, Redo } from "lucide-react";
import { formatTime } from "../utils/files";
import { useVideoData } from "../store/DataContext";

import { VideoItemProps } from "../types/db";

const VideoItem = ({ item }: VideoItemProps) => {
  const isInProgress = false;
  const currentFile = "";

  const onClick = () => {
    // if (!isInProgress) {
    //   setCurrentFile(item.url);
    // }
  };
  return (
    <div
      className={`rounded-md ${currentFile === item.url ? "bg-zinc-500" : ""} p-1 ${!isInProgress && "hover:cursor-pointer"} `}
      onClick={onClick}
    >
      <div className="flex flex-row items-center space-x-3">
        {item.transcripts === null || item.transcripts.length === 0 ? (
          <Redo className="text-gray-500" />
        ) : (
          <CheckCircle className="text-green-500" />
        )}
        <p className="text-gray-200">
          {item.title.length > 17
            ? item.title.substring(0, 17) + "..."
            : item.title}
        </p>
      </div>
      <div className="flex flex-row justify-between text-sm text-gray-400">
        <p>{formatTime(item.duration * 1000)}</p>
      </div>
    </div>
  );
};

const VideoItems = () => {
  const [contentHeight, setContentHeight] = React.useState(
    window.innerHeight - 10,
  );

  const { videos } = useVideoData();

  React.useEffect(() => {
    const handleResize = () => {
      setContentHeight(window.innerHeight - 10);
    };
    window.addEventListener("resize", handleResize);
    return () => window.removeEventListener("resize", handleResize);
  }, []);
  return (
    <div
      className="flex flex-col gap-2 h-full px-2 overflow-y-hidden"
      style={{
        height: contentHeight,
      }}
      onWheel={(e) => {
        e.currentTarget.scrollBy({
          top: e.deltaY,
          behavior: "smooth",
        });
        e.preventDefault();
      }}
    >
      {videos.map((video, index) => (
        <VideoItem key={index} item={video} />
      ))}
    </div>
  );
};

export default VideoItems;
