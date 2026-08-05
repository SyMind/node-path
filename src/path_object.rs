//! Node-compatible parse and format records.

/// Node's five-field path record, usable with owned or borrowed strings.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PathObject<S = String> {
    /// Root prefix, such as `/`, `C:\\`, or a UNC share root.
    pub root: S,
    /// Directory component; when non-empty it takes precedence over `root` in `format`.
    pub dir: S,
    /// Final component; when non-empty it takes precedence over `name` plus `ext`.
    pub base: S,
    /// Extension, with a leading dot added by `format` when absent.
    pub ext: S,
    /// Filename stem used by `format` only when `base` is empty.
    pub name: S,
}

/// A parse result that borrows all fields from the original input.
pub type ParsedPath<'a> = PathObject<&'a str>;
