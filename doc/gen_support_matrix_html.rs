#!/usr/bin/env -S cargo -Zscript
---cargo
[package]
edition = "2021"

[dependencies]
argh = { version = "0.1.12" }
miette = { version = "7.2.0", features = ["fancy"] }
minijinja = { version = "2.0.3" }
serde = { version = "1.0.0", features = ["derive"] }
serde_yaml = { version = "0.9.34" } # TODO: use a maintained crate instead
thiserror = { version = "1.0.61" }
---

use std::{fs, io, path::{Path, PathBuf}};

use serde::Serialize;
use miette::Diagnostic;

const TABLE_TEMPLATE: &str =
r##"<!-- This table is auto-generated. Do not edit manually. -->
<table>
  <thead>
    <tr>
      <th>Chip</th>
      <th>Testing Board</th>
      <th colspan="{{ matrix.functionalities|length }}">Functionality</th>
    </tr>
    <tr>
      <th></th>
      <th></th>
      {%- for functionality in matrix.functionalities %}
      <th>{{ functionality.title }}</th>
      {%- endfor %}
    </tr>
  </thead>
  <tbody>
    {%- for board in boards %}
    <tr>
      <td>{{ board.chip }}</td>
      <td>{{ board.name }}</td>
      {%- for functionality in board.functionalities %}
      <td class="support-cell" title="{{ functionality.description }}">{{ functionality.icon }}</td>
      {%- endfor %}
    </tr>
    {%- endfor %}
  </tbody>
</table>
<style>
.support-cell {
  text-align: center;
}
</style>
"##;

const KEY_TEMPLATE: &str =
r##"<p>Key:</p>

<ul>
  {%- for support_key in matrix.support_keys %}
  <li class="no-marker">{{ support_key.icon }}&nbsp;&nbsp;{{ support_key.description }}</li>
  {%- endfor %}
</ul>
<style>
.no-marker::marker {
  content: '';
}
</style>
"##;

#[derive(argh::FromArgs)]
/// Generate the HTML support matrix, or check it is up to date.
struct Args {
    #[argh(subcommand)]
    command: SubCommand,
}

impl Args {
    fn input_file(&self) -> &Path {
        match self.command {
            SubCommand::Generate(SubCommandGenerate { ref input_file, .. }) => input_file,
            SubCommand::Check(SubCommandCheck { ref input_file, .. }) => input_file,
        }
    }

    fn output_file(&self) -> &Path {
        match self.command {
            SubCommand::Generate(SubCommandGenerate { ref output_file, .. }) => output_file,
            SubCommand::Check(SubCommandCheck { ref output_file, .. }) => output_file,
        }
    }
}

#[derive(argh::FromArgs)]
#[argh(subcommand)]
enum SubCommand {
    Generate(SubCommandGenerate),
    Check(SubCommandCheck),
}

#[derive(argh::FromArgs)]
#[argh(subcommand, name = "generate")]
/// generate the HTML support matrix
struct SubCommandGenerate {
    #[argh(positional)]
    /// path of the input YAML file
    input_file: PathBuf,
    #[argh(positional)]
    /// path of the HTML file to generate
    output_file: PathBuf,
}

#[derive(argh::FromArgs)]
#[argh(subcommand, name = "check")]
/// check that the generated HTML support matrix is up to date
struct SubCommandCheck {
    #[argh(positional)]
    /// path of the input YAML file
    input_file: PathBuf,
    #[argh(positional)]
    /// path of the HTML file to check
    output_file: PathBuf,
}

