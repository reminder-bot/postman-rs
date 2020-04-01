table! {
    reminders (id) {
        id -> Unsigned<Integer>,
        uid -> VarChar,

        message_id -> Unsigned<Integer>,

        channel_id -> Nullable<Unsigned<Integer>>,
        user_id -> Nullable<Unsigned<Integer>>,

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

table! {
    channels (id) {
        id -> Unsigned<Integer>,
        channel -> Unsigned<BigInt>,

        nudge -> SmallInt,
        blacklisted -> Bool,

        name -> Nullable<VarChar>,

        webhook_id -> Nullable<Unsigned<BigInt>>,
        webhook_token -> Nullable<VarChar>,

        guild_id -> Unsigned<Integer>,
    }
}

table! {
    users (id) {
        id -> Unsigned<Integer>,
        user -> Unsigned<BigInt>,
        name -> VarChar,

        language -> VarChar,
        timezone -> Nullable<VarChar>,
        allowed_dm -> Bool,

        patreon -> Bool,
        dm_channel -> Unsigned<BigInt>,
    }
}
