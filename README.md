# wl_translation_window

Translate the text (with google translate) and display it in a window under cursor, works on wayland.

Only tested on sway.

## Usage

```text
Usage: wl_translation_window [OPTIONS] --from-lang <FROM_LANG> --to-lang <TO_LANG>

Options:
  -f, --from-lang <FROM_LANG>  Language code (https://cloud.google.com/translate/docs/languages) to translate from
  -t, --to-lang <TO_LANG>      Language code (https://cloud.google.com/translate/docs/languages) to translate to
  -s, --src-text <SRC_TEXT>    Text to translate, if not provided, stdin will be used
  -h, --help                   Print help
  -V, --version                Print version
```

You can bind something like below to a shortcut in your desktop environment.

Combine with [wl-clipboard](https://github.com/bugaevc/wl-clipboard) to translate text from clipboard.

```sh
wl-paste | wl_translation_window --from-lang en --to-lang zh-CN
```

Combine with [grimshot](https://github.com/OctopusET/sway-contrib) and [tesseract](https://github.com/tesseract-ocr/tesseract) to translate text from screenshot.

```sh
grimshot save area - | tesseract stdin stdout | wl_translation_window --from-lang en --to-lang zh-CN
```
