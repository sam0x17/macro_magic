#[middle_crate::use_attr]
use middle_crate::distant_re_export_attr;

#[distant_re_export_attr(middle_crate::ForeignItem)]
struct AttachedItem {}

#[test]
fn test_distant_re_export_attr() {
    assert_eq!(DISTANT_ATTR_ATTACHED_ITEM, "struct AttachedItem {}");
    assert_eq!(DISTANT_ATTR_IMPORTED_ITEM, "struct ForeignItem {}");
}
