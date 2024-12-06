export interface VideoData {
  id: number;
  url: string;
  title: string;
  duration: number;
  upload_date: string;
  transcripts: string;
  translate: string;
  summary: string;
}

export interface VideoItemProps {
  item: VideoData;
}

export interface VideoListProps {
  items: VideoData[];
}
