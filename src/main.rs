mod env_info;
mod translator;
mod window;

use std::io::Read;

use clap::Parser;
use gtk4::{prelude::*, Application};
use window::TranslationWindowConfig;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(
        short,
        long,
        help = "Language code (https://cloud.google.com/translate/docs/languages) to translate from"
    )]
    pub from_lang: String,
    #[arg(
        short,
        long,
        help = "Language code (https://cloud.google.com/translate/docs/languages) to translate to"
    )]
    pub to_lang: String,
    #[arg(
        short,
        long,
        help = "Text to translate, if not provided, stdin will be used"
    )]
    pub src_text: Option<String>,
}

impl From<Args> for TranslationWindowConfig {
    fn from(args: Args) -> Self {
        TranslationWindowConfig {
            src_text: args.src_text.unwrap_or_default(),
            from_lang: args.from_lang,
            to_lang: args.to_lang,
        }
    }
}

fn main() {
    let mut args = Args::parse();
    if args.src_text.is_none() {
        let mut text = String::new();
        std::io::stdin().read_to_string(&mut text).unwrap();
        args.src_text.replace(text);
    }
    let config = TranslationWindowConfig::from(args);

    let application = Application::builder()
        .application_id("com.github.plzfgme.wl_translation_window")
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();

    let window = window::TranslationWindow::new(&config);

    application.connect_activate(move |app| {
        window.create(app);
    });

    application.run_with_args(&Vec::<String>::new());
}
