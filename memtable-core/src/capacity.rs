/// Represents the capacity of the list
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum Capacity {
    /// Represents a capacity that has a maximum bounds
    Limited(usize),

    /// Represents a capacity with no bounds
    Unlimited,
}

impl Capacity {
    /// Returns true if the capacity is unlimited
    pub fn is_unlimited(self) -> bool {
        matches!(self, Self::Unlimited)
    }

    /// Returns true if the capacity is limited
    pub fn is_limited(self) -> bool {
        matches!(self, Self::Limited(_))
    }

    /// Returns the limit associated with the capacity if it has one
    pub fn limit(self) -> Option<usize> {
        match self {
            Self::Limited(x) => Some(x),
            _ => None,
        }
    }
}
