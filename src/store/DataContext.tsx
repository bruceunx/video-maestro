import * as React from "react";
import { invoke } from "@tauri-apps/api/core";
import { VideoData } from "../types/db";

interface VideoDataContextType {
  videos: VideoData[];
  currentVideo: VideoData | null;
  updateCurrentVideo: (video_id: number) => void;
  fetchVideos: () => Promise<void>;
  saveVideo: (videoData: Omit<VideoData, "id">) => Promise<VideoData>;
  updateVideo: (
    id: number,
    videoData: Partial<VideoData>,
  ) => Promise<VideoData>;
  deleteVideo: (id: number) => Promise<void>;
  getVideoById: (id: number) => VideoData | undefined;

  inProgress: boolean;
  setInProgress: (value: boolean) => void;
}

const VideoDataContext = React.createContext<VideoDataContextType | undefined>(
  undefined,
);

export const VideoDataProvider: React.FC<{ children: React.ReactNode }> = ({
  children,
}) => {
  const [videos, setVideos] = React.useState<VideoData[]>([]);
  const [inProgress, setInProgress] = React.useState<boolean>(false);
  const [currentVideo, setCurrentVideo] = React.useState<VideoData | null>(
    null,
  );

  const updateCurrentVideo = (video_id: number) => {
    const video = videos.find((video) => video.id === video_id);
    if (video !== undefined) setCurrentVideo(video);
  };

  const fetchVideos = React.useCallback(async () => {
    try {
      const fetchedVideos = await invoke<VideoData[]>("get_videos");
      setVideos(fetchedVideos);
      if (fetchedVideos.length > 0) setCurrentVideo(fetchedVideos[0]);
    } catch (error) {
      console.error("Failed to fetch videos:", error);
      throw error;
    }
  }, []);

  const saveVideo = React.useCallback(
    async (videoData: Omit<VideoData, "id">) => {
      try {
        const savedVideo = await invoke<VideoData>("create_video", {
          videoData,
        });
        setVideos((prevVideos) => [...prevVideos, savedVideo]);
        return savedVideo;
      } catch (error) {
        console.error("Failed to save video:", error);
        throw error;
      }
    },
    [],
  );

  const updateVideo = React.useCallback(
    async (id: number, videoData: Partial<VideoData>) => {
      try {
        const updatedVideo = await invoke<VideoData>("update_video", {
          id,
          videoData,
        });
        setVideos((prevVideos) =>
          prevVideos.map((video) => (video.id === id ? updatedVideo : video)),
        );
        return updatedVideo;
      } catch (error) {
        console.error("Failed to update video:", error);
        throw error;
      }
    },
    [],
  );

  const deleteVideo = React.useCallback(async (id: number) => {
    try {
      await invoke("delete_video", { id });
      setVideos((prevVideos) => prevVideos.filter((video) => video.id !== id));
      setCurrentVideo(null);
    } catch (error) {
      console.error("Failed to delete video:", error);
      throw error;
    }
  }, []);

  const getVideoById = React.useCallback(
    (id: number) => {
      return videos.find((video) => video.id === id);
    },
    [videos],
  );

  React.useEffect(() => {
    fetchVideos();
    console.log(videos);
  }, []);

  const contextValue = {
    videos,
    currentVideo,
    updateCurrentVideo,
    fetchVideos,
    saveVideo,
    updateVideo,
    deleteVideo,
    getVideoById,
    inProgress,
    setInProgress,
  };

  return (
    <VideoDataContext.Provider value={contextValue}>
      {children}
    </VideoDataContext.Provider>
  );
};

export const useVideoData = () => {
  const context = React.useContext(VideoDataContext);
  if (context === undefined) {
    throw new Error("useVideoData must be used within a VideoDataProvider");
  }
  return context;
};
