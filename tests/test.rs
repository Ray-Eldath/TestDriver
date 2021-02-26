use test_driver::drive;

struct Subject {
    field: i32
}

impl Subject {
    fn call(&self) -> i32 { 42 }
}

impl From<i32> for Subject {
    fn from(_: i32) -> Self {
        Subject { field: 42 }
    }
}

#[test]
fn test() {
    drive!(Subject with 56,
            assert {
                eq [call(), 42],
                eq [field, 42]
            });

    drive!(String with "abc",
            assert {
                ne [len(), 2],
                eq [len(), 4],
                eq [self, "abc"]
            });
}