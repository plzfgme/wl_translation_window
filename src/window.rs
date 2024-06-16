use std::cell::RefCell;
use std::sync::OnceLock;

use glib::clone;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow};
use gtk4_layer_shell::LayerShell;

use crate::env_info::{collect_env_info, EnvironmentInfo};
use crate::translator::{GoogleTranslator, Translator};

fn tokio_runtime() -> &'static tokio::runtime::Runtime {
    static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| {
        tokio::runtime::Runtime::new().expect("Setting up tokio runtime needs to succeed.")
    })
}

#[derive(Debug, Clone)]
pub struct TranslationWindowConfig {
    pub src_text: String,
    pub from_lang: String,
    pub to_lang: String,
}

#[derive(Debug, Clone)]
pub struct TranslationWindow {
    config: TranslationWindowConfig,
    src_textview: glib::WeakRef<gtk4::TextView>,
    dst_textview: glib::WeakRef<gtk4::TextView>,
    translate_button: glib::WeakRef<gtk4::Button>,
    sender: RefCell<Option<tokio::sync::mpsc::Sender<anyhow::Result<String>>>>,
}

impl TranslationWindow {
    pub fn new(config: &TranslationWindowConfig) -> Self {
        Self {
            config: config.clone(),
            src_textview: glib::WeakRef::default(),
            dst_textview: glib::WeakRef::default(),
            translate_button: glib::WeakRef::default(),
            sender: RefCell::new(None),
        }
    }

    pub fn create(&self, app: &Application) {
        let window = ApplicationWindow::new(app);
        let css_provider = gtk4::CssProvider::new();
        css_provider.load_from_data(
            "window { 
                padding: 10px; 
                border-radius: 10px; 
            }",
        );
        window
            .style_context()
            .add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

        let grid = gtk4::Grid::new();
        window.set_child(Some(&grid));
        grid.set_column_homogeneous(true);
        grid.set_row_homogeneous(true);
        grid.set_column_spacing(10);
        grid.set_row_spacing(10);

        let src_textview = self.make_src_textview();
        grid.attach(&src_textview, 0, 0, 1, 4);

        let scrolled_dst_textview = self.make_dst_textview();
        grid.attach(&scrolled_dst_textview, 1, 0, 1, 4);

        let receiver = self.make_translation_channel();

        let translate_button = self.make_translate_button();
        grid.attach(&translate_button, 0, 4, 1, 1);

        let close_button = Self::make_close_button(&window);
        grid.attach(&close_button, 1, 4, 1, 1);

        self.start_one_translation();

        self.start_displaying_translations(receiver);

        let env_info = collect_env_info();
        let width = env_info.monitor_width / 4;
        let height = env_info.monitor_height / 4;
        setup_floating(&window, env_info, width, height);

