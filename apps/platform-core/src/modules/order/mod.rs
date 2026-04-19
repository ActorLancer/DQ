pub const MODULE: &str = "order";

pub mod api;
pub mod application;
pub mod domain;
pub mod dto;
pub mod order_templates;
pub mod repo;

#[cfg(test)]
mod tests;
