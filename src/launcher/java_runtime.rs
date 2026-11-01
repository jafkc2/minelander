use anyhow::{anyhow, Result};
use serde_json::Value;
use std::path::PathBuf;

use crate::java::JavaType;

pub fn detect_required_java(version_json: &Value) -> JavaType {
    let major = version_json["javaVersion"]["majorVersion"]
        .as_i64()
        .or_else(|| version_json["javaVersion"]["Version"].as_i64())
        .unwrap_or(0);

    match major {
        v if v >= 25 => JavaType::Java25,
        v if v >= 21 => JavaType::Java21,
        v if v > 8 || v == 0 => JavaType::Java17,
        _ => JavaType::Java8,
    }
}

pub fn get_java_binary(java_type: &JavaType, custom_path: Option<&str>) -> Result<PathBuf> {
    if let JavaType::Custom = java_type {
        if let Some(path) = custom_path {
            if !path.is_empty() {
                return Ok(PathBuf::from(path));
            }
        }
        return Err(anyhow!("Custom JRE path not found"));
    }

    let install_dir = java_type.installation_path()?;
    let binary_name = if std::env::consts::OS == "windows" {
        "bin/javaw.exe"
    } else {
        "bin/java"
    };

    let full_path = install_dir.join(binary_name);
    Ok(full_path)
}

pub fn get_jvm_optimizations(java_type: &JavaType) -> Vec<String> {
    let flags_str = match java_type {
        JavaType::Java8 => {
            "-XX:+UnlockExperimentalVMOptions -XX:+UnlockDiagnosticVMOptions \
             -XX:+AlwaysActAsServerClassMachine -XX:+ParallelRefProcEnabled \
             -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:+AggressiveOpts \
             -XX:MaxInlineLevel=15 -XX:MaxVectorSize=32 -XX:ThreadPriorityPolicy=1 \
             -XX:+UseNUMA -XX:+UseDynamicNumberOfGCThreads -XX:NmethodSweepActivity=1 \
             -XX:ReservedCodeCacheSize=350M -XX:-DontCompileHugeMethods \
             -XX:MaxNodeLimit=240000 -XX:NodeLimitFudgeFactor=8000 \
             -Dgraal.CompilerConfiguration=community"
        }
        _ => {
            "-XX:+UnlockExperimentalVMOptions -XX:+UnlockDiagnosticVMOptions \
             -XX:+AlwaysActAsServerClassMachine -XX:+AlwaysPreTouch \
             -XX:+DisableExplicitGC -XX:+UseNUMA -XX:NmethodSweepActivity=1 \
             -XX:ReservedCodeCacheSize=400M -XX:NonNMethodCodeHeapSize=12M \
             -XX:ProfiledCodeHeapSize=194M -XX:NonProfiledCodeHeapSize=194M \
             -XX:-DontCompileHugeMethods -XX:MaxNodeLimit=240000 \
             -XX:NodeLimitFudgeFactor=8000 -XX:+UseVectorCmov -XX:+PerfDisableSharedMem \
             -XX:+UseFastUnorderedTimeStamps -XX:+UseCriticalJavaThreadPriority \
             -XX:ThreadPriorityPolicy=1 -XX:AllocatePrefetchStyle=3"
        }
    };

    flags_str.split_whitespace().map(String::from).collect()
}