fn main() -> miette::Result<()> {
    let args: Args = argh::from_env();

    let input_file = fs::read_to_string(&args.input_file()).map_err(|source| Error::InputFile {
        path: args.input_file().into(),
        source,
    })?;

    let matrix = serde_yaml::from_str(&input_file).map_err(|source| {
        let err_span = miette::SourceSpan::from(source.location().unwrap().index());
        Error::Parsing {
            path: args.input_file().into(),
            src: input_file,
            err_span,
            source,
        }
    })?;

    validate_input(&matrix)?;

    let html = render_html(&matrix)?;

    match args.command {
        SubCommand::Generate(_) => {
            fs::write(args.output_file(), html).map_err(|source| Error::WritingOutputFile {
                path: args.output_file().into(),
                source,
            })?;
            Ok(())
        }
        SubCommand::Check(_) => {
            let existing_html = fs::read_to_string(args.output_file()).map_err(|source| Error::ReadingExistingFile {
                path: args.output_file().into(),
                source,
            })?;

            if existing_html == html {
                Ok(())
            } else {
                Err(Error::ExistingHtmlNotUpToDate {
                    path: args.output_file().into(),
                })?
            }
        }
    }
}

fn validate_input(matrix: &schema::Matrix) -> Result<(), Error> {
    for (_, board_info) in &matrix.boards {
        let invalid_functionality_name = board_info.support
            .keys()
            .find(|f| matrix.functionalities.iter().all(|functionality| functionality.name != **f));

        if let Some(f) = invalid_functionality_name {
            return Err(Error::InvalidFunctionalityNameBoard {
                found: f.to_owned(),
                board: board_info.name.to_owned(),
            });
        }
    }

    for (_, chip_info) in &matrix.chips {
        let invalid_functionality_name = chip_info.support
            .keys()
            .find(|f| matrix.functionalities.iter().all(|functionality| functionality.name != **f));

        if let Some(f) = invalid_functionality_name {
            return Err(Error::InvalidFunctionalityNameChip {
                found: f.to_owned(),
                chip: chip_info.name.to_owned(),
            });
        }
    }

    Ok(())
}

