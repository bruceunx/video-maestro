import * as React from "react";
import { AiOutlineCheckCircle, AiOutlineRedo } from "react-icons/ai";
import { formatTime } from "../utils/files";
import { useData } from "../store/DataContext";

import { VideoItemProps, VideoListProps } from "../types/db";

const VideoItem = ({ item }: VideoItemProps) => {
  const { item: currentFile } = useData();
  const isInProgress = false;

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
        {item.transcripts.length === 0 ? (
          <AiOutlineRedo className="text-gray-500" />
        ) : (
          <AiOutlineCheckCircle className="text-green-500" />
        )}
        <p className="text-gray-200">
          {item.videoTitle.length > 17
            ? item.videoTitle.substring(0, 17) + "..."
            : item.videoTitle}
        </p>
      </div>
      <div className="flex flex-row justify-between text-sm text-gray-400">
        <p>{formatTime(item.timeLength)}</p>
      </div>
    </div>
  );
};

const VideoItems = ({ items }: VideoListProps) => {
  const [contentHeight, setContentHeight] = React.useState(
    window.innerHeight - 10,
  );

  React.useEffect(() => {
    const handleResize = () => {
      setContentHeight(window.innerHeight - 10);
    };
    window.addEventListener("resize", handleResize);
    return () => window.removeEventListener("resize", handleResize);
  }, []);
  return (
    <div
      className="flex flex-col gap-2 h-full pt-7 px-2 overflow-y-hidden"
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
      {items.map((item, index) => (
        <VideoItem key={index} item={item} />
      ))}
    </div>
  );
};

export default VideoItems;
