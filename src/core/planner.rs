use crate::models::config::Config;
use crate::models::package::{Package, PackageRepository};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateMode {
    EntireSystem,
    OfficialOnly,
}

pub struct UpdatePlan {
    pub mode: UpdateMode,
    pub packages: Vec<Package>,
    pub ignore_list: Vec<String>,
}

impl UpdatePlan {
    pub fn build_command(&self, config: &Config) -> Vec<String> {
        let mut cmd = match self.mode {
            UpdateMode::EntireSystem => vec!["paru".to_string(), "-Syu".to_string()],
            UpdateMode::OfficialOnly => {
                vec!["sudo".to_string(), "pacman".to_string(), "-Syu".to_string()]
            }
        };

        if !self.ignore_list.is_empty() {
            cmd.push("--ignore".to_string());
            cmd.push(self.ignore_list.join(","));
        }

        cmd.extend(config.behavior.extra_args.iter().cloned());
        cmd
    }

    pub fn execute(&self, config: &Config) -> std::io::Result<std::process::ExitStatus> {
        let cmd_parts = self.build_command(config);
        let program = &cmd_parts[0];
        let args = &cmd_parts[1..];

        std::process::Command::new(program)
            .args(args)
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .status()
    }
}

pub fn create_plan(
    mode: UpdateMode,
    all_packages: Vec<Package>,
    excluded: Vec<String>,
) -> UpdatePlan {
    let packages = match mode {
        UpdateMode::EntireSystem => all_packages,
        UpdateMode::OfficialOnly => all_packages
            .into_iter()
            .filter(|p| p.repository == PackageRepository::Official)
            .collect(),
    };

    UpdatePlan {
        mode,
        packages,
        ignore_list: excluded,
    }
}
