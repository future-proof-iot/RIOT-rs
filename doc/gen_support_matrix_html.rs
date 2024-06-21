#!/usr/bin/env -S cargo +nightly -Zscript
---cargo
[package]
edition = "2021"

[dependencies]
miette = { version = "7.2.0", features = ["fancy"] }
minijinja = { version = "2.0.3" }
serde = { version = "1.0.0", features = ["derive"] }
serde_yaml = { version = "0.9.34" } # TODO: use a maintained crate instead
thiserror = { version = "1.0.61" }
---

use std::{env, fs, io, path::PathBuf};

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
  {%- for _, support_key in matrix.support_keys|items %}
  <li class="no-marker">{{ support_key.icon }}&nbsp;&nbsp;{{ support_key.description }}</li>
  {%- endfor %}
</ul>
<style>
.no-marker::marker {
  content: '';
}
</style>
"##;

fn main() -> miette::Result<()> {
    // TODO: add a flag to only check, for CLI
    // TODO: maybe use argh instead
    let input_file_path = env::args()
        .nth(1)
        .expect("input file path as first CLI argument");
    let output_file_path = env::args()
        .nth(2)
        .expect("output file path as second CLI argument");

    let input_file = fs::read_to_string(&input_file_path).map_err(|source| Error::InputFile {
        path: input_file_path.clone().into(),
        source,
    })?;

    let matrix = serde_yaml::from_str(&input_file).map_err(|source| {
        let err_span = miette::SourceSpan::from(source.location().unwrap().index());
        Error::Parsing {
            path: input_file_path.into(),
            src: input_file,
            err_span,
            source,
        }
    })?;

    let html = render_html(&matrix)?;
    fs::write(output_file_path, html).expect("could not write the output HTML file");

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
        let functionalities = matrix.functionalities
            .iter()
            .map(|functionality_info| {
                let name = &functionality_info.name;

                let support_info = if let Some(support_info) = board_info.supports.get(name) {
                    support_info
                } else {
                    // Implement chip info inheritance
                    let chip_info = matrix.chips.get(&board_info.chip).ok_or(Error::InvalidChipName)?;
                    chip_info.supports.get(name).ok_or(Error::MissingSupportInfo {
                        board: board_info.name.to_owned(),
                        chip: board_info.chip.to_owned(),
                        functionality: functionality_info.title.to_owned(),
                    })?
                };

                // FIXME: make sure invalid functionality names in boards are rejected

                let status = support_info.status();
                let support_key = matrix.support_keys.get(status)
                    .ok_or(Error::InvalidSupportKeyName { found: status.to_owned() })?;

                Ok(FunctionalitySupport {
                    icon: support_key.icon.to_owned(),
                    description: support_key.description.to_owned(),
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;


        let chip = matrix.chips.get(&board_info.chip).ok_or(Error::InvalidChipName)?;

        Ok(BoardSupport {
            chip: chip.name.to_owned(),
            name: board_info.name.to_owned(),
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
    #[error("invalid chip name")] // FIXME: improve this error message
    InvalidChipName,
    #[error("invalid functionality name")] // FIXME: improve this error message
    InvalidFunctionalityName,
    #[error("invalid support key name `{found}`")] // FIXME: improve this error message
    InvalidSupportKeyName {
        found: String,
    },
    #[error("missing support info on board `{board}` or chip `{chip}` regarding functionality `{functionality}`")]
    MissingSupportInfo {
        board: String,
        chip: String,
        functionality: String,
    }
}

mod schema {
    use std::collections::HashMap;

    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct Matrix {
        pub support_keys: HashMap<String, SupportKeyInfo>,
        pub functionalities: Vec<FunctionalityInfo>,
        pub chips: HashMap<String, ChipInfo>,
        pub boards: HashMap<String, BoardInfo>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct SupportKeyInfo {
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
        pub supports: HashMap<String, SupportInfo>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct BoardInfo {
        pub name: String,
        pub description: Option<String>,
        pub chip: String,
        pub supports: HashMap<String, SupportInfo>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(untagged)]
    pub enum SupportInfo {
        StatusOnly(String),
        Details {
            status: String,
            comments: Option<String>,
            github_pr: Option<u32>,
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
