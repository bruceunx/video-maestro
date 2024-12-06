import * as React from "react";
import { Languages, Captions, FileText } from "lucide-react";

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import "./App.css";
import CaptionCheckBox from "./components/CaptionCheckBox";
import VideoItems from "./components/VideoItems";
import { useToast } from "hooks/ToastProvider";
import SettingsModal from "components/SettingsModal";
import StreamText from "components/StreamText";
import { useVideoData } from "store/DataContext";

function App() {
  const [url, setUrl] = React.useState("");

  const { setInProgress, currentVideo, fetchVideos } = useVideoData();

  const [content, setContent] = React.useState("");
  const [summary, setSummary] = React.useState("");
  const [useCaption, setUseCaption] = React.useState(true);

  const { addToast } = useToast();

  React.useEffect(() => {
    if (currentVideo !== null) {
      setContent(currentVideo.transcripts || "");
      setSummary(currentVideo.summary || "");
    }
  }, [currentVideo]);

  async function handle_transcript() {
    // test_sql();
    // check if select lang, if select, then download vtt directly
    // setGreetMsg(await invoke("run_yt_vtt", { url, lang }));
    console.log(useCaption);
    setContent("");
    try {
      let parse_url;
      let input_id = -1;
      if (currentVideo !== null && currentVideo.transcripts === null) {
        parse_url = currentVideo.url;
        input_id = currentVideo.id;
      } else {
        if (url.trim().length === 0) return;
        parse_url = url.trim();
      }
      const result_msg = await invoke("run_yt", { url: parse_url, input_id });

      addToast({
        message: result_msg as string,
        variant: "success",
        duration: 5000,
      });
      fetchVideos();
    } catch (error) {
      addToast({
        message: error as string,
        variant: "error",
        duration: 5000,
      });
    }
  }

  async function handle_summary() {
    if (currentVideo === null || currentVideo.transcripts === null) return;
    try {
      const result_msg = await invoke("run_summary", {
        context: currentVideo.transcripts,
        video_id: currentVideo.id,
      });
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

  async function handle_translate() {
    console.log("translate");
  }

  const handleChecked = (checked: boolean) => {
    setUseCaption(checked);
  };

  React.useEffect(() => {
    const unlisten = listen("stream", (event) => {
      if (event.payload === "[start]") {
        setInProgress(true);
        setContent("");
      } else if (event.payload === "[end]") {
        setInProgress(false);
        addToast({
          message: "数据流完成",
          variant: "success",
          duration: 5000,
        });
      } else {
        setContent((prevContent) => prevContent + event.payload);
      }
    });

    const unlisten_summary = listen("summary", (event) => {
      if (event.payload === "[start]") {
        setInProgress(true);
        setSummary("");
      } else if (event.payload === "[end]") {
        setInProgress(false);
        addToast({
          message: "数据流完成",
          variant: "success",
          duration: 5000,
        });
      } else {
        setSummary((prevContent) => prevContent + event.payload);
      }
    });

    return () => {
      unlisten.then((fn) => fn());
      unlisten_summary.then((fn) => fn());
    };
  }, []);

  return (
    <>
      <div className="flex h-screen w-screen">
        <div className="flex flex-col w-64 h-full bg-zinc-700 text-white justify-stretch">
          <SettingsModal />
          <VideoItems />
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
              onClick={handle_transcript}
            >
              <Captions className="w-7 h-7" />
              <span>Transcripts</span>
            </button>
          </div>

          <div className="flex flex-row justify-between items-stretch w-full overflow-hidden h-full">
            <div className="w-1/2 overflow-y-auto">
              <StreamText content={content} />
            </div>
            <div className="flex flex-col w-1/2 h-full">
              <div className="flex bg-zinc-600 py-2 justify-center items-center gap-7">
                <button
                  onClick={handle_translate}
                  className="flex items-center space-x-2 px-4 py-2 
                              bg-blue-500 text-white rounded-lg 
                              hover:bg-blue-600 active:bg-blue-700"
                >
                  <Languages className="w-7 h-7" />
                  <span>Translate</span>
                </button>
                <button
                  onClick={handle_summary}
                  className="flex items-center space-x-2 px-4 py-2 
                              bg-green-500 text-white rounded-lg 
                              hover:bg-green-600 active:bg-green-700"
                >
                  <FileText className="w-7 h-7" />
                  <span>Summary</span>
                </button>
              </div>
              <div className="flex-1 overflow-y-auto bg-zinc-300 h-full">
                <StreamText content={summary} />
              </div>
            </div>
          </div>
        </div>
      </div>
    </>
  );
}

export default App;
