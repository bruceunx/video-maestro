import * as React from "react";
import { invoke } from "@tauri-apps/api/core";

import { listen } from "@tauri-apps/api/event";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import "./App.css";

const StreamText = () => {
  const [content, setContent] = React.useState("");

  React.useEffect(() => {
    const unlisten = listen("stream", (event) => {
      setContent((prevContent) => prevContent + event.payload);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  return (
    <div>
      <ReactMarkdown remarkPlugins={[remarkGfm]}>{content}</ReactMarkdown>
    </div>
  );
};

function App() {
  const [greetMsg, setGreetMsg] = React.useState("");
  const [url, setUrl] = React.useState("");
  const [lang, setLang] = React.useState("en");

  async function greet() {
    // setGreetMsg(await invoke("run_yt_vtt", { url, lang }));
    setGreetMsg(await invoke("run_yt", { url }));
  }

  return (
    <main className="container">
      <h1>Welcome to Tauri + React</h1>

      <form
        className="row"
        onSubmit={(e) => {
          e.preventDefault();
          greet();
        }}
      >
        <input
          id="greet-input"
          onChange={(e) => setUrl(e.currentTarget.value)}
          placeholder="Enter a url..."
        />
        <input
          id="lang-input"
          onChange={(e) => setLang(e.currentTarget.value)}
          placeholder="Enter a lang"
        />

        <button type="submit">Greet</button>
      </form>
      <p>{greetMsg}</p>
      <StreamText />
    </main>
  );
}

export default App;
