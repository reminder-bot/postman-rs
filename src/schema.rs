table! {
    reminders (id) {
        id -> Unsigned<Integer>,
        uid -> VarChar,

        message_id -> Unsigned<Integer>,

        channel -> Unsigned<BigInt>,
        webhook -> Nullable<VarChar>,

        time -> Unsigned<Integer>,
        interval -> Nullable<Unsigned<Integer>>,
        enabled -> Bool,

        avatar -> VarChar,
        username -> VarChar,

        method -> Nullable<VarChar>,
    }
}

table! {
    messages (id) {
        id -> Unsigned<Integer>,

        content -> VarChar,
        embed_id -> Nullable<Unsigned<Integer>>,
    }
}

table! {
    embeds (id) {
        id -> Unsigned<Integer>,

        title -> VarChar,
        description -> VarChar,
        color -> Unsigned<Integer>,
    }
}
