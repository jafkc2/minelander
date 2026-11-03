use std::env;

use serde_json::Value;

use crate::{launcher::MinecraftData, minecraft::get_minecraft_dir};

pub fn get_version_specific_jvm_args(p: &Value, nativedir: &str) -> Vec<String> {
    let lib_dir = format!("{}/libraries", get_minecraft_dir());
    let separator = if cfg!(windows) { ";" } else { ":" };
    let version_id = p["id"].as_str().unwrap_or("");
    let mut arguments = Vec::new();

    if let Some(values) = p["arguments"]["jvm"].as_array() {
        for value in values {
            for argument in argument_values(value) {
                let argument = argument
                    .replace("${natives_directory}", nativedir)
                    .replace("${library_directory}", &lib_dir)
                    .replace("${classpath_separator}", separator)
                    .replace("${version_name}", version_id)
                    .replace("${launcher_name}", "Minelander")
                    .replace("${launcher_version}", env!("CARGO_PKG_VERSION"));
                if !argument.contains("${classpath}") && argument != "-cp" {
                    arguments.push(argument);
                }
            }
        }
    } else {
        arguments.push(format!("-Djava.library.path={nativedir}"));
    }

    arguments
}

pub fn get_minecraft_args(p: &Value, data: &MinecraftData) -> Vec<String> {
    let mut arguments = Vec::new();

    if let Some(values) = p["arguments"]["game"].as_array() {
        for value in values {
            for argument in argument_values(value) {
                if argument != "--demo" {
                    arguments.push(data.replace_placeholders(&argument));
                }
            }
        }
    } else if let Some(args_str) = p["minecraftArguments"].as_str() {
        for token in args_str.split_whitespace() {
            if token != "--demo" {
                arguments.push(data.replace_placeholders(token));
            }
        }
    }

    arguments
}

fn argument_values(value: &Value) -> Vec<String> {
    if let Some(value) = value.as_str() {
        return vec![value.to_string()];
    }

    if value.get("rules").is_some() {
        return Vec::new();
    }

    if let Some(values) = value.get("value") {
        if let Some(value) = values.as_str() {
            return vec![value.to_string()];
        }
        if let Some(values) = values.as_array() {
            return values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect();
        }
    }

    Vec::new()
}