        window.present();
    }

    fn start_one_translation(&self) {
        let src_textview = match self.src_textview.upgrade() {
            Some(textview) => textview,
            None => return,
        };
        let dst_textview = match self.dst_textview.upgrade() {
            Some(textview) => textview,
            None => return,
        };
        let translate_button = match self.translate_button.upgrade() {
            Some(button) => button,
            None => return,
        };
        let sender = self.sender.borrow();
        let sender = match sender.as_ref() {
            Some(sender) => sender,
            None => return,
        };
        if sender.is_closed() {
            return;
        }

        translate_button.set_sensitive(false);
        translate_button.set_label("Translating...");
        dst_textview.buffer().set_text("");
        dst_textview.style_context().remove_class("error");

        let src_text = src_textview.buffer().text(
            &src_textview.buffer().start_iter(),
            &src_textview.buffer().end_iter(),
            false,
        );
        let from_lang = &self.config.from_lang;
        let to_lang = &self.config.to_lang;

        tokio_runtime().spawn(clone!(@strong sender, @strong from_lang, @strong to_lang => async move {
            let translator = GoogleTranslator::new();
            let translated = <GoogleTranslator as Translator>::translate(&translator, &from_lang, &to_lang, &src_text).await;
            sender.send(translated).await.unwrap();
        }));
    }

    fn start_displaying_translations(
        &self,
        mut receiver: tokio::sync::mpsc::Receiver<anyhow::Result<String>>,
    ) {
        let dst_textview = match self.dst_textview.upgrade() {
            Some(textview) => textview,
            None => return,
        };
        let translate_button = match self.translate_button.upgrade() {
            Some(button) => button,
            None => return,
        };

        glib::spawn_future_local(
            clone!(@weak dst_textview, @weak translate_button => async move {
                while let Some(translated) = receiver.recv().await {
                    match translated {
                        Ok(translated) => {
                            if translated.is_empty() {
                                dst_textview.buffer().set_text("No translation found.");
                            } else {
                                dst_textview.buffer().set_text(&translated);
                            }
                        }
                        Err(err) => {
                            dst_textview.buffer().set_text(&format!("Error: {}", err));
                            dst_textview.style_context().add_class("error");
                        }
                    }

                    translate_button.set_sensitive(true);
                    translate_button.set_label("Translate");
                }
            }),
        );
    }

    fn make_src_textview(&self) -> gtk4::ScrolledWindow {
        let scrolled_src_textview = gtk4::ScrolledWindow::new();
        let css_provider = gtk4::CssProvider::new();
        css_provider.load_from_data("scrolledwindow { border: 1px solid gray; }");
        scrolled_src_textview
            .style_context()
            .add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
        let src_textview = gtk4::TextView::new();
        src_textview.set_wrap_mode(gtk4::WrapMode::Word);
        self.src_textview.set(Some(&src_textview));
        scrolled_src_textview.set_child(Some(&src_textview));
        src_textview.buffer().set_text(&self.config.src_text);

        scrolled_src_textview
    }

    fn make_dst_textview(&self) -> gtk4::ScrolledWindow {
        let scrolled_dst_textview = gtk4::ScrolledWindow::new();
        let css_provider = gtk4::CssProvider::new();
        css_provider.load_from_data("scrolledwindow { border: 1px solid gray; }");
        scrolled_dst_textview
            .style_context()
            .add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
        let dst_textview = gtk4::TextView::new();
        dst_textview.set_wrap_mode(gtk4::WrapMode::Word);
        self.dst_textview.set(Some(&dst_textview));
        let css_provider = gtk4::CssProvider::new();
        css_provider.load_from_data("textview.error { background-color: #ff0000; }");
        dst_textview
            .style_context()
            .add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
        scrolled_dst_textview.set_child(Some(&dst_textview));
        scrolled_dst_textview
    }

    fn make_translation_channel(&self) -> tokio::sync::mpsc::Receiver<anyhow::Result<String>> {
        let (sender, receiver) = tokio::sync::mpsc::channel(1);
        self.sender.replace(Some(sender));

        receiver
    }

    fn make_translate_button(&self) -> gtk4::Button {
        let translate_button = gtk4::Button::with_label("Translate");
        self.translate_button.set(Some(&translate_button));
        let translation_window = self.clone();
        translate_button.connect_clicked(move |_| translation_window.start_one_translation());

        translate_button
    }

    fn make_close_button(window: &ApplicationWindow) -> gtk4::Button {
        let close_button = gtk4::Button::with_label("Close");
        close_button.add_css_class("destructive-action");
        close_button.connect_clicked(clone!(@weak window => move |_| {
            window.close();
        }));

        close_button
    }
}

fn setup_floating(
    window: &ApplicationWindow,
    env_info: EnvironmentInfo,
    max_width: i32,
    max_height: i32,
) {
    window.init_layer_shell();
    window.set_anchor(gtk4_layer_shell::Edge::Top, true);
    window.set_anchor(gtk4_layer_shell::Edge::Right, true);
    window.set_anchor(gtk4_layer_shell::Edge::Left, true);
    window.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::OnDemand);
    window.set_layer(gtk4_layer_shell::Layer::Overlay);

    let (margin_top, margin_right, margin_bottom, margin_left) =
        calculate_margins(&env_info, max_width, max_height);
    window.set_margin(gtk4_layer_shell::Edge::Top, margin_top);
    window.set_margin(gtk4_layer_shell::Edge::Right, margin_right);
    window.set_margin(gtk4_layer_shell::Edge::Bottom, margin_bottom);
    window.set_margin(gtk4_layer_shell::Edge::Left, margin_left);
}

fn calculate_margins(env_info: &EnvironmentInfo, width: i32, height: i32) -> (i32, i32, i32, i32) {
    let margin_top;
    let margin_right;
    let margin_bottom;
    let margin_left;

    if env_info.monitor_width - env_info.pointer_x >= width {
        // Right
        margin_right = env_info.monitor_width - env_info.pointer_x - width;
        margin_left = env_info.pointer_x;
    } else if env_info.pointer_x < width {
        // Still right but shrink width
        margin_right = 0;
        margin_left = env_info.pointer_x;
    } else {
        // Left
        margin_right = env_info.monitor_width - env_info.pointer_x;
        margin_left = env_info.pointer_x - width;
    }

    if env_info.monitor_height - env_info.pointer_y >= height {
        // Down
        margin_bottom = env_info.monitor_height - env_info.pointer_y - height;
        margin_top = env_info.pointer_y;
    } else if env_info.pointer_y < height {
        // Still down but shrink height
        margin_bottom = 0;
        margin_top = env_info.pointer_y;
    } else {
        // Up
        margin_bottom = env_info.monitor_height - env_info.pointer_y;
        margin_top = env_info.pointer_y - height;
    }

    (margin_top, margin_right, margin_bottom, margin_left)
}
