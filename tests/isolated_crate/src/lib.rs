#![cfg(test)]

use middle_crate::distant_re_export_attr;

use middle_crate::distant_re_export_proc;

#[distant_re_export_attr(middle_crate::ForeignItem)]
struct AttachedItem {
    // TODO: implement as UI test maybe
    // Uncomment and see the error message with span
    // foo: unresolved_path
}

#[test]
fn test_distant_re_export_attr() {
    assert_eq!(DISTANT_ATTR_ATTACHED_ITEM, "struct AttachedItem {}");
    assert_eq!(DISTANT_ATTR_IMPORTED_ITEM, "struct ForeignItem {}");
}

#[test]
fn test_distant_re_export_proc() {
    let tokens_str = distant_re_export_proc!(middle_crate::ForeignItem);
    assert_eq!(tokens_str, "struct ForeignItem {}");
}
