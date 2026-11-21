// ABOUTME: Pins the public Snowflake node validation API and inclusive bounds.
// ABOUTME: Guards behavior while validator becomes an optional Snowflake dependency.

#![cfg(feature = "snowflake")]

use pretty_assertions::assert_eq;
use tagid::MachineNode;
use validator::{Validate, ValidationErrors};

static_assertions::assert_impl_all!(MachineNode: Validate);

#[test]
fn accepts_inclusive_bounds() {
    for (machine_id, node_id) in [(0, 0), (0, 31), (31, 0), (31, 31)] {
        let result: Result<MachineNode, ValidationErrors> = MachineNode::new(machine_id, node_id);
        let node = result.expect("inclusive bounds must be valid");
        assert_eq!(node.machine_id, machine_id);
        assert_eq!(node.node_id, node_id);
    }
}

#[test]
fn rejects_each_out_of_range_field() {
    for invalid in [i32::MIN, -1, 32, i32::MAX] {
        for (machine_id, node_id, field) in [(invalid, 0, "machine_id"), (0, invalid, "node_id")] {
            let error = MachineNode::new(machine_id, node_id).expect_err("outside five-bit range");
            let fields = error.field_errors();
            assert_eq!(fields.len(), 1);
            assert_eq!(fields[field].len(), 1);
            assert_eq!(fields[field][0].code, "range");
        }
    }
}

#[test]
fn reports_both_invalid_fields() {
    let error = MachineNode::new(-1, 32).expect_err("both fields are invalid");
    let fields = error.field_errors();
    assert_eq!(fields.len(), 2);
    assert_eq!(fields["machine_id"][0].code, "range");
    assert_eq!(fields["node_id"][0].code, "range");
}

#[test]
fn preserves_direct_validation_and_default() {
    let default = MachineNode::default();
    assert_eq!((default.machine_id, default.node_id), (1, 1));
    assert!(default.validate().is_ok());
    let invalid = MachineNode {
        machine_id: 32,
        node_id: -1,
    };
    let error = invalid.validate().expect_err("public Validate impl");
    assert_eq!(error.field_errors().len(), 2);
}
