export interface VideoData {
  id: number;
  url: string;
  videoTitle: string;
  timeLength: number;
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
