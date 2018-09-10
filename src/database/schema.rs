table! {
    characters (id) {
        id -> Int8,
        data -> Jsonb,
        frecency -> Float8,
        last_update -> Timestamp,
    }
}
