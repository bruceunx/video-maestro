export function formatTime(millis: number): string {
  const hours = Math.floor(millis / 3600000);
  millis %= 3600000;
  const minutes = Math.floor(millis / 60000);
  millis %= 60000;
  const seconds = Math.floor(millis / 1000);
  const formattedHours = String(hours).padStart(2, "0");
  const formattedMinutes = String(minutes).padStart(2, "0");
  const formattedSeconds = String(seconds).padStart(2, "0");

  if (hours === 0) {
    return `${formattedMinutes}:${formattedSeconds}`;
  } else {
    return `${formattedHours}:${formattedMinutes}:${formattedSeconds}`;
  }
}

export function formatTimestamp(timestamp: number): string {
  const date = new Date(timestamp * 1000);

  return date.toISOString().slice(0, 10);
}
export function formatDate(dateString: string): string {
  return `${dateString.slice(0, 4)}-${dateString.slice(4, 6)}-${dateString.slice(6)}`;
}
