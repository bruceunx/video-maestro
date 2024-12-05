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

export const MarkdownContent = `

# h1 Heading 8-)
## h2 Heading
### h3 Heading
#### h4 Heading
##### h5 Heading
###### h6 Heading


## Horizontal Rules

___

---

***


## Typographic replacements

Enable typographer option to see result.

(c) (C) (r) (R) (tm) (TM) (p) (P) +-

test.. test... test..... test?..... test!....

!!!!!! ???? ,,  -- ---

"Smartypants, double quotes" and 'single quotes'


## Emphasis

**This is bold text**

__This is bold text__

*This is italic text*

_This is italic text_

~~Strikethrough~~


## Blockquotes


> Blockquotes can also be nested...
>> ...by using additional greater-than signs right next to each other...
> > > ...or with spaces between arrows.

Unordered

+ Create a list by starting a line with \` +
  \`
+ Sub-lists are made by indenting 2 spaces:
  - Marker character change forces new list start:
    * Ac tristique libero volutpat at
    + Facilisis in pretium nisl aliquet
    - Nulla volutpat aliquam velit
+ Very easy!

Ordered

1. Lorem ipsum dolor sit amet
2. Consectetur adipiscing elit
3. Integer molestie lorem at massa


1. You can use sequential numbers...
1. ...or keep all the numbers as \`1.\`

Start numbering with offset:

57. foo
1. bar


## Code

Inline \`code\`

Indented code

    // Some comments
    line 1 of code
    line 2 of code
    line 3 of code


Block code "fences"

\`\`\`
Sample text here...
\`\`\`

Syntax highlighting

\`\`\` js
var foo = function (bar) {
  return bar++;
};

console.log(foo(5));
\`\`\`

## Tables

| Option | Description |
| ------ | ----------- |
| data   | path to data files to supply the data that will be passed into templates. |
| engine | engine to be used for processing templates. Handlebars is the default. |
| ext    | extension to be used for dest files. |

`;
