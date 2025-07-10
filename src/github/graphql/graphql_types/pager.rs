use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageInfo {
    #[serde(rename = "hasNextPage")]
    pub has_next_page: bool,
    #[serde(rename = "endCursor")]
    pub end_cursor: Option<String>,
}

impl From<PageInfo> for crate::types::SearchResultPager {
    fn from(page_info: PageInfo) -> Self {
        Self {
            next_page_cursor: page_info.end_cursor.map(crate::types::SearchCursor),
            has_next_page: page_info.has_next_page,
        }
    }
}
