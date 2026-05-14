//! Chocolatey publisher — assemble a `.nuspec` + `chocolateyinstall.ps1`,
//! pack a native nupkg (OPC/ZIP), and push to the configured NuGet V2 feed.

mod install;
mod nuspec;
pub(crate) mod package;
pub(crate) mod publish;
pub mod publisher;

#[cfg(test)]
mod tests;

pub use install::{InstallScriptDual, generate_install_script, generate_install_script_dual};
pub use nuspec::{NuspecParams, generate_nuspec};
pub use publish::publish_to_chocolatey;
pub use publisher::ChocolateyPublisher;
