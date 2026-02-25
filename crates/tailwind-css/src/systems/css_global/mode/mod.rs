use super::*;

#[derive(Debug, Clone, Default, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum CssInlineMode {
    /// ```html
    /// <img class="tailwind"/>
    /// ```
    #[default]
    None,
    /// ```html
    /// <img style="key:value"/>
    /// ```
    Inline,
    /// ```html
    /// <img class="_b2JmdXNjYXRl"/>
    /// ```
    Scoped,
    /// ```html
    /// <img data-tw-b2JmdXNjYXRl/>
    /// ```
    DataKey,
    /// ```html
    /// <img data-tw="b2JmdXNjYXRl"/>
    /// ```
    DataValue,
}

