use argi::{cli, Argument, Command};

/// Runner for `connect` subcommand
fn connect(_ctx: &Command, _data: Option<String>) {
    todo!()
}

/// Runner for `stream --file` cli invocation
fn stream_file(_ctx: &Argument, _data: Option<String>) {
    todo!()
}

/// Runner for `stream --sound` cli invocation
fn stream_sound(_ctx: &Argument, _data: Option<String>) {
    todo!()
}

fn main() {
    cli!(
        help: "Realtime sound streaming and file sharing",
        connect [pin]: {
            help: "Connects to existing stream",
            run: (connect)
        },
        stream: {
            help: "Creates new stream with options"
            --file [path]: {
                help: "Streams a file",
                run: (stream_file)
            },
            --sound: {
                help: "Streams default desktop audio",
                run: (stream_sound)
            }
        }
    )
    .launch();
}
