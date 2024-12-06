# News Center -- Desktop Application

## Design

- use `yt-dlp` to download as audio fragments
- use `whisper.cpp` handle fragments and extract text from audio

## Framework

- use `tauri` to develop desktop application

      why not go on cloud, we need local whipser model and later llama model to handle content, which consume a lot a resource.

# TODO

- [ ] add handle settings in backend
- [x] build front page with tailwind and `radix-ui`
- [x] render the markdown in `reactjs`
- [x] add choices for snapshot or save the summary and the title to a database
