pub const MODULE: &str = "delivery";

pub mod api;
pub mod application;
pub mod domain;
pub mod dto;
pub mod events;
pub mod repo;

#[cfg(test)]
mod tests;
