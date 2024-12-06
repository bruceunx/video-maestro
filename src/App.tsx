import * as React from "react";
import { Languages, Captions, FileText } from "lucide-react";

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import "./App.css";
import CaptionCheckBox from "./components/CaptionCheckBox";
import VideoItems from "./components/VideoItems";
import { videoList, MarkdownContent } from "./data";
import { useToast } from "hooks/ToastProvider";
import SettingsModal from "components/SettingsModal";
import StreamText from "components/StreamText";

function App() {
  const [url, setUrl] = React.useState("");
  const [content, setContent] = React.useState("");
  const [useCaption, setUseCaption] = React.useState(true);

  const { addToast } = useToast();

  async function parse_and_summarize() {
    // test_sql();
    // check if select lang, if select, then download vtt directly
    // setGreetMsg(await invoke("run_yt_vtt", { url, lang }));
    console.log(useCaption);
    if (url.trim().length === 0) return;
    setContent("");
    try {
      const result_msg = await invoke("run_yt", { url });
      addToast({
        message: result_msg as string,
        variant: "success",
        duration: 5000,
      });
    } catch (error) {
      addToast({
        message: error as string,
        variant: "error",
        duration: 5000,
      });
    }
  }

  const handleChecked = (checked: boolean) => {
    setUseCaption(checked);
  };

  React.useEffect(() => {
    const unlisten = listen("stream", (event) => {
      if (event.payload !== "[start]" && event.payload !== "[end]")
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
          <SettingsModal />
          <VideoItems items={videoList} />
        </div>
        <div id="main" className="flex flex-col bg-gray-200 w-full">
          <div
            id="header-bar"
            className="flex flex-row justify-center space-x-10 w-full mx-auto py-2 bg-zinc-700"
          >
            <input
              id="url-input"
              className="p-2 rounded-md w-1/2 min-w-96"
              onChange={(e) => setUrl(e.currentTarget.value)}
              placeholder="Enter a video url..."
            />
            <CaptionCheckBox handleChecked={handleChecked} />
            <button
              className="flex items-center space-x-2 px-4 py-2 
                          bg-purple-500 text-white rounded-lg 
                          hover:bg-purple-600 active:bg-purple-700"
              onClick={parse_and_summarize}
            >
              <Captions className="w-7 h-7" />
              <span>Transcripts</span>
            </button>
          </div>

          <div className="flex flex-row justify-between items-stretch w-full overflow-hidden">
            <div className="w-1/2 overflow-y-auto">
              <StreamText content={content} />
            </div>
            <div className="flex flex-col w-1/2">
              <div className="flex bg-zinc-600 py-2 justify-center items-center gap-7">
                <button
                  className="flex items-center space-x-2 px-4 py-2 
                              bg-blue-500 text-white rounded-lg 
                              hover:bg-blue-600 active:bg-blue-700"
                >
                  <Languages className="w-7 h-7" />
                  <span>Translate</span>
                </button>
                <button
                  className="flex items-center space-x-2 px-4 py-2 
                              bg-green-500 text-white rounded-lg 
                              hover:bg-green-600 active:bg-green-700"
                >
                  <FileText className="w-7 h-7" />
                  <span>Summary</span>
                </button>
              </div>
              <div className="flex-1 overflow-y-auto bg-zinc-300">
                <StreamText content={MarkdownContent} />
              </div>
            </div>
          </div>
        </div>
      </div>
    </>
  );
}

export default App;
