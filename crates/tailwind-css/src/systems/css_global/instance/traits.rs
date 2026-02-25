use super::*;

impl Eq for CssInstance {}

impl PartialEq<Self> for CssInstance {
    fn eq(&self, other: &Self) -> bool {
        self.selector.eq(&other.selector)
    }
}

impl PartialOrd<Self> for CssInstance {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CssInstance {
    fn cmp(&self, other: &Self) -> Ordering {
        self.selector.cmp(&other.selector)
    }
}
