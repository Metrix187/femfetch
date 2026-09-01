pub struct AsciiArt {
    pub name: &'static str,
    pub small: &'static str,
    pub medium: &'static str,
    pub large: &'static str,
}

const PRESETS: &[AsciiArt] = &[
    AsciiArt {
        name: "bunny",
        small: " /\\_/\\\n( •.• )\n /づっ",
        medium: "  /\\_/\\\n ( •.• )\n /づっ♡\n /  \\",
        large: "    /\\_/\\\n   ( •.• )\n   /づっ♡\n  /    \\\n (      )\n  \\__/\\_/",
    },
    AsciiArt {
        name: "cat",
        small: " /\\_/\\ \n( o.o )\n > ^ <",
        medium: " /\\_/\\  \n( o.o ) \n > ^ <  \n /   \\",
        large: " /\\_/\\      \n( o.o )     \n > ^ <      \n /   \\     \n(     )    \n \\___/ ",
    },
    AsciiArt {
        name: "fox",
        small: " /\\   /\\\n(  •ᴥ•  )\n \\ づづ /",
        medium: "  /\\     /\\\n /  \\___/  \\\n(    •ᴥ•    )\n \\  づ♡づ  /",
        large: "    /\\       /\\\n   /  \\_____/  \\\n  /             \\\n (      •ᴥ•      )\n  \\    づ♡づ    /\n   \\___________/",
    },
    AsciiArt {
        name: "wolf",
        small: " /\\_/\\\n( ᵔᴥᵔ )\n / づづ",
        medium: "  /\\___/\\\n (  ᵔᴥᵔ  )\n /   づ♡づ \\",
        large: "    /\\_____/\\\n   /         \\\n  (    ᵔᴥᵔ    )\n  /    づ♡づ    \\\n /_____________\\",
    },
    AsciiArt {
        name: "whale",
        small: "  __\n<(o )___\n ( ._> /",
        medium: "   __\n<(o )___\n ( ._> /\n  `---'",
        large: "    __\n <(o )___\n  ( ._> /\n   `---'\n  ~~~~~~~",
    },
    AsciiArt {
        name: "hot-wolf",
        small: " /\\_/\\  ♨\n( >ᴥ< )\n / づづ",
        medium: "  /\\___/\\  ♨\n (  >ᴥ<  )\n /   づづ   \\",
        large: "    /\\_____/\\   ♨\n   /         \\\n  (    >ᴥ<    )\n  /     づづ    \\",
    },
    AsciiArt {
        name: "sleepy-wolf",
        small: " /\\_/\\\n( -ᴥ- ) z\n / づづ",
        medium: "  /\\___/\\\n (  -ᴥ-  ) zZ\n /   づづ   \\",
        large: "    /\\_____/\\\n   /         \\\n  (    -ᴥ-    ) zZ\n  /     づづ    \\",
    },
    AsciiArt {
        name: "alert-wolf",
        small: " /\\_/\\ !\n( •ᴥ• )\n / づづ",
        medium: "  /\\___/\\ !\n (  •ᴥ•  )\n /   づづ   \\",
        large: "    /\\_____/\\ !\n   /         \\\n  (    •ᴥ•    )\n  /     づづ    \\",
    },
];

pub fn names() -> impl Iterator<Item = &'static str> {
    PRESETS.iter().map(|preset| preset.name)
}

pub fn contains(name: &str) -> bool {
    PRESETS
        .iter()
        .any(|preset| preset.name.eq_ignore_ascii_case(name))
}

pub fn get_ascii(name: &str, size: &str) -> Option<String> {
    PRESETS
        .iter()
        .find(|preset| preset.name.eq_ignore_ascii_case(name))
        .map(|preset| match size {
            "small" => preset.small,
            "large" => preset.large,
            _ => preset.medium,
        })
        .map(str::to_string)
}
