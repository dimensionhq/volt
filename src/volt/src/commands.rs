use clap::{
    crate_version,
    AppSettings,
    Clap,
};

#[derive(Debug, Clap)]
#[clap(
    author = "XtremeDevX <xdx@xtremedevx.com>",
    about = "Handle your NPM packages.",
    version = crate_version!(),
    setting = AppSettings::ColoredHelp,
    setting = AppSettings::DisableHelpSubcommand,
    setting = AppSettings::DeriveDisplayOrder,
)]
pub struct VoltOpts
{
    #[clap(subcommand)]
    sub_cmd: VoltCmd,
}

#[derive(Debug, Clap)]
enum VoltCmd
{
    #[clap(
        about = "Add a package to your project dependencies.",
        version = crate_version!(),
        help_template = "volt add {version}\n\n{about-with-newline}",
        setting = AppSettings::ColoredHelp,
        setting = AppSettings::DisableHelpSubcommand,
        setting = AppSettings::DeriveDisplayOrder,
    )]
    Add(AddCmd),
}

#[derive(Debug, Clap)]
struct AddCmd
{
    package_name: String,
}
