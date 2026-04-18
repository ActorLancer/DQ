pub mod api;
pub mod domain;
pub mod repository;
pub mod router;
pub mod service;
pub mod standard_scenarios;
#[cfg(test)]
mod tests;

pub const MODULE: &str = "catalog";
