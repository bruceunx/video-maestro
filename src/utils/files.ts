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