fn render_html(matrix: &schema::Matrix) -> Result<String, Error> {
    use minijinja::{Environment, context};

    #[derive(Debug, Serialize)]
    struct BoardSupport {
        chip: String,
        name: String,
        functionalities: Vec<FunctionalitySupport>,
    }

    #[derive(Debug, Serialize)]
    struct FunctionalitySupport {
        icon: String,
        description: String,
        // TODO: add comments
        // TODO: add the PR link
    }

    let mut boards = matrix.boards.iter().map(|(_, board_info)| {
        let board_name = &board_info.name;

        let functionalities = matrix.functionalities
            .iter()
            .map(|functionality_info| {
                let name = &functionality_info.name;

                let support_key = if let Some(support_info) = board_info.support.get(name) {
                    let status = support_info.status();
                    matrix.support_keys
                        .iter()
                        .find(|s| s.name == status)
                        .ok_or(Error::InvalidSupportKeyNameBoard {
                            found: status.to_owned(),
                            functionality: name.to_owned(),
                            board: board_name.to_owned(),
                        })?
                } else {
                    let chip = &board_info.chip;
                    // Implement chip info inheritance
                    let chip_info = matrix.chips.get(chip).ok_or(Error::InvalidChipName {
                        found: chip.to_owned(),
                        board: board_name.to_owned(),
                    })?;
                    let support_info = chip_info.support
                        .get(name)
                        .ok_or(Error::MissingSupportInfo {
                            board: board_name.to_owned(),
                            chip: board_info.chip.to_owned(),
                            functionality: functionality_info.title.to_owned(),
                        })?;
                    let status = support_info.status();
                    matrix.support_keys
                        .iter()
                        .find(|s| s.name == status)
                        .ok_or(Error::InvalidSupportKeyNameChip {
                            found: status.to_owned(),
                            functionality: name.to_owned(),
                            chip: chip_info.name.to_owned(),
                        })?
                };

                Ok(FunctionalitySupport {
                    icon: support_key.icon.to_owned(),
                    description: support_key.description.to_owned(),
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;


        let chip = matrix.chips.get(&board_info.chip).ok_or(Error::InvalidChipName {
            found: board_info.chip.to_owned(),
            board: board_name.to_owned(),
        })?;

        Ok(BoardSupport {
            chip: chip.name.to_owned(),
            name: board_name.to_owned(),
            functionalities,
        })
    }).collect::<Result<Vec<_>, Error>>()?;
    // TODO: read the order from the YAML file instead?
    boards.sort_unstable_by_key(|b| b.name.clone());

    let mut env = Environment::new();
    env.add_template("matrix", TABLE_TEMPLATE).unwrap();
    env.add_template("matrix_key", KEY_TEMPLATE).unwrap();

    let tmpl = env.get_template("matrix").unwrap();
    let matrix_html = tmpl.render(context!(matrix => matrix, boards => boards)).unwrap();

    let tmpl = env.get_template("matrix_key").unwrap();
    let key_html = tmpl.render(context!(matrix => matrix, boards => boards)).unwrap();

    // NOTE: We may want to return the table and its key separately later
    Ok(format!("{matrix_html}{key_html}\n"))
}

#[derive(Debug, thiserror::Error, Diagnostic)]
enum Error {
    #[error("could not find file `{path}`")]
    InputFile {
        path: PathBuf,
        source: io::Error,
    },
    #[error("could not parse YAML file `{path}`")]
    Parsing {
        path: PathBuf,
        #[source_code]
        src: String,
        #[label = "Syntax error"]
        err_span: miette::SourceSpan,
        source: serde_yaml::Error,
    },
    #[error("invalid chip name `{found}` for board `{board}`")]
    InvalidChipName {
        found: String,
        board: String,
    },
    #[error("invalid functionality name `{found}` for board `{board}`")]
    InvalidFunctionalityNameBoard {
        found: String,
        board: String,
    },
    #[error("invalid functionality name `{found}` for chip `{chip}`")]
    InvalidFunctionalityNameChip {
        found: String,
        chip: String,
    },
    #[error("invalid support key name `{found}` for functionality `{functionality}` for board `{board}`")]
    InvalidSupportKeyNameBoard {
        found: String,
        functionality: String,
        board: String,
    },
    #[error("invalid support key name `{found}` for functionality `{functionality}` for chip `{chip}`")]
    InvalidSupportKeyNameChip {
        found: String,
        functionality: String,
        chip: String,
    },
    #[error("missing support info on board `{board}` or chip `{chip}` regarding functionality `{functionality}`")]
    MissingSupportInfo {
        board: String,
        chip: String,
        functionality: String,
    },
    #[error("could not write the output HTML file `{path}`")]
    WritingOutputFile {
        path: PathBuf,
        source: io::Error,
    },
    #[error("could not read existing output HTML file `{path}`")]
    ReadingExistingFile {
        path: PathBuf,
        source: io::Error,
    },
    #[error("existing HTML file `{path}` is not up to date")]
    ExistingHtmlNotUpToDate {
        path: PathBuf,
    },
}

mod schema {
    use std::collections::HashMap;

    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct Matrix {
        pub support_keys: Vec<SupportKeyInfo>,
        pub functionalities: Vec<FunctionalityInfo>,
        pub chips: HashMap<String, ChipInfo>,
        pub boards: HashMap<String, BoardInfo>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct SupportKeyInfo {
        pub name: String,
        pub icon: String,
        pub description: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct FunctionalityInfo {
        pub name: String,
        pub title: String, // FIXME: rename this
        pub description: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct ChipInfo {
        pub name: String,
        pub description: Option<String>,
        pub support: HashMap<String, SupportInfo>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct BoardInfo {
        pub name: String,
        pub description: Option<String>,
        pub chip: String,
        pub support: HashMap<String, SupportInfo>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(untagged)]
    pub enum SupportInfo {
        StatusOnly(String),
        Details {
            status: String,
            comments: Option<String>,
            link: Option<String>,
        },
    }

    impl SupportInfo {
        pub fn status(&self) -> &str {
            match self {
                SupportInfo::StatusOnly(status) => status,
                SupportInfo::Details { status, .. } => status,
            }
        }
    }
}
