import * as React from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import "./App.css";

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
  const [lang, setLang] = React.useState("en");
  const [content, setContent] = React.useState("");

  async function parse_and_summarize() {
    // setGreetMsg(await invoke("run_yt_vtt", { url, lang }));
    if (url.trim().length === 0) return;
    console.log(lang);
    setContent("");
    setProgress(await invoke("run_yt", { url }));
  }

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
      <div className="flex h-screen w-screen bg-gray-200">
        <div id="leftbar" className="flex flex-col w-60 bg-blue-200"></div>
        <div id="main" className="flex flex-col bg-red-100 w-full">
          <div
            id="header-bar"
            className="flex flex-row justify-center space-x-10 w-full mx-auto mt-5"
          >
            <input
              id="url-input"
              className="p-2 rounded-md"
              onChange={(e) => setUrl(e.currentTarget.value)}
              placeholder="Enter a url..."
            />
            <input
              id="lang-input"
              className="p-2 rounded-md"
              onChange={(e) => setLang(e.currentTarget.value)}
              placeholder="Enter the language"
            />
            <button
              type="submit"
              className="bg-blue-500 text-white p-2 rounded-md hover:bg-blue-700 active:bg-blue-900"
              onClick={parse_and_summarize}
            >
              parse
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
