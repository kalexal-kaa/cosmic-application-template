// SPDX-License-Identifier: GPL-3

use crate::config::Config;
use crate::fl;
use cosmic::app::context_drawer;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Alignment, Length, Subscription};
use cosmic::widget::{self, about::About, icon, menu, nav_bar};
use cosmic::{iced_futures, prelude::*};
use futures_util::SinkExt;
use std::collections::HashMap;
use std::time::Duration;
use rand::Rng;

const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const APP_ICON: &[u8] = include_bytes!("../resources/icons/hicolor/scalable/apps/icon.svg");

/// The application model stores app-specific state used to describe its interface and
/// drive its logic.
pub struct AppModel {
    /// Application state which is managed by the COSMIC runtime.
    core: cosmic::Core,
    /// Display a context drawer with the designated page if defined.
    context_page: ContextPage,
    /// The about page for this app.
    about: About,
    /// Contains items assigned to the nav bar panel.
    nav: nav_bar::Model,
    /// Key bindings for the application's menu bar.
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    /// Configuration data that persists between application runs.
    config: Config,
    /// Time active
    time: u32,
    /// Toggle the watch subscription
    watch_is_active: bool,
    value_counter: i64,
    password: String,
    secret_number: i64,
    number: String,
    feedback: String,
    attempts_counter: i64,
    attempts: String,
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    Increment,
    Decrement,
    InputPassword(String),
    ClearPassword,
    GeneratePassword,
    InputNumber(String),
    ClearNumber,
    CheckNumber,
    NewGame,
    LaunchUrl(String),
    ToggleContextPage(ContextPage),
    ToggleWatch,
    UpdateConfig(Config),
    WatchTick(u32),
}

/// Create a COSMIC application from the app model
impl cosmic::Application for AppModel {
    /// The async executor that will be used to run your application's commands.
    type Executor = cosmic::executor::Default;

    /// Data that your application receives to its init method.
    type Flags = ();

    /// Messages which the application and its widgets will emit.
    type Message = Message;

    /// Unique identifier in RDNN (reverse domain name notation) format.
    const APP_ID: &'static str = "dev.mmurphy.Test";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    /// Initializes the application with any given flags and startup commands.
    fn init(
        core: cosmic::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        // Create a nav bar with three page items.
        let mut nav = nav_bar::Model::default();

        nav.insert()
            .text(fl!("page-id", num = 1))
            .data::<Page>(Page::Page1)
            .icon(icon::from_name("applications-science-symbolic"))
            .activate();

        nav.insert()
            .text(fl!("page-id", num = 2))
            .data::<Page>(Page::Page2)
            .icon(icon::from_name("applications-system-symbolic"));

        nav.insert()
            .text(fl!("page-id", num = 3))
            .data::<Page>(Page::Page3)
            .icon(icon::from_name("applications-utilities-symbolic"));

        nav.insert()
            .text(fl!("page-id", num = 4))
            .data::<Page>(Page::Page4)
            .icon(icon::from_name("applications-games-symbolic"));

        // Create the about widget
        let about = About::default()
            .name(fl!("app-title"))
            .icon(widget::icon::from_svg_bytes(APP_ICON))
            .version(env!("CARGO_PKG_VERSION"))
            .links([(fl!("repository"), REPOSITORY)])
            .license(env!("CARGO_PKG_LICENSE"));

        // Construct the app model with the runtime's core.
        let mut app = AppModel {
            core,
            context_page: ContextPage::default(),
            about,
            nav,
            key_binds: HashMap::new(),
            // Optional configuration file for an application.
            config: cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
                .map(|context| match Config::get_entry(&context) {
                    Ok(config) => config,
                    Err((_errors, config)) => {
                        // for why in errors {
                        //     tracing::error!(%why, "error loading app config");
                        // }

                        config
                    }
                })
                .unwrap_or_default(),
            time: 0,
            watch_is_active: false,
            value_counter: 0,
            password: String::new(),
            secret_number: rand::thread_rng().gen_range(1..=100),
            number: String::new(),
            feedback: "A number from 1 to 100 is hidden. Guess it!".to_string(),
            attempts_counter: 0,
            attempts: "Number of attempts: 0".to_string(),
        };

        // Create a startup command that sets the window title.
        let command = app.update_title();

        (app, command)
    }

    /// Elements to pack at the start of the header bar.
    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        let menu_bar = menu::bar(vec![menu::Tree::with_children(
            menu::root(fl!("view")).apply(Element::from),
            menu::items(
                &self.key_binds,
                vec![menu::Item::Button(fl!("about"), None, MenuAction::About)],
            ),
        )]);

