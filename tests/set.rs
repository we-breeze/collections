use brz_collections::{BrzHashSet, CPythonHash};

#[test]
fn membership_and_difference() {
    let mut requested = BrzHashSet::with_capacity(128);
    requested.extend([2092_i64, 2937, 237, 3814, 2040]);
    assert!(!requested.insert(2040));
    assert!(!requested.remove(&2));
    assert_eq!(requested.len(), 5);
    assert!(requested.contains(&237));

    let found: BrzHashSet<_> = [237].into_iter().collect();
    let missing = &requested - &found;
    assert_eq!(missing, [2092, 2937, 3814, 2040].into_iter().collect());
    assert!(requested.contains(&237));
    assert_eq!(missing.iter().len(), 4);

    let mut iter = missing.into_iter();
    assert!(iter.next().is_some());
    assert_eq!(iter.len(), 3);
    assert_eq!(iter.count(), 3);

    requested.clear();
    assert!(requested.is_empty());
    assert!(requested.iter().next().is_none());
}

#[test]
fn cpython_integer_hash() {
    const MODULUS: i64 = (1 << 61) - 1;
    let cases = [
        (0, 0),
        (-1, -2),
        (-2, -2),
        (MODULUS, 0),
        (MODULUS + 1, 1),
        (-MODULUS - 1, -2),
        (i64::MIN, -4),
        (i64::MAX, 3),
    ];
    for (value, hash) in cases {
        assert_eq!(value.cpython_hash(), hash);
    }
}

#[cfg(not(brz_cpython_set))]
#[test]
fn default_backend_accepts_rust_hash_keys() {
    #[derive(PartialEq, Eq, Hash)]
    struct Key(String);

    let mut set = BrzHashSet::new();
    assert!(set.insert(Key("a".to_owned())));
    assert!(set.contains(&Key("a".to_owned())));
    assert!(set.remove(&Key("a".to_owned())));
}

#[cfg(brz_cpython_set)]
#[test]
fn wegent_iteration_and_joined_difference() {
    let ids = [2092_i64, 2937, 237, 3814, 2040];
    let subscriptions: BrzHashSet<_> = ids.into_iter().collect();
    let mut requested = BrzHashSet::with_capacity(256);
    requested.extend(ids);
    let expected = [3814, 2092, 237, 2040, 2937];
    assert_eq!(requested.iter().copied().collect::<Vec<_>>(), expected);
    assert_eq!(subscriptions.into_iter().collect::<Vec<_>>(), expected);

    let found: BrzHashSet<_> = [237].into_iter().collect();
    let missing = &requested - &found;
    let joined = missing
        .iter()
        .map(i64::to_string)
        .collect::<Vec<_>>()
        .join(", ");
    assert_eq!(joined, "2040, 2937, 2092, 3814");

    let mut collided: BrzHashSet<_> = [0_i64, 8, 16].into_iter().collect();
    assert!(collided.remove(&8));
    assert!(collided.insert(24));
    assert_eq!(collided.into_iter().collect::<Vec<_>>(), [0, 24, 16]);
}
