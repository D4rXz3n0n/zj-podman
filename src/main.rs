use owo_colors::OwoColorize;
use std::collections::BTreeMap;
use zellij_tile::prelude::*;

#[derive(Default)]
struct State {
    selected: usize,
    containers: Vec<Vec<String>>,
}

impl State {
    fn select_down(&mut self) {
        self.selected = (self.selected + 1) % self.containers.len();
    }

    fn select_up(&mut self) {
        if self.selected == 0 {
            self.selected = self.containers.len() - 1;
            return;
        }
        self.selected = self.selected - 1;
    }
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, _: BTreeMap<String, String>) {
        request_permission(&[
            PermissionType::RunCommands,
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
        ]);
        subscribe(&[
            EventType::Key,
            EventType::TabUpdate,
            EventType::PaneUpdate,
            EventType::RunCommandResult,
        ]);
        let args = &["podman", "ps", "-a", "--format", "{{.Names}} {{.State}}"][..];
        let envs: BTreeMap<String, String> = BTreeMap::new();
        run_command(args, envs);
    }

    fn update(&mut self, event: Event) -> bool {
        let mut should_render = false;
        let args = &["podman", "ps", "-a", "--format", "{{.Names}} {{.State}}"][..];
        let envs: BTreeMap<String, String> = BTreeMap::new();
        run_command(args, envs);
        match event {
            Event::RunCommandResult(_, _stdout, _, _) => {
                if let stdout = _stdout {
                    let output = String::from_utf8(stdout.to_vec()).unwrap_or_default();
                    self.containers = output
                        .lines()
                        .map(|s| s.split_whitespace().map(|s| s.to_string()).collect())
                        .collect();
                }
                should_render = true;
            }

            Event::PaneUpdate(pane_manifest) => {
                should_render = true;
            }
            Event::Key(key) => match key.bare_key {
                BareKey::Down | BareKey::Char('j') => {
                    if self.containers.len() > 0 {
                        self.select_down();
                        should_render = true;
                    }
                }
                BareKey::Up | BareKey::Char('k') => {
                    if self.containers.len() > 0 {
                        self.select_up();
                        should_render = true;
                    }
                }
                BareKey::Enter | BareKey::Char('l') => {
                    let container = self.containers.get(self.selected);

                    if let Some(container) = container {
                        let args = &["podman", "start", &container[0]][..];
                        let envs: BTreeMap<String, String> = BTreeMap::new();
                        run_command(args, envs);
                    }
                }
                BareKey::Char('s') => {
                    let container = self.containers.get(self.selected);

                    if let Some(container) = container {
                        let args = &["podman", "stop", &container[0]][..];
                        let envs: BTreeMap<String, String> = BTreeMap::new();
                        run_command(args, envs);
                    }
                }
                BareKey::Char('e') => {
                    let container = self.containers.get(self.selected);
                    if let Some(container) = container {
                        let args = &["podman", "exec", "-it", &container[0], "/bin/bash"][..];
                        let envs: BTreeMap<String, String> = BTreeMap::new();

                        open_command_pane(
                            CommandToRun {
                                path: "podman".into(),
                                cwd: None,
                                args: vec![
                                    "exec".to_owned(),
                                    "-it".to_owned(),
                                    container[0].clone(),
                                    "/bin/bash".to_owned(),
                                ],
                            },
                            envs,
                        );
                    }
                }
                BareKey::Char('q') => {
                    hide_self();
                }
                _ => (),
            },

            _ => (),
        };

        should_render
    }

    fn render(&mut self, _rows: usize, _cols: usize) {
        println!(
            "{}",
            self.containers
                .iter()
                .enumerate()
                .map(|(idx, container)| {
                    if let Some(name) = container.get(0)
                        && let Some(state) = container.get(1)
                    {
                        if idx == self.selected {
                            name.to_string().red().bold().to_string()
                                + " "
                                + &state.to_string().red().bold().to_string()
                        } else {
                            name.to_string() + " " + &state.to_string()
                        }
                    } else {
                        "There is no container to display".to_string()
                    }
                })
                .collect::<Vec<String>>()
                .join("\n")
        );
    }
}
