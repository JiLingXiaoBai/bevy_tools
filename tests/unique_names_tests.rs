use bevy_tools::UniqueNamePool;

#[test]
fn unique_name_pool_reuses_names_and_preserves_empty_name() {
    let mut pool = UniqueNamePool::default();
    let empty = pool.new_name("");
    let first = pool.new_name("Ability.Fireball");
    let second = pool.new_name("Ability.Fireball");

    assert_eq!(empty, pool.new_name(""));
    assert_eq!(first, second);
    assert_eq!(pool.get_display_str(&empty), "");
    assert_eq!(pool.get_display_str(&first), "Ability.Fireball");

    pool.clear();
    assert_eq!(pool.get_display_str(&empty), "");
    assert_eq!(pool.new_name("Ability.Fireball"), first);
}
