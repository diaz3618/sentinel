pub mod config;
pub mod mem;
pub mod procinfo;
pub mod reserve;
pub mod policy;
pub mod actions;
pub mod psi;
pub mod cgroups;

#[cfg(test)]
mod config_test;
#[cfg(test)]
mod policy_test;
#[cfg(test)]
mod reserve_test;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
