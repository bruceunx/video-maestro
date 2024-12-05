import * as React from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import "./App.css";
import CaptionCheckBox from "./components/CaptionCheckBox";
import VideoItems from "./components/VideoItems";
import { VideoData } from "./types/db";
import { videoList } from "./data";

const StreamText = ({ content }: { content: string }) => {
  return (
    <div>
      <ReactMarkdown remarkPlugins={[remarkGfm]}>{content}</ReactMarkdown>
    </div>
  );
};

function App() {
  const [progressState, setProgress] = React.useState("");
  const [url, setUrl] = React.useState("");
  const [content, setContent] = React.useState("");
  const [useCaption, setUseCaption] = React.useState(true);

  async function test_sql(): Promise<number> {
    const video: VideoData = await invoke("create_video", {
      url: "http://example1.com",
      title: "just test video link",
    });
    console.log(video);
    return video.id;
  }

  async function parse_and_summarize() {
    test_sql();
    // check if select lang, if select, then download vtt directly
    // setGreetMsg(await invoke("run_yt_vtt", { url, lang }));
    console.log(useCaption);
    if (url.trim().length === 0) return;
    setContent("");
    setProgress("");
    try {
      const result_msg = await invoke("run_yt", { url });
      setProgress(result_msg as string);
    } catch (error) {
      setProgress(`Error from ${error}`);
    }
  }

  const handleChecked = (checked: boolean) => {
    setUseCaption(checked);
    console.log(checked);
  };

  React.useEffect(() => {
    const unlisten = listen("stream", (event) => {
      setContent((prevContent) => prevContent + event.payload);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  return (
    <>
      <div className="flex h-screen w-screen">
        <div className="flex flex-col w-64 h-full bg-zinc-700 text-white justify-stretch">
          <VideoItems items={videoList} />
        </div>
        <div id="main" className="flex flex-col bg-zinc-500 w-full">
          <div
            id="header-bar"
            className="flex flex-row justify-center space-x-10 w-full mx-auto py-2 bg-zinc-700"
          >
            <input
              id="url-input"
              className="p-2 rounded-md w-72"
              onChange={(e) => setUrl(e.currentTarget.value)}
              placeholder="Enter a video url..."
            />
            <CaptionCheckBox handleChecked={handleChecked} />
            <button
              type="submit"
              className="bg-blue-500 text-white p-2 rounded-md hover:bg-blue-700 active:bg-blue-900"
              onClick={parse_and_summarize}
            >
              Summarize
            </button>
          </div>

          <p>{progressState}</p>
          <StreamText content={content} />
        </div>
      </div>
    </>
  );
}

export default App;
