#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Site {
    id: u32,
    url: String,
    is_accurate: bool,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Data {
    id: u32,
    site_id: u32,
    last_updated: u32,
    html: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Visit {
    id: u32,
    site_id: u32,
    bounce_rate: u32,
}