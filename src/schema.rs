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

        method -> VarChar,
    }
}

table! {
    messages (id) {
        id -> Unsigned<Integer>,

        content -> VarChar,
        embed -> Nullable<Integer>,
    }
}

table! {
    embeds (id) {
        id -> Unsigned<Integer>,

        title -> VarChar,
        description -> VarChar,
        color -> VarChar,
    }
}
