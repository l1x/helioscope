// src/http/ui/templates.rs
//
// Askama template structs - compiled at build time for optimal performance

use askama::Template;

use super::models::{NodeDetails, NodeSummary};

// Custom filters module - must be named `filters` for Askama to find them
mod filters {
    /// Shorten UUID for display (first 8 characters)
    pub fn shorten_uuid(uuid: &str) -> ::askama::Result<String> {
        Ok(if uuid.len() > 8 {
            format!("{}...", &uuid[..8])
        } else {
            uuid.to_string()
        })
    }

    /// Format Option<f64> memory as GB string
    pub fn format_memory(gb: &Option<f64>) -> ::askama::Result<String> {
        Ok(match gb {
            Some(v) => format!("{:.1} GB", v),
            None => "N/A".to_string(),
        })
    }

    /// Format Option<f64> temperature as Celsius string
    pub fn format_temp(temp: &Option<f64>) -> ::askama::Result<String> {
        Ok(match temp {
            Some(v) => format!("{:.1}°C", v),
            None => "N/A".to_string(),
        })
    }

    /// Format Option<String> timestamp for display
    pub fn format_timestamp(timestamp: &Option<String>) -> ::askama::Result<String> {
        Ok(match timestamp {
            Some(ts) if ts.len() >= 19 => format!("{} {}", &ts[..10], &ts[11..19]),
            Some(ts) => ts.clone(),
            None => "N/A".to_string(),
        })
    }
}

/// Home page template showing all nodes
#[derive(Template)]
#[template(path = "home.html")]
pub struct HomeTemplate<'a> {
    pub nodes: &'a [NodeSummary],
    pub version: &'a str,
}

/// Node detail page template
#[derive(Template)]
#[template(path = "node.html")]
pub struct NodeTemplate<'a> {
    pub node: &'a NodeDetails,
    pub display_name: &'a str,
    pub version: &'a str,
}

/// Error page template
#[derive(Template)]
#[template(path = "error.html")]
pub struct ErrorTemplate<'a> {
    pub title: &'a str,
    pub message: &'a str,
    pub version: &'a str,
}
