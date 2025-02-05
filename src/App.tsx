import * as React from "react";
import { Captions, FileText, XIcon } from "lucide-react";

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import "./App.css";
import CaptionCheckBox from "./components/CaptionCheckBox";
import VideoItems from "./components/VideoItems";
import { useToast } from "hooks/ToastProvider";
import SettingsModal from "components/SettingsModal";
import StreamText from "components/StreamText";
import { useVideoData } from "store/DataContext";
import LanguageSelector from "components/LanguageSelector";
import { formatDate } from "utils/files";

function App() {
  const [url, setUrl] = React.useState<string>("");

  const [selectedLanguage, setSelectedLanguage] = React.useState<string>("en");
  const [imgUrl, setImgUrl] = React.useState<string | null>(null);

  const {
    setInProgress,
    currentVideo,
    fetchVideos,
    deleteVideo,
    updateCurrentVideo,
    inProgress,
  } = useVideoData();

  const [content, setContent] = React.useState<string>("");
  const [summary, setSummary] = React.useState<string>("");
  const [auto, setAuto] = React.useState<boolean>(false);

  const { addToast } = useToast();

  React.useEffect(() => {
    if (currentVideo !== null) {
      setImgUrl(null);
      setContent(currentVideo.transcripts || "");
      setSummary(currentVideo.summary || "");
      handle_new_video_image(currentVideo.thumbnail_url);
    } else {
      setContent("");
      setSummary("");
    }
  }, [currentVideo]);

  function onDeleteVideo() {
    if (currentVideo === null) return;
    deleteVideo(currentVideo.id);
  }

  async function handle_new_video_image(url: string) {
    try {
      const imageBytes: ArrayBufferLike = await invoke("fetch_image", {
        url,
      });
      const blob = new Blob([new Uint8Array(imageBytes)], {
        type: "image/png",
      });
      const imageUrl = URL.createObjectURL(blob);
      setImgUrl(imageUrl);
    } catch (error) {
      console.error("Failed to load image:", error);
    }
  }

  async function handle_transcript() {
    try {
      let parse_url: string;
      let input_id = -1;
      if (currentVideo !== null && currentVideo.transcripts === null) {
        parse_url = currentVideo.video_id;
        input_id = currentVideo.id;
      } else {
        if (url.trim().length === 0) return;
        parse_url = url.trim();
        updateCurrentVideo(-1);
      }
      setInProgress(true);
      await invoke("run_yt", { url: parse_url, input_id });
      fetchVideos();
    } catch (error) {
      const error_msg = error as string;
      if (error_msg.includes("403")) {
        addToast({
          message: `please try again later with this video platform!!!, error: ${error_msg}`,
          variant: "error",
          duration: 10000,
        });
      } else {
        addToast({
          message: error as string,
          variant: "error",
          duration: 5000,
        });
      }
    } finally {
      setInProgress(false);
    }
  }

  async function handle_summary() {
    if (currentVideo === null || currentVideo.transcripts === null) return;
    try {
      setInProgress(true);
      await invoke("run_summary", {
        video_id: currentVideo.id,
        language: selectedLanguage,
        auto: auto,
      });
      fetchVideos(false);
    } catch (error) {
      addToast({
        message: error as string,
        variant: "error",
        duration: 5000,
      });
    } finally {
      setInProgress(false);
    }
  }

  React.useEffect(() => {
    const unlisten = listen("stream", (event) => {
      if (event.payload === "[start]") {
        setInProgress(true);
        setContent("");
        setSummary("");
      } else if (event.payload === "[end]") {
        setInProgress(false);
        addToast({
          message: "Stream ended successfully",
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
        addToast({
          message: "Stream ended successfully",
          variant: "success",
          duration: 5000,
        });
      } else {
        setSummary((prevContent) => prevContent + event.payload);
      }
    });

    const unlisten_state = listen("state", (event) => {
      if (event.payload === "update video") {
        fetchVideos();
      }
    });

    return () => {
      unlisten.then((fn) => fn());
      unlisten_summary.then((fn) => fn());
      unlisten_state.then((fn) => fn());
    };
  }, [setInProgress, addToast, fetchVideos]);

  React.useEffect(() => {
    const handleContextMenu = (event: MouseEvent) => {
      if (event instanceof MouseEvent) {
        const target = event.target as HTMLElement;
        // Allow default context menu on input fields and text areas
        if (
          target.tagName.toLowerCase() === "input" ||
          target.tagName.toLowerCase() === "textarea" ||
          target.isContentEditable
        ) {
          return;
        }
      }
    };

    document.addEventListener("contextmenu", handleContextMenu);

    return () => {
      document.removeEventListener("contextmenu", handleContextMenu);
    };
  }, []);

  return (
    <>
      <div className="flex h-screen w-screen">
        <div className="flex flex-col w-64 h-full bg-zinc-700 text-white justify-stretch">
          <div className="flex justify-between">
            <SettingsModal />
            <button
              type="button"
              className="p-2 rounded-full transition-colors focus:outline-none"
              onClick={onDeleteVideo}
            >
              <XIcon className="w-6 h-6 text-gray-500 hover:text-gray-400 active:text-gray-300" />
            </button>
          </div>

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
            <button
              type="button"
              className="flex items-center space-x-2 px-4 py-2
                          bg-purple-500 text-white rounded-lg
                          hover:bg-purple-600 active:bg-purple-700 disabled:bg-purple-300 disabled:text-gray-500 disabled:cursor-default"
              onClick={handle_transcript}
              disabled={inProgress}
            >
              <Captions className="w-7 h-7" />
              <span>Transcript</span>
            </button>
          </div>

          <div className="flex flex-row justify-between items-stretch w-full overflow-hidden h-full">
            <div className="w-1/2 overflow-y-auto h-full">
              {currentVideo && (
                <>
                  <h2 className="text-center text-xl text-gray-700">
                    {currentVideo.title}
                  </h2>
                  <p className="text-right text-sm pr-2 text-gray-700">
                    {formatDate(currentVideo.upload_date)}
                  </p>
                  {imgUrl && (
                    <img
                      src={imgUrl}
                      className="mx-auto w-70 h-40 rounded-lg"
                      alt="thumbnail"
                    />
                  )}
                </>
              )}
              <StreamText content={content} />
            </div>
            <div className="flex flex-col w-1/2 h-full">
              <div className="flex bg-zinc-600 py-2 justify-center items-center gap-7">
                <LanguageSelector
                  selectedLanguage={selectedLanguage}
                  onLanguageChange={setSelectedLanguage}
                />
                <CaptionCheckBox ischecked={auto} handleChecked={setAuto} />
                <button
                  type="button"
                  onClick={handle_summary}
                  className="flex items-center space-x-2 px-4 py-2 
                              bg-green-500 text-white rounded-lg 
                              hover:bg-green-600 active:bg-green-700 disabled:bg-green-300 disabled:text-gray-500 disabled:cursor-default"
                  disabled={inProgress}
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
