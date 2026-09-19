/*
 * Copyright 2026, Seqera Labs
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 * Original portions of this file are licensed under the MIT License.
 * See THIRD_PARTY_LICENSES
 */

use std::{
    env, fs,
    path::{Path, PathBuf},
};

use zed::serde_json::Value;
use zed_extension_api::{self as zed, http_client::HttpMethod};

struct NextflowExtension;

impl zed::Extension for NextflowExtension {
    #[allow(clippy::single_match_else)]
    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<zed::Command> {
        let settings =
            zed::settings::LspSettings::for_worktree(language_server_id.as_ref(), worktree)?;

        let java_path = resolve_java(worktree, settings.settings.as_ref())?;

        // users can provide a path to language server JAR in settings
        // this will mainly be used for testing with a locally built language server
        let extension_directory = env::current_dir()
            .map_err(|error| format!("Failed to resolve language-server path: {error}"))?;
        let jar_path = match parse_language_server_path(settings.settings.as_ref())? {
            Some(path) => extension_directory.join(path),
            None => {
                let language_version = parse_selected_language_version(settings.settings.as_ref())?;
                extension_directory.join(resolve_managed_language_server(
                    language_server_id,
                    language_version,
                )?)
            }
        };

        Ok(zed::Command {
            command: java_path,
            args: vec!["-jar".to_string(), jar_path.to_string_lossy().into_owned()],
            env: Vec::new(),
        })
    }

    fn language_server_initialization_options(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<Option<Value>> {
        zed::settings::LspSettings::for_worktree(language_server_id.as_ref(), worktree)
            .map(|settings| settings.initialization_options)
    }

    // it is important to send something for workspace settings or the nextflow language server
    // wont register a workspace - this sends the same defaults as the vscode plugin but allows
    // any user provided settings to override them
    fn language_server_workspace_configuration(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<Option<Value>> {
        let user_settings =
            zed::settings::LspSettings::for_worktree(language_server_id.as_ref(), worktree)?
                .settings
                .unwrap_or_else(|| zed::serde_json::json!({}));

        // defaults taken from vscode plugin
        // they are merged with user settings - with any user provided overwriting these
        let mut settings = zed::serde_json::json!({
            "nextflow": {
                "completion": {
                    "extended": false,
                    "maxItems": 100
                },
                "debug": false,
                "errorReportingMode": "warnings",
                "files": {
                    "exclude": [".git", ".lineage", ".nf-test", ".pixi", ".venv", "work"]
                },
                "formatting": {
                    "harshilAlignment": false,
                    "maheshForm": false,
                    "sortDeclarations": false
                }
            }
        });
        merge_json(user_settings, &mut settings);

        Ok(Some(settings))
    }

    fn new() -> Self {
        Self
    }
}

fn resolve_java(worktree: &zed::Worktree, settings: Option<&Value>) -> zed::Result<String> {
    let executable = match zed::current_platform().0 {
        zed::Os::Windows => "java.exe",
        zed::Os::Mac | zed::Os::Linux => "java",
    };
    let configured_java_home = settings
        .and_then(|settings| settings.get("nextflow"))
        .and_then(|nextflow| nextflow.get("java"))
        .and_then(|java| java.get("home"))
        .and_then(Value::as_str)
        .filter(|java_home| !java_home.is_empty());

    let java_path = match configured_java_home {
        Some(java_home) => {
            let path = PathBuf::from(java_home).join("bin").join(executable);
            if !fs::metadata(&path).is_ok_and(|metadata| metadata.is_file()) {
                return Err(
                    "The nextflow.java.home setting does not point to a valid Java install"
                        .to_string(),
                );
            }

            Some(path.to_string_lossy().into_owned())
        }
        None => None,
    };
    let java_path = java_path
        .or_else(|| {
            let shell_env = worktree.shell_env();
            let (_, java_home) = shell_env
                .iter()
                .find(|(name, _)| name == "JAVA_HOME")?;

            let path = PathBuf::from(java_home).join("bin").join(executable);
            let Ok(metadata) = fs::metadata(&path) else {
                return None;
            };
            if !metadata.is_file() {
                return None;
            }

            Some(path.to_string_lossy().into_owned())
        })
        .or_else(|| worktree.which("java"))
        .ok_or_else(|| {
            "Could not locate Java - configure nextflow.java.home or make Java available through JAVA_HOME or PATH"
                .to_string()
        })?;

    let output = zed::process::Command::new(&java_path)
        .arg("-version")
        .output()
        .map_err(|error| format!("Failed to query Java at {java_path}: {error}"))?;
    match output.status {
        Some(0) => (),
        Some(status) => {
            return Err(format!(
                "Failed to query Java at {java_path}: process exited with status {status}"
            ));
        }
        None => {
            return Err(format!(
                "Failed to query Java at {java_path}: process terminated without an exit status"
            ));
        }
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let major_version = stderr
        .split_once("version \"")
        .and_then(|(_, rest)| rest.split_once('"'))
        .map(|(version, _)| version.strip_prefix("1.").unwrap_or(version))
        .and_then(|version| {
            version
                .split(|character: char| !character.is_ascii_digit())
                .next()?
                .parse::<u32>()
                .ok()
        })
        .ok_or_else(|| format!("Could not determine Java version from: {stderr}"))?;

    if major_version < 17 {
        return Err(format!(
            "Java 17 or later is required by the Nextflow language server. Found Java {major_version} at {java_path}"
        ));
    }

    Ok(java_path)
}

fn parse_language_server_path(settings: Option<&Value>) -> zed::Result<Option<&Path>> {
    let path = settings
        .and_then(|settings| settings.get("nextflow"))
        .and_then(|nextflow| nextflow.get("languageServer"))
        .and_then(|language_server| language_server.get("path"))
        .and_then(Value::as_str)
        .filter(|path| !path.is_empty());
    let Some(path) = path else {
        return Ok(None);
    };
    let path = Path::new(path);
    let metadata = fs::metadata(path).map_err(|error| {
        format!(
            "Cannot access language-server path {}: {error}",
            path.display()
        )
    })?;
    if !metadata.is_file() {
        return Err(format!(
            "Language-server path is not a file: {}",
            path.display()
        ));
    }

    Ok(Some(path))
}

fn parse_selected_language_version(settings: Option<&Value>) -> zed::Result<&str> {
    let language_version = settings
        .and_then(|settings| settings.get("nextflow"))
        .and_then(|nextflow| nextflow.get("languageVersion"))
        .and_then(Value::as_str)
        .unwrap_or("26.04");

    // TODO: this does mean newer nextflow releases require a plugin update to work which
    // might not be ideal - currently only used for early validation so could accept it
    // and let the download fail later
    let supported_versions = ["26.04", "25.10", "25.04", "24.10"];

    if !supported_versions.contains(&language_version) {
        return Err(format!(
            "Unsupported Nextflow language version: {language_version}"
        ));
    }

    Ok(language_version)
}

fn resolve_managed_language_server(
    language_server_id: &zed::LanguageServerId,
    language_version: &str,
) -> zed::Result<String> {
    zed::set_language_server_installation_status(
        language_server_id,
        &zed::LanguageServerInstallationStatus::CheckingForUpdate,
    );
    let result: zed::Result<String> = (|| {
        let release_tag =
            get_latest_language_server_release(language_version)?.ok_or_else(|| {
                format!("No language-server release found for Nextflow {language_version}")
            })?;

        let cache_directory = format!("nextflow-language-server/v{language_version}");
        let jar_path = format!("{cache_directory}/{release_tag}.jar");

        if fs::metadata(&jar_path).is_ok_and(|metadata| metadata.is_file() && metadata.len() != 0) {
            return Ok(jar_path);
        }

        fs::create_dir_all(&cache_directory)
            .map_err(|error| format!("Failed to create language-server cache: {error}"))?;

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::Downloading,
        );

        zed::download_file(
            &format!(
                "https://github.com/nextflow-io/language-server/releases/download/{release_tag}/language-server-all.jar"
            ),
            &jar_path,
            zed::DownloadedFileType::Uncompressed,
        )
        .map_err(|error| {
            format!("Failed to download Nextflow language server {release_tag}: {error}")
        })?;

        Ok(jar_path)
    })();

    match result {
        Ok(jar_path) => {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::None,
            );
            Ok(jar_path)
        }
        Err(error) => {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Failed(error.clone()),
            );
            Err(error)
        }
    }
}

