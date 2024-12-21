export interface VideoData {
  id: number;
  video_id: string;
  title: string;
  duration: number;
  upload_date: number;
  transcripts: string;
  summary: string;
  keywords: string;
  timestamp: number;
  thumbnail_url: string;
}

export interface VideoItemProps {
  item: VideoData;
}

export interface VideoListProps {
  items: VideoData[];
}
