export interface VideoData {
  id: number;
  url: string;
  videoTitle: string | null;
  timeLength: number | null;
  transcripts: string | null;
  translate: string | null;
  summary: string | null;
}

export interface VideoItemProps {
  item: VideoData;
}

export interface VideoListProps {
  items: VideoData[];
}
