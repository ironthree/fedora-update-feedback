use clap::Parser;

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
#[derive(Debug, Parser)]
pub struct Command {
    /// Override or provide FAS username
    #[arg(long, short)]
    pub username: Option<String>,
    /// Check for installed obsolete updates
    #[arg(long, short = 'O')]
    pub check_obsoleted: bool,
    /// Include updates in "pending" state
    #[arg(long, short = 'P')]
    pub check_pending: bool,
    /// Include updates that were already commented on
    #[arg(long, short = 'c')]
    pub check_commented: bool,
    /// Include updates that were previously ignored
    #[arg(long, short = 'I')]
    pub check_ignored: bool,
    /// Check for installed unpushed updates
    #[arg(long, short = 'U')]
    pub check_unpushed: bool,
    /// Clear ignored updates
    #[arg(long, short = 'i')]
    pub clear_ignored: bool,
    /// Ignore password stored in session keyring
    #[arg(long)]
    pub ignore_keyring: bool,
    /// Add a package name to the list of ignored packages
    #[arg(long, short = 'a')]
    pub add_ignored_package: Option<String>,
    /// Remove a package name from the list of ignored packages
    #[arg(long, short = 'r')]
    pub remove_ignored_package: Option<String>,
    /// Print the list of ignored packages and updates
    #[arg(long, short = 'p')]
    pub print_ignored: bool,
    /// Print more progress information and command output
    #[arg(long, short = 'v')]
    pub verbose: bool,
}
