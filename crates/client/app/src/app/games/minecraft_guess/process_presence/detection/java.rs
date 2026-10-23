use super::argument_file::ArgumentFile;
use std::ffi::OsString;
use std::path::Path;

pub(super) struct JavaLaunch {
    arguments: Vec<String>,
    unreadable: bool,
    expand_files: bool,
}

impl JavaLaunch {
    pub(super) fn from_process(arguments: &[OsString], cwd: Option<&Path>) -> Self {
        let mut launch = Self {
            arguments: Vec::new(),
            unreadable: false,
            expand_files: true,
        };
        for argument in arguments.iter().skip(1) {
            let argument = argument.to_string_lossy();
            if launch.expand_files
                && let Some(filename) = argument.strip_prefix('@')
                && !filename.is_empty()
                && !filename.starts_with('@')
            {
                match ArgumentFile::read(Path::new(filename), cwd) {
                    Some(arguments) => {
                        // Java does not recursively expand @files inside argument files.
                        for argument in arguments {
                            launch.push(argument);
                        }
                    }
                    None => launch.unreadable = true,
                }
            } else {
                launch.push(argument.into_owned());
            }
        }
        launch
    }

    pub(super) fn minecraft_client(&self) -> Option<bool> {
        let detected = self.client_entry_point()
            || self.forge_client()
            || (self.has_option("--assetsdir") && self.has_option("--username"))
            // Advisory heuristic requested for launcher wrappers that hide the main class.
            || self.arguments.iter().any(|argument| argument.contains("lwjgl"));
        if detected {
            Some(true)
        } else {
            (!self.unreadable).then_some(false)
        }
    }

    fn push(&mut self, argument: String) {
        let argument = argument.to_ascii_lowercase();
        if argument == "--disable-@files" {
            self.expand_files = false;
        }
        self.arguments.push(argument);
    }

    fn client_entry_point(&self) -> bool {
        self.arguments.iter().any(|argument| {
            matches!(
                argument.as_str(),
                "net.minecraft.client.main.main"
                    | "net.minecraft.client.minecraft"
                    | "net.fabricmc.loader.impl.launch.knot.knotclient"
                    | "net.fabricmc.loader.launch.knot.knotclient"
                    | "org.quiltmc.loader.impl.launch.knot.knotclient"
                    // Prism / MultiMC / PolyMC supply the game entry point through stdin.
                    | "org.prismlauncher.entrypoint"
                    | "org.multimc.entrypoint"
                    | "org.polymc.entrypoint"
                    // HMCL's MultiMC pack wrapper invokes the game main class itself.
                    | "org.jackhuang.hmcl.hmclmultimcbootstrap"
            )
        })
    }

    fn forge_client(&self) -> bool {
        self.arguments.iter().enumerate().any(|(index, argument)| {
            let target = argument.strip_prefix("--launchtarget=").or_else(|| {
                (argument == "--launchtarget")
                    .then(|| self.arguments.get(index + 1).map(String::as_str))
                    .flatten()
            });
            matches!(target, Some("forgeclient" | "neoforgeclient" | "fmlclient"))
        })
    }

    fn has_option(&self, option: &str) -> bool {
        self.arguments.iter().any(|argument| {
            argument == option
                || argument
                    .strip_prefix(option)
                    .is_some_and(|suffix| suffix.starts_with('='))
        })
    }
}
