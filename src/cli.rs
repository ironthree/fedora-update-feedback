use structopt::StructOpt;

/// There are some features that are configurable with the config file located at
/// ~/.config/fedora.toml.
///
/// The [FAS] section is expected to have a value for the "username" key. If this
/// is not the case, the legacy method for determining usernames (reading the
/// "~/.fedora.upn" file) is used.
///
/// The [fedora-update-feedback] section can contain values for:
///
/// check-pending = bool: Additionally queries bodhi for updates that are still pending;
/// equivalent to using the --check-pending CLI switch.
///
/// check-obsoleted = bool: Run additional checks whether obsoleted updates are installed on the
/// system; equivalent to using to the --check-obsoleted CLI switch.
///
/// check-unpushed = bool: Run additional checks whether unpushed updates are installed on the
/// system; equivalent to using the --check-unpushed CLI switch.
///
/// save-password: Try to saves the FAS password in the session keyring. To ignore
/// a password that was stored in the session keyring (for example, if you changed
/// it, or made a typo when it was prompted), use the --ignore-keyring CLI switch
/// to ask for the password again.
#[derive(Debug, StructOpt)]
pub struct Command {
    /// Override or provide FAS username
    #[structopt(long, short)]
    pub username: Option<String>,
    /// Check for installed obsolete updates
    #[structopt(long, short = "O")]
    pub check_obsoleted: bool,
    /// Include updates in "pending" state
    #[structopt(long, short = "P")]
    pub check_pending: bool,
    /// Include updates that were already commented on
    #[structopt(long, short = "c")]
    pub check_commented: bool,
    /// Include updates that were previously ignored
    #[structopt(long, short = "I")]
    pub check_ignored: bool,
    /// Check for installed unpushed updates
    #[structopt(long, short = "U")]
    pub check_unpushed: bool,
    /// Clear ignored updates
    #[structopt(long, short = "i")]
    pub clear_ignored: bool,
    /// Ignore password stored in session keyring
    #[structopt(long)]
    pub ignore_keyring: bool,
    /// Add a package name to the list of ignored packages
    #[structopt(long, short = "A")]
    pub add_ignored_package: Option<String>,
    /// Remove a package name from the list of ignored packages
    #[structopt(long, short = "R")]
    pub remove_ignored_package: Option<String>,
    /// Print the list of ignored packages and updates
    #[structopt(long, short = "p")]
    pub print_ignored: bool,
    /// Print more progress information and command output
    #[structopt(long, short = "v")]
    pub verbose: bool,
}
