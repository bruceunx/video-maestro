import { VideoData } from "./types/db";

export const videoList: VideoData[] = [
  {
    id: 1,
    url: "https://example.com/video1",
    videoTitle: "Understanding JavaScript Closures",
    timeLength: 600, // in seconds
    transcripts: "In this video, we will explore closures in JavaScript...",
    translate:
      "Dalam video ini, kita akan menjelajahi closure dalam JavaScript...",
    summary:
      "This video explains closures, their uses, and common examples in JavaScript.",
  },
  {
    id: 4,
    url: "https://example.com/video2",
    videoTitle: "Introduction to Rust Programming",
    timeLength: 900, // in seconds
    transcripts:
      "Rust is a modern programming language that ensures memory safety...",
    translate:
      "Rust adalah bahasa pemrograman modern yang menjamin keamanan memori...",
    summary:
      "An overview of Rust programming language, focusing on memory safety and performance.",
  },
  {
    id: 5,
    url: "https://example.com/video3",
    videoTitle: "Machine Learning Basics",
    timeLength: 1200, // in seconds
    transcripts: "Machine learning is a subset of artificial intelligence...",
    translate: "Pembelajaran mesin adalah bagian dari kecerdasan buatan...",
    summary: "An introduction to machine learning concepts and applications.",
  },
  {
    id: 6,
    url: "https://example.com/video4",
    videoTitle: "Building UIs with React",
    timeLength: 750, // in seconds
    transcripts:
      "React is a JavaScript library for building user interfaces...",
    translate:
      "React adalah pustaka JavaScript untuk membangun antarmuka pengguna...",
    summary:
      "This video covers the fundamentals of React and how to build a simple UI.",
  },
  {
    id: 7,
    url: "https://example.com/video5",
    videoTitle: "Understanding Databases: SQL vs NoSQL",
    timeLength: 1050, // in seconds
    transcripts: "Databases are essential for storing and managing data...",
    translate: "Basis data penting untuk menyimpan dan mengelola data...",
    summary:
      "A comparison of SQL and NoSQL databases, their use cases, and differences.",
  },
];
