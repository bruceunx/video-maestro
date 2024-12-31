import * as React from "react";
import { Activity, CheckCircle } from "lucide-react";
import { formatTime, formatTimestamp } from "../utils/files";
import { useVideoData } from "../store/DataContext";

import type { VideoItemProps } from "../types/db";

const VideoItem = ({ item }: VideoItemProps) => {
  const { currentVideo, updateCurrentVideo, inProgress } = useVideoData();

  const onClick = () => {
    if (!inProgress) {
      updateCurrentVideo(item.id);
    }
  };
  return (
    <div
      className={`rounded-md ${
        currentVideo !== null && currentVideo.id === item.id
          ? "bg-zinc-500"
          : ""
      } p-1 ${!inProgress && "hover:cursor-pointer"} `}
      onClick={onClick}
      onKeyDown={onClick}
    >
      <div className="flex flex-row items-center space-x-3">
        {item.transcripts === null || item.transcripts.length === 0 ? (
          <Activity className="text-gray-200" />
        ) : (
          <CheckCircle className="text-green-500" />
        )}
        <p className="text-gray-200">
          {item.title.length > 17
            ? `${item.title.substring(0, 17)}...`
            : item.title}
        </p>
      </div>
      <div className="flex flex-row justify-between text-sm text-gray-400">
        <p>{formatTime(item.duration * 1000)}</p>
        <p>{formatTimestamp(item.timestamp)}</p>
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
      {videos.map((video) => (
        <VideoItem key={video.id} item={video} />
      ))}
    </div>
  );
};

export default VideoItems;