fn get_latest_language_server_release(language_version: &str) -> zed::Result<Option<String>> {
    let response = zed::http_client::HttpRequest::builder()
        .method(HttpMethod::Get)
        // may need to fetch more than 100 at some point
        // TODO: vscode supports GITHUB_TOKEN - may wish to implement that
        .url("https://api.github.com/repos/nextflow-io/language-server/releases?per_page=100")
        .build()?
        .fetch()?;

    let releases: Value = zed::serde_json::from_slice(&response.body)
        .map_err(|error| format!("Failed to parse response: {error}"))?;
    let releases = releases
        .as_array()
        .ok_or_else(|| "Failed to parse releases".to_string())?;
    let tag_prefix = format!("v{language_version}.");

    Ok(releases
        .iter()
        .filter_map(|release| {
            let tag = release.get("tag_name")?.as_str()?;
            let patch = tag.strip_prefix(&tag_prefix)?.parse::<u64>().ok()?;

            Some((patch, tag))
        })
        .max_by_key(|(patch, _)| *patch)
        .map(|(_, tag)| tag.to_owned()))
}

fn merge_json(source: Value, target: &mut Value) {
    match (source, target) {
        (Value::Object(source), Value::Object(target)) => {
            for (key, value) in source {
                match target.get_mut(&key) {
                    Some(target) => merge_json(value, target),
                    None => {
                        target.insert(key, value);
                    }
                }
            }
        }
        (source, target) => *target = source,
    }
}

zed::register_extension!(NextflowExtension);
