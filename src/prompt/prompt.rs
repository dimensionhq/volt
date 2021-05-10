use crate::prompt::input;
use dialoguer::theme::ColorfulTheme;
use std::io::Result;
use structopt::StructOpt;

/// Prompt that returns `true` or `false` (as strings)
#[derive(Debug, StructOpt)]
pub struct Confirm {
    /// Message for the prompt
    #[structopt(short, long)]
    pub message: String,

    /// Default value for the prompt is `true`
    #[structopt(short, long)]
    pub default: bool,
    // TODO: Validation
    // #[structopt(short, long)]
    // /// Command to validate the submitted value
    // validate: Option<String>,
}

impl Confirm {
    pub fn run(&self) -> Result<bool> {
        let theme = ColorfulTheme {
            defaults_style: console::Style::new(),
            prompt_style: console::Style::new(),
            prompt_prefix: console::style(String::from("?")).yellow().bright(),
            prompt_suffix: console::style(String::from(">")).blue().dim(),
            success_prefix: console::style(String::from("√")).green().bright(),
            success_suffix: console::style(String::from("·")).blue().dim(),
            error_prefix: console::style(String::from("❌")).bright().red(),
            error_style: console::Style::new(),
            hint_style: console::Style::new(),
            values_style: console::Style::new(),
            active_item_style: console::Style::new(),
            inactive_item_style: console::Style::new(),
            active_item_prefix: console::style(String::from("√")).bright().green(),
            inactive_item_prefix: console::style(String::from(" ")),
            checked_item_prefix: console::style(String::from("")),
            unchecked_item_prefix: console::style(String::from("")),
            picked_item_prefix: console::style(String::from("")),
            unpicked_item_prefix: console::style(String::from("")),
            inline_selections: false,
        };

        let value = dialoguer::Confirm::with_theme(&theme)
            .with_prompt(&self.message)
            .default(self.default)
            .interact()?;

        Ok(value)
    }
}

/// Prompt that takes user input and returns a string.
#[derive(Debug, StructOpt)]
pub struct Input {
    /// Message for the prompt
    #[structopt(short, long)]
    pub message: String,

    /// Default value for the prompt
    #[structopt(short, long)]
    pub default: Option<String>,

    /// Allow empty input. Conflicts with `default`
    #[structopt(short, long, conflicts_with = "default")]
    pub allow_empty: bool,
}

impl Input {
    pub fn run(&self) -> Result<String> {
        let theme = ColorfulTheme {
            defaults_style: console::Style::new(),
            prompt_style: console::Style::new(),
            prompt_prefix: console::style(String::from("?")).yellow().bright(),
            prompt_suffix: console::style(String::from(">")).blue().dim(),
            success_prefix: console::style(String::from("√")).green().bright(),
            success_suffix: console::style(String::from("·")).blue().dim(),
            error_prefix: console::style(String::from("❌")).bright().red(),
            error_style: console::Style::new(),
            hint_style: console::Style::new(),
            values_style: console::Style::new(),
            active_item_style: console::Style::new(),
            inactive_item_style: console::Style::new(),
            active_item_prefix: console::style(String::from("√")).bright().green(),
            inactive_item_prefix: console::style(String::from(" ")),
            checked_item_prefix: console::style(String::from("")),
            unchecked_item_prefix: console::style(String::from("")),
            picked_item_prefix: console::style(String::from("")),
            unpicked_item_prefix: console::style(String::from("")),
            inline_selections: false,
        };

        let mut input = input::Input::<String>::with_theme(&theme);

        input
            .with_prompt(&self.message)
            .allow_empty(self.allow_empty);

        if self.default.is_some() {
            input.default(self.default.as_ref().unwrap().to_string());
        }

        let value = input.interact_text()?;

        Ok(value)
    }
}
/// Prompt that takes user input, hides it from the terminal, and returns a string
#[derive(Debug, StructOpt)]
pub struct Secret {
    /// Message for the prompt
    #[structopt(short, long)]
    pub message: String,

    /// Enable confirmation prompt with this message
    #[structopt(short, long, requires = "error")]
    pub confirm: Option<String>,

    /// Error message when secrets doesn't match during confirmation
    #[structopt(short, long, requires = "confirm")]
    pub error: Option<String>,

    /// Allow empty secret
    #[structopt(short, long)]
    pub allow_empty: bool,
}

impl Secret {
    #[allow(dead_code)]
    pub fn run(&self) -> Result<String> {
        let theme = ColorfulTheme::default();
        let mut input = dialoguer::Password::with_theme(&theme);

        input
            .with_prompt(&self.message)
            .allow_empty_password(self.allow_empty);

        if self.confirm.is_some() {
            input.with_confirmation(self.confirm.as_ref().unwrap(), self.error.as_ref().unwrap());
        }

        let value = input.interact()?;

        Ok(value)
    }
}

/// Prompt that allows the user to select from a list of options
#[derive(Debug, StructOpt)]
pub struct Select {
    /// Message for the prompt
    #[structopt(short, long)]
    pub message: String,

    /// Enables paging. Uses your terminal size
    #[structopt(short, long)]
    pub paged: bool,

    /// Specify number of the item that will be selected by default
    #[structopt(short, long)]
    pub selected: Option<usize>,

    /// Items that can be selected
    pub items: Vec<String>,
}

impl Select {
    pub fn run(&self) -> Result<()> {
        let item_len = self.items.len();

        if item_len == 0 {
            return Ok(());
        }

        let theme = ColorfulTheme {
            defaults_style: console::Style::new(),
            prompt_style: console::Style::new().bold(),
            prompt_prefix: console::style(String::from("?")).yellow().bright(),
            prompt_suffix: console::style(String::from(">")).blue().dim(),
            success_prefix: console::style(String::from("√")).green().bright(),
            success_suffix: console::style(String::from("·")).blue().dim(),
            error_prefix: console::style(String::from("❌")).bright().red(),
            error_style: console::Style::new(),
            hint_style: console::Style::new().bold(),
            values_style: console::Style::new(),
            active_item_style: console::Style::new(),
            inactive_item_style: console::Style::new(),
            active_item_prefix: console::style(String::from("√")).bright().green(),
            inactive_item_prefix: console::style(String::from(" ")),
            checked_item_prefix: console::style(String::from("")),
            unchecked_item_prefix: console::style(String::from("")),
            picked_item_prefix: console::style(String::from("")),
            unpicked_item_prefix: console::style(String::from("")),
            inline_selections: false,
        };

        let mut input = dialoguer::Select::with_theme(&theme);

        input
            .with_prompt(&self.message)
            .paged(self.paged)
            .items(&self.items);

        if self.selected.is_some() {
            input.default(self.selected.unwrap() - 1);
        }

        input.interact()?;

        Ok(())
    }
}
