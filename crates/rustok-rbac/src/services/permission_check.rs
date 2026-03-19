use rustok_core::Permission;

#[derive(Debug, Clone, Copy)]
pub enum PermissionCheck<'a> {
    Single(&'a Permission),
    Any(&'a [Permission]),
    All(&'a [Permission]),
}