        vec![menu_bar.into()]
    }

    /// Enables the COSMIC application to create a nav bar with this model.
    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav)
    }

    /// Display a context drawer if the context page is requested.
    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<'_, Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => context_drawer::about(
                &self.about,
                |url| Message::LaunchUrl(url.to_string()),
                Message::ToggleContextPage(ContextPage::About),
            ),
        })
    }

    /// Describes the interface based on the current state of the application model.
    ///
    /// Application events will be processed through the view. Any messages emitted by
    /// events received by widgets will be passed to the update method.
    fn view(&self) -> Element<'_, Self::Message> {
        let space_s = cosmic::theme::spacing().space_s;
        let content: Element<_> = match self.nav.active_data::<Page>().unwrap() {
            Page::Page1 => {
                let header = widget::row::with_capacity(2)
                    .push(widget::text::title1(fl!("welcome")))
                    .push(widget::text::title3(fl!("page-id", num = 1)))
                    .align_y(Alignment::End)
                    .spacing(space_s);

                let counter_label = ["Watch: ", self.time.to_string().as_str()].concat();
                let section = cosmic::widget::settings::section().add(
                    cosmic::widget::settings::item::builder(counter_label).control(
                        widget::button::text(if self.watch_is_active {
                            "Stop"
                        } else {
                            "Start"
                        })
                        .on_press(Message::ToggleWatch),
                    ),
                );

                widget::column::with_capacity(2)
                    .push(header)
                    .push(section)
                    .spacing(space_s)
                    .height(Length::Fill)
                    .into()
            }

            Page::Page2 => {
                let header = widget::row::with_capacity(2)
                    .push(widget::text::title1(fl!("welcome")))
                    .push(widget::text::title3(fl!("page-id", num = 2)))
                    .align_y(Alignment::End)
                    .spacing(space_s);

                let button_minus = widget::button::text("-").on_press(Message::Decrement);
                let counter_text = widget::text::title3(self.value_counter.to_string());
                let button_plus = widget::button::text("+").on_press(Message::Increment);

                let row_counter = widget::row::with_capacity(2)
                     .push(button_minus)
                     .push(counter_text)
                     .push(button_plus)
                     .align_y(Vertical::Center)
                     .spacing(space_s);

                widget::column::with_capacity(1)
                    .push(header)
                    .push(row_counter)
                    .spacing(space_s)
                    .height(Length::Fill)
                    .into()
            }

            Page::Page3 => {
                let header = widget::row::with_capacity(2)
                    .push(widget::text::title1(fl!("welcome")))
                    .push(widget::text::title3(fl!("page-id", num = 3)))
                    .align_y(Alignment::End)
                    .spacing(space_s);

                let password_text_input = widget::text_input("Your password will be here!", self.password.clone())
                    .on_input(Message::InputPassword)
                    .on_clear(Message::ClearPassword);

                let generate_button = widget::button::text("Generate password").on_press(Message::GeneratePassword);

                let row_password = widget::row::with_capacity(2)
                    .push(password_text_input)
                    .push(generate_button)
                    .align_y(Vertical::Center)
                    .spacing(space_s);

                widget::column::with_capacity(1)
                    .push(header)
                    .push(row_password)
                    .spacing(space_s)
                    .height(Length::Fill)
                    .into()
            }

            Page::Page4 => {
                 let header = widget::row::with_capacity(2)
                    .push(widget::text::title1(fl!("welcome")))
                    .push(widget::text::title3(fl!("page-id", num = 4)))
                    .align_y(Alignment::End)
                    .spacing(space_s);

                 let number_text_input = widget::text_input("Enter your number", self.number.clone())
                    .on_input(Message::InputNumber)
                    .on_clear(Message::ClearNumber);

                 let check_button = widget::button::text("Check the number").on_press(Message::CheckNumber);

                 let row_number = widget::row::with_capacity(2)
                    .push(number_text_input)
                    .push(check_button)
                    .align_y(Vertical::Center)
                    .spacing(space_s);

                 let feedback_text = widget::text::title3(self.feedback.clone());
                 let attempts_text = widget::text::title3(self.attempts.clone());
                 let new_game_button = widget::button::text("Start a new game").on_press(Message::NewGame);

                 widget::column::with_capacity(1)
                    .push(header)
                    .push(row_number)
                    .push(feedback_text)
                    .push(attempts_text)
                    .push(new_game_button)
                    .spacing(space_s)
                    .height(Length::Fill)
                    .into()
            }
        };

        widget::container(content)
            .width(600)
            .height(Length::Fill)
            .apply(widget::container)
            .width(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .into()
    }

    /// Register subscriptions for this application.
    ///
    /// Subscriptions are long-running async tasks running in the background which
    /// emit messages to the application through a channel. They can be dynamically
    /// stopped and started conditionally based on application state, or persist
    /// indefinitely.
    fn subscription(&self) -> Subscription<Self::Message> {
        // Add subscriptions which are always active.
        let mut subscriptions = vec![
            // Watch for application configuration changes.
            self.core()
                .watch_config::<Config>(Self::APP_ID)
                .map(|update| {
                    // for why in update.errors {
                    //     tracing::error!(?why, "app config error");
                    // }

                    Message::UpdateConfig(update.config)
                }),
        ];

        // Conditionally enables a timer that emits a message every second.
        if self.watch_is_active {
            subscriptions.push(Subscription::run(|| {
                iced_futures::stream::channel(1, |mut emitter| async move {
                    let mut time = 1;
                    let mut interval = tokio::time::interval(Duration::from_secs(1));

                    loop {
                        interval.tick().await;
                        _ = emitter.send(Message::WatchTick(time)).await;
                        time += 1;
                    }
                })
            }));
        }

        Subscription::batch(subscriptions)
    }

    /// Handles messages emitted by the application and its widgets.
    ///
    /// Tasks may be returned for asynchronous execution of code in the background
    /// on the application's async runtime.
    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::Increment => {
                self.value_counter += 1;
            }
            Message::Decrement => {
                self.value_counter -= 1;
            }
            Message::InputPassword(v) => {
                self.password = v;
            }
            Message::ClearPassword => {
                self.password.clear();
            }
            Message::GeneratePassword => {
                 const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";

                 let mut rng = rand::thread_rng();

                 self.password = (0..16)
                     .map(|_| {
                         let idx = rng.gen_range(0..CHARSET.len());
                         CHARSET[idx] as char
                      })
                     .collect();
            }
            Message::InputNumber(v) => {
                self.number = v;
            }
            Message::ClearNumber => {
                self.number.clear();
            }
            Message::CheckNumber => {
                match self.number.parse::<i64>() {
                    Ok(num) => {
                        if num == self.secret_number {
                            self.feedback = format!("✅ Right! This is the number {}", self.secret_number);
                        } else if num < self.secret_number {
                            self.feedback = "⏫ My number is higher!".to_string();
                        } else {
                            self.feedback = "⏬ My number is less!".to_string();
                        }
                        self.attempts_counter += 1;
                        self.attempts = format!("Number of attempts: {}", self.attempts_counter);
                    }
                    Err(_) => self.feedback = "❌ Enter a number!".to_string(),
                }
            }
            Message::NewGame => {
                self.secret_number = rand::thread_rng().gen_range(1..=100);
                self.number.clear();
                self.attempts_counter = 0;
                self.feedback = "A new number has been guessed. Guess it!".to_string();
                self.attempts = "Number of attempts: 0".to_string();
            }
            Message::WatchTick(time) => {
                self.time = time;
            }

            Message::ToggleWatch => {
                self.watch_is_active = !self.watch_is_active;
            }

            Message::ToggleContextPage(context_page) => {
                if self.context_page == context_page {
                    // Close the context drawer if the toggled context page is the same.
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    // Open the context drawer to display the requested context page.
                    self.context_page = context_page;
                    self.core.window.show_context = true;
                }
            }

            Message::UpdateConfig(config) => {
                self.config = config;
            }

            Message::LaunchUrl(url) => match open::that_detached(&url) {
                Ok(()) => {}
                Err(err) => {
                    eprintln!("failed to open {url:?}: {err}");
                }
            },
        }
        Task::none()
    }

    /// Called when a nav item is selected.
    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<cosmic::Action<Self::Message>> {
        // Activate the page in the model.
        self.nav.activate(id);

        self.update_title()
    }
}

impl AppModel {
    /// Updates the header and window titles.
    pub fn update_title(&mut self) -> Task<cosmic::Action<Message>> {
        let mut window_title = fl!("app-title");

        if let Some(page) = self.nav.text(self.nav.active()) {
            window_title.push_str(" — ");
            window_title.push_str(page);
        }

        if let Some(id) = self.core.main_window_id() {
            self.set_window_title(window_title, id)
        } else {
            Task::none()
        }
    }
}

/// The page to display in the application.
pub enum Page {
    Page1,
    Page2,
    Page3,
    Page4,
}

/// The context page to display in the context drawer.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
        }
    }
}
