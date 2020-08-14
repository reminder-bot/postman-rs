table! {
    reminders (id) {
        id -> Unsigned<Integer>,
        uid -> VarChar,

        message_id -> Unsigned<Integer>,

        channel_id -> Unsigned<Integer>,

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

        content ->  VarChar,
        tts -> Bool,
        embed_id -> Nullable<Unsigned<Integer>>,

        attachment -> Nullable<Binary>,
        attachment_name -> Nullable<VarChar>,
    }
}

table! {
    embeds (id) {
        id -> Unsigned<Integer>,

        title -> VarChar,
        description -> VarChar,

        image_url -> Nullable<VarChar>,
        thumbnail_url -> Nullable<VarChar>,

        footer -> VarChar,
        footer_icon -> Nullable<VarChar>,

        color -> Unsigned<Integer>,
    }
}

table! {
    embed_fields (id) {
        id -> Unsigned<Integer>,

        title -> VarChar,
        value -> VarChar,

        inline -> Bool,

        embed_id -> Unsigned<Integer>,
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

        paused -> Bool,
        paused_until -> Nullable<Timestamp>,

        guild_id -> Nullable<Unsigned<Integer>>,
    }
}
