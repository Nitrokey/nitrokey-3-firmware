use array_tuple_concat::const_array_concat;

#[test]
fn const_context() {
    // One
    assert_eq!(
        (const { const_array_concat!(const { [1] }) }),
        [1].as_slice()
    );

    // Two
    assert_eq!(
        (const { const_array_concat!(const { [1] }, const { [] }) }),
        [1].as_slice()
    );
    assert_eq!(
        (const { const_array_concat!(const { [] }, const { [2] }) }),
        [2].as_slice()
    );
    assert_eq!(
        (const { const_array_concat!(const { [1] }, const { [2] }) }),
        [1, 2].as_slice()
    );

    // Three
    assert_eq!(
        (const { const_array_concat!(const { [1] }, const { [] }, const { [] }) }),
        [1].as_slice()
    );
    assert_eq!(
        (const { const_array_concat!(const { [] }, const { [] }, const { [3] }) }),
        [3].as_slice()
    );
    assert_eq!(
        (const { const_array_concat!(const { [1] }, const { [2] }, const { [3] }) }),
        [1, 2, 3].as_slice()
    );

    // Four
    assert_eq!(
        (const { const_array_concat!(const { [1] }, const { [] }, const { [] }, const { [] }) }),
        [1].as_slice()
    );
    assert_eq!(
        (const { const_array_concat!(const { [] }, const { [] }, const { [] }, const { [4] }) }),
        [4].as_slice()
    );
    assert_eq!(
        (const { const_array_concat!(const { [1] }, const { [2] }, const { [3] }, const { [4] }) }),
        [1, 2, 3, 4].as_slice()
    );

    // Five
    assert_eq!(
        (const {
            const_array_concat!(
                const { [1] },
                const { [] },
                const { [] },
                const { [] },
                const { [] }
            )
        }),
        [1].as_slice()
    );
    assert_eq!(
        (const {
            const_array_concat!(
                const { [] },
                const { [] },
                const { [] },
                const { [] },
                const { [5] }
            )
        }),
        [5].as_slice()
    );
    assert_eq!(
        (const {
            const_array_concat!(
                const { [1] },
                const { [2] },
                const { [3] },
                const { [4] },
                const { [5] }
            )
        }),
        [1, 2, 3, 4, 5].as_slice()
    );

    // Six
    assert_eq!(
        (const {
            const_array_concat!(
                const { [1] },
                const { [] },
                const { [] },
                const { [] },
                const { [] },
                const { [] }
            )
        }),
        [1].as_slice()
    );
    assert_eq!(
        (const {
            const_array_concat!(
                const { [] },
                const { [] },
                const { [] },
                const { [] },
                const { [] },
                const { [6] }
            )
        }),
        [6].as_slice()
    );
    assert_eq!(
        (const {
            const_array_concat!(
                const { [1] },
                const { [2] },
                const { [3] },
                const { [4] },
                const { [5] },
                const { [6] }
            )
        }),
        [1, 2, 3, 4, 5, 6].as_slice()
    );

    // Seven
    assert_eq!(
        (const {
            const_array_concat!(
                const { [1] },
                const { [] },
                const { [] },
                const { [] },
                const { [] },
                const { [] },
                const { [] }
            )
        }),
        [1].as_slice()
    );
    assert_eq!(
        (const {
            const_array_concat!(
                const { [] },
                const { [] },
                const { [] },
                const { [] },
                const { [] },
                const { [] },
                const { [7] }
            )
        }),
        [7].as_slice()
    );
    assert_eq!(
        (const {
            const_array_concat!(
                const { [1] },
                const { [2] },
                const { [3] },
                const { [4] },
                const { [5] },
                const { [6] },
                const { [7] }
            )
        }),
        [1, 2, 3, 4, 5, 6, 7].as_slice()
    );

    // Eight
    assert_eq!(
        (const {
            const_array_concat!(
                const { [1] },
                const { [] },
                const { [] },
                const { [] },
                const { [] },
                const { [] },
                const { [] },
                const { [] }
            )
        }),
        [1].as_slice()
    );
    assert_eq!(
        (const {
            const_array_concat!(
                const { [] },
                const { [] },
                const { [] },
                const { [] },
                const { [] },
                const { [] },
                const { [] },
                const { [8] }
            )
        }),
        [8].as_slice()
    );
    assert_eq!(
        (const {
            const_array_concat!(
                const { [1] },
                const { [2] },
                const { [3] },
                const { [4] },
                const { [5] },
                const { [6] },
                const { [7] },
                const { [8] }
            )
        }),
        [1, 2, 3, 4, 5, 6, 7, 8].as_slice()
    );

    // Nine
    assert_eq!(
        (const {
            const_array_concat!(
                const { [1] },
                const { [] },
                const { [] },
                const { [] },
                const { [] },
                const { [] },
                const { [] },
                const { [] },
                const { [] }
            )
        }),
        [1].as_slice()
    );
    assert_eq!(
        (const {
            const_array_concat!(
                const { [] },
                const { [] },
                const { [] },
                const { [] },
                const { [] },
                const { [] },
                const { [] },
                const { [] },
                const { [9] }
            )
        }),
        [9].as_slice()
    );
    assert_eq!(
        (const {
            const_array_concat!(
                const { [1] },
                const { [2] },
                const { [3] },
                const { [4] },
                const { [5] },
                const { [6] },
                const { [7] },
                const { [8] },
                const { [9] }
            )
        }),
        [1, 2, 3, 4, 5, 6, 7, 8, 9].as_slice()
    );

    // Ten
    assert_eq!(
        (const {
            const_array_concat!(
                const { [1] },
                const { [] },
                const { [] },
                const { [] },
                const { [] },
                const { [] },
                const { [] },
                const { [] },
                const { [] },
                const { [] }
            )
        }),
        [1].as_slice()
    );
    assert_eq!(
        (const {
            const_array_concat!(
                const { [] },
                const { [] },
                const { [] },
                const { [] },
                const { [] },
                const { [] },
                const { [] },
                const { [] },
                const { [] },
                const { [10] }
            )
        }),
        [10].as_slice()
    );
    assert_eq!(
        (const {
            const_array_concat!(
                const { [1] },
                const { [2] },
                const { [3] },
                const { [4] },
                const { [5] },
                const { [6] },
                const { [7] },
                const { [8] },
                const { [9] },
                const { [10] }
            )
        }),
        [1, 2, 3, 4, 5, 6, 7, 8, 9, 10].as_slice()
    );
}
